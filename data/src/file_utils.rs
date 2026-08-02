use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
    time::SystemTime,
};

/// Published data keeps fifty records per JSON chunk. Analysis consumes one
/// of these chunks at a time; larger chunks reduce directory overhead but
/// increase the bounded per-worker memory budget.
pub const DATA_CHUNK_SIZE: usize = 50;

pub fn empty_dir(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    std::fs::create_dir(path)?;

    Ok(())
}

pub fn ensure_dir(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

pub fn write_in_chunks<T: serde::Serialize>(
    path: &Path,
    data: &[T],
    chunk_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if chunk_size == 0 {
        return Err("chunk size must be greater than zero".into());
    }
    ensure_dir(path)?;

    for (i, chunk) in data.chunks(chunk_size).enumerate() {
        let chunk_path = path.join(format!("chunk_{i}.json"));
        let file = File::create(chunk_path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, chunk)?;
        writer.flush()?;
    }
    Ok(())
}

pub fn write_in_chunks_atomic<T: serde::Serialize>(
    path: &Path,
    data: &[T],
    chunk_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = path.parent().unwrap_or(Path::new("."));
    ensure_dir(parent)?;

    let millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let tmp_path = parent.join(format!(
        ".tmp_{}_{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("out"),
        millis
    ));
    let backup_path = parent.join(format!(
        ".bak_{}_{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("out"),
        millis
    ));

    publish_chunk_directory(path, &tmp_path, &backup_path, || {
        ensure_dir(&tmp_path)?;
        write_in_chunks(&tmp_path, data, chunk_size)
    })
}

/// Atomically publish chunks that have already been produced by a bounded
/// producer. The iterator is consumed while the destination is still in its
/// temporary directory, so a failed chunk never replaces the previous output.
pub fn write_chunks_atomic<T, I>(path: &Path, chunks: I) -> Result<(), Box<dyn std::error::Error>>
where
    T: serde::Serialize,
    I: IntoIterator<Item = Vec<T>>,
{
    let parent = path.parent().unwrap_or(Path::new("."));
    ensure_dir(parent)?;
    let suffix = unique_suffix();
    let tmp_path = parent.join(format!(".tmp_{}_{}", output_name(path), suffix));
    let backup_path = parent.join(format!(".bak_{}_{}", output_name(path), suffix));

    publish_chunk_directory(path, &tmp_path, &backup_path, || {
        ensure_dir(&tmp_path)?;
        for (index, chunk) in chunks.into_iter().enumerate() {
            write_chunk(&tmp_path, index, &chunk)?;
        }
        Ok(())
    })
}

pub fn write_chunks_atomic_results<T, I>(
    path: &Path,
    chunks: I,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: serde::Serialize,
    I: IntoIterator<Item = Result<Vec<T>, Box<dyn std::error::Error>>>,
{
    let parent = path.parent().unwrap_or(Path::new("."));
    ensure_dir(parent)?;
    let suffix = unique_suffix();
    let tmp_path = parent.join(format!(".tmp_{}_{}", output_name(path), suffix));
    let backup_path = parent.join(format!(".bak_{}_{}", output_name(path), suffix));

    publish_chunk_directory(path, &tmp_path, &backup_path, || {
        ensure_dir(&tmp_path)?;
        for (index, chunk) in chunks.into_iter().enumerate() {
            write_chunk(&tmp_path, index, &chunk?)?;
        }
        Ok(())
    })
}

fn write_chunk<T: serde::Serialize>(
    directory: &Path,
    index: usize,
    chunk: &[T],
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(directory.join(format!("chunk_{index}.json")))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, chunk)?;
    writer.flush()?;
    Ok(())
}

fn publish_chunk_directory<F>(
    path: &Path,
    tmp_path: &Path,
    backup_path: &Path,
    write: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce() -> Result<(), Box<dyn std::error::Error>>,
{
    if let Err(error) = write() {
        let _ = std::fs::remove_dir_all(tmp_path);
        return Err(error);
    }

    if path.exists() {
        if let Err(error) = std::fs::rename(path, backup_path) {
            let _ = std::fs::remove_dir_all(tmp_path);
            return Err(Box::new(error));
        }
    }

    match std::fs::rename(tmp_path, path) {
        Ok(()) => {
            if backup_path.exists() {
                std::fs::remove_dir_all(backup_path)?;
            }
            Ok(())
        }
        Err(error) => {
            if backup_path.exists() {
                let _ = std::fs::rename(backup_path, path);
            }
            let _ = std::fs::remove_dir_all(tmp_path);
            Err(Box::new(error))
        }
    }
}

fn output_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("out")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

/// A sorted, one-chunk-at-a-time reader for chunked JSON directories.
pub struct ChunkReader<T> {
    paths: Vec<PathBuf>,
    next: usize,
    marker: PhantomData<T>,
}

impl<T: serde::de::DeserializeOwned> Iterator for ChunkReader<T> {
    type Item = Result<Vec<T>, Box<dyn std::error::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.paths.get(self.next)?.clone();
        self.next += 1;

        let result = File::open(&path)
            .map(BufReader::new)
            .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
            .and_then(|reader| {
                serde_json::from_reader(reader)
                    .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
            });
        Some(result)
    }
}

pub fn read_chunked_data_iter<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<ChunkReader<T>, Box<dyn std::error::Error>> {
    let mut paths = std::fs::read_dir(path)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort_by(
        |left, right| match (chunk_index(left), chunk_index(right)) {
            (Some(left_index), Some(right_index)) => left_index.cmp(&right_index),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            _ => left.cmp(right),
        },
    );

    Ok(ChunkReader {
        paths,
        next: 0,
        marker: PhantomData,
    })
}

pub fn read_chunked_data<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let mut data = Vec::new();
    for chunk in read_chunked_data_iter(path)? {
        data.extend(chunk?);
    }
    Ok(data)
}

