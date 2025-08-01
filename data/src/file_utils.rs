use std::path::Path;

pub fn empty_dir(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    std::fs::create_dir(path)?;

    Ok(())
}

pub fn write_in_chunks<T: serde::Serialize>(
    path: &Path,
    data: &[T],
    chunk_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    for (i, chunk) in data.chunks(chunk_size).enumerate() {
        let data_string = serde_json::to_string(chunk)?;
        std::fs::write(path.join(format!("chunk_{i}.json")), data_string)?;
    }
    Ok(())
}

pub fn read_chunked_data<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let mut data = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
            let chunk_data: Vec<T> = serde_json::from_str(&std::fs::read_to_string(entry.path())?)?;
            data.extend(chunk_data);
        }
    }
    Ok(data)
}
