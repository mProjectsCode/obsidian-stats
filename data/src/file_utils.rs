use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    time::SystemTime,
};

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

    ensure_dir(&tmp_path)?;
    write_in_chunks(&tmp_path, data, chunk_size)?;

    if path.exists() {
        std::fs::rename(path, &backup_path)?;
    }

    match std::fs::rename(&tmp_path, path) {
        Ok(()) => {
            if backup_path.exists() {
                std::fs::remove_dir_all(&backup_path)?;
            }
            Ok(())
        }
        Err(e) => {
            if backup_path.exists() {
                let _ = std::fs::rename(&backup_path, path);
            }
            if tmp_path.exists() {
                let _ = std::fs::remove_dir_all(&tmp_path);
            }
            Err(Box::new(e))
        }
    }
}

pub fn read_chunked_data<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let mut data = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
            let file = File::open(entry.path())?;
            let mut reader = BufReader::new(file);
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes)?;
            let chunk_data: Vec<T> = serde_json::from_slice(&bytes)?;
            data.extend(chunk_data);
        }
    }
    Ok(data)
}

pub fn read_chunked_data_or_default<T: serde::de::DeserializeOwned>(path: &Path) -> Vec<T> {
    read_chunked_data(path).unwrap_or_default()
}
