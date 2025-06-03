use serde::{Deserialize, Serialize};

use crate::date::Date;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    pub date: Date,
    pub hash: String,
}

impl Commit {
    pub fn from_git_log(log: String) -> Vec<Self> {
        log.lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line[1..line.len() - 1].splitn(2, ' ').collect();
                if parts.len() == 2 {
                    Some(Self {
                        date: parts[0]
                            .split('T')
                            .next()
                            .and_then(|str| Date::from_string(str))
                            .expect("Failed to parse date"),
                        hash: parts[1].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
