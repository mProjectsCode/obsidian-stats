use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::Path,
};

use serde::{Serialize, de::DeserializeOwned};

pub fn read_json_or_default<T>(path: &Path) -> T
where
    T: DeserializeOwned + Default,
{
    File::open(path)
        .ok()
        .and_then(|file| serde_json::from_reader(BufReader::new(file)).ok())
        .unwrap_or_default()
}

pub fn write_json_atomic<T: Serialize>(
    path: &Path,
    data: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;

    let millis = now_unix_seconds();
    let tmp_path = parent.join(format!(
        ".tmp_{}_{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("state"),
        millis
    ));
    let backup_path = parent.join(format!(
        ".bak_{}_{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("state"),
        millis
    ));

    let tmp_file = File::create(&tmp_path)?;
    let mut writer = BufWriter::new(tmp_file);
    if should_pretty_format(path) {
        serde_json::to_writer_pretty(&mut writer, data)?;
    } else {
        serde_json::to_writer(&mut writer, data)?;
    }
    writer.flush()?;

    if path.exists() {
        fs::rename(path, &backup_path)?;
    }

    match fs::rename(&tmp_path, path) {
        Ok(()) => {
            if backup_path.exists() {
                fs::remove_file(backup_path)?;
            }
            Ok(())
        }
        Err(e) => {
            if backup_path.exists() {
                let _ = fs::rename(backup_path, path);
            }
            if tmp_path.exists() {
                let _ = fs::remove_file(tmp_path);
            }
            Err(Box::new(e))
        }
    }
}

fn should_pretty_format(path: &Path) -> bool {
    let mut components = path
        .components()
        .filter_map(|component| component.as_os_str().to_str());

    while let Some(component) = components.next() {
        if component == "out"
            && let Some(next_component) = components.next()
            && next_component == "state"
        {
            return true;
        }
    }

    false
}

pub fn now_unix_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn is_fresh(last_checked_unix: i64, refresh_days: i64) -> bool {
    let age = now_unix_seconds() - last_checked_unix;
    age >= 0 && age < refresh_days * 24 * 60 * 60
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{is_fresh, now_unix_seconds, should_pretty_format};

    #[test]
    fn is_fresh_for_recent_timestamp() {
        let now = now_unix_seconds();
        assert!(is_fresh(now - 60, 1));
    }

    #[test]
    fn is_not_fresh_when_older_than_window() {
        let now = now_unix_seconds();
        assert!(!is_fresh(now - (3 * 24 * 60 * 60), 2));
    }

    #[test]
    fn future_timestamp_is_not_fresh() {
        let now = now_unix_seconds();
        assert!(!is_fresh(now + 120, 1));
    }

    #[test]
    fn pretty_format_enabled_for_state_path() {
        assert!(should_pretty_format(Path::new(
            "./out/state/clone-state.json"
        )));
    }

    #[test]
    fn pretty_format_disabled_for_non_state_path() {
        assert!(!should_pretty_format(Path::new(
            "./out/plugin-repo-data/chunk_0.json"
        )));
    }
}
