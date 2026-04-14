use regex::Regex;

const MIN_BLOB_LEN: usize = 1024;

pub(super) fn detect_base64(source: &str) -> (u32, u32) {
    let re = match Regex::new(r"[A-Za-z0-9+/]{1024,}={0,2}") {
        Ok(re) => re,
        Err(_) => return (0, 0),
    };

    let mut count = 0u32;
    let mut largest = 0u32;

    for blob in re.find_iter(source).map(|m| m.as_str()) {
        if !looks_like_base64(blob) {
            continue;
        }

        count += 1;
        let len = blob.len() as u32;
        if len > largest {
            largest = len;
        }
    }

    (count, largest)
}

fn looks_like_base64(input: &str) -> bool {
    if input.len() < MIN_BLOB_LEN {
        return false;
    }

    if !input.len().is_multiple_of(4) {
        return false;
    }

    let mut padding = 0usize;
    for (idx, ch) in input.chars().enumerate() {
        let is_valid = ch.is_ascii_alphanumeric() || ch == '+' || ch == '/' || ch == '=';
        if !is_valid {
            return false;
        }

        if ch == '=' {
            padding += 1;
            if idx < input.len() - 2 {
                return false;
            }
        }
    }

    padding <= 2
}
