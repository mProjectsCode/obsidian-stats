use serde::{Deserialize, Serialize};
use tsify::Tsify;

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
                            .and_then(Date::from_string)
                            .expect("Failed to parse date"),
                        hash: parts[1].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn to_string_commit(&self) -> StringCommit {
        StringCommit {
            date: self.date.to_fancy_string(),
            hash: self.hash.clone(),
        }
    }
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct StringCommit {
    pub date: String,
    pub hash: String,
}