fn chunk_index(path: &Path) -> Option<usize> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| {
            stem.strip_prefix("chunk_")
                .or_else(|| stem.strip_prefix("chunk-"))
        })
        .and_then(|idx| idx.parse().ok())
}

pub fn read_chunked_data_or_default<T: serde::de::DeserializeOwned>(path: &Path) -> Vec<T> {
    read_chunked_data(path).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{read_chunked_data, read_chunked_data_iter, write_chunks_atomic_results};
    use tempfile::TempDir;

    fn test_directory() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn reads_numeric_chunks_in_numeric_order_and_supports_old_names() {
        let directory = test_directory();
        let directory = directory.path();
        fs::write(directory.join("chunk-10.json"), "[10]").unwrap();
        fs::write(directory.join("chunk_2.json"), "[2]").unwrap();
        fs::write(directory.join("chunk_1.json"), "[1]").unwrap();
        fs::write(directory.join("ignored.txt"), "not json").unwrap();

        assert_eq!(
            read_chunked_data::<u32>(&directory).unwrap(),
            vec![1, 2, 10]
        );
    }

    #[test]
    fn empty_chunk_directory_is_empty() {
        let directory = test_directory();
        let directory = directory.path();
        let chunks = read_chunked_data_iter::<u32>(&directory).unwrap();
        assert_eq!(
            chunks.collect::<Result<Vec<_>, _>>().unwrap(),
            Vec::<Vec<u32>>::new()
        );
    }

    #[test]
    fn malformed_chunk_is_reported_without_reading_later_chunks() {
        let directory = test_directory();
        let directory = directory.path();
        fs::write(directory.join("chunk_0.json"), "not json").unwrap();
        fs::write(directory.join("chunk_1.json"), "[1]").unwrap();

        let mut chunks = read_chunked_data_iter::<u32>(&directory).unwrap();
        assert!(chunks.next().unwrap().is_err());
        assert_eq!(chunks.next().unwrap().unwrap(), vec![1]);
    }

    #[test]
    fn failed_chunk_write_preserves_previous_directory() {
        let parent = test_directory();
        let output = parent.path().join("output");
        write_chunks_atomic_results(&output, [Ok(vec![1_u32])].into_iter()).unwrap();

        let error = std::io::Error::other("intentional chunk failure");
        assert!(
            write_chunks_atomic_results(
                &output,
                [
                    Ok(vec![2_u32]),
                    Err(Box::new(error) as Box<dyn std::error::Error>)
                ]
                .into_iter(),
            )
            .is_err()
        );
        assert_eq!(read_chunked_data::<u32>(&output).unwrap(), vec![1]);
    }
}
