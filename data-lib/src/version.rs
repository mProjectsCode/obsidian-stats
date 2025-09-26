use std::cell::LazyCell;

use chumsky::{
    cache::{Cache, Cached},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

fn version_parser<'a>() -> impl Parser<'a, &'a str, Version> {
    let number = text::int(10).map(|s: &str| s.parse::<u32>().unwrap());
    let pre_release = just('-')
        .then(text::ident())
        .map(|(_, s): (_, &str)| s.to_string());

    let dot_number = group((just('.'), number)).map(|(_, n)| n);

    group((
        just('v').or_not(),
        number,
        dot_number,
        dot_number.or_not(),
        pre_release.or_not(),
    ))
    .map(|(_, major, minor, patch, pre)| Version {
        major,
        minor,
        patch: patch.unwrap_or(0),
        pre_release: pre,
    })
}

#[derive(Default)]
struct VersionParser;
impl Cached for VersionParser {
    type Parser<'a> = Box<dyn Parser<'a, &'a str, Version> + 'a>;

    fn make_parser<'a>(self) -> Self::Parser<'a> {
        Box::new(version_parser())
    }
}

thread_local! {
    static VERSION_PARSER: LazyCell<Cache<VersionParser>> = LazyCell::new(Cache::default);
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Hash)]
#[wasm_bindgen]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    #[wasm_bindgen(getter_with_clone)]
    pub pre_release: Option<String>,
}

#[wasm_bindgen]
impl Version {
    #[wasm_bindgen(constructor)]
    pub fn new(major: u32, minor: u32, patch: u32, pre_release: Option<String>) -> Self {
        Version {
            major,
            minor,
            patch,
            pre_release,
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        VERSION_PARSER.with(|parser| parser.get().parse(input).into_output())
    }

    pub fn validate(input: &str) -> bool {
        VERSION_PARSER.with(|parser| parser.get().parse(input).has_output())
    }

    pub fn to_fancy_string(&self) -> String {
        let mut version = format!("{}.{}.{}", self.major, self.minor, self.patch);
        if let Some(ref pre) = self.pre_release {
            version.push_str(&format!("-{pre}"));
        }
        version
    }

    pub fn get_major(&self) -> Self {
        Version {
            major: self.major,
            minor: 0,
            patch: 0,
            pre_release: None,
        }
    }

    pub fn get_minor(&self) -> Self {
        Version {
            major: self.major,
            minor: self.minor,
            patch: 0,
            pre_release: None,
        }
    }

    pub fn get_patch(&self) -> Self {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch,
            pre_release: None,
        }
    }

    pub fn earlier_than(&self, other: &Version) -> bool {
        self < other
    }

    pub fn later_than(&self, other: &Version) -> bool {
        self > other
    }

    pub fn equals(&self, other: &Version) -> bool {
        self == other
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then(other.pre_release.cmp(&self.pre_release))
    }
}

#[test]
fn version_compare() {
    let v1 = Version::new(1, 0, 0, None);
    let v2 = Version::new(1, 0, 1, None);
    let v3 = Version::new(1, 1, 0, None);
    let v4 = Version::new(2, 0, 0, None);

    assert!(v1 < v2);
    assert!(v2 < v3);
    assert!(v3 < v4);
    assert!(v1 < v3);
    assert!(v1 < v4);
    assert!(v2 < v4);
}

#[test]
fn version_compare_with_pre_release() {
    let v1 = Version::new(0, 1, 0, None);
    let v2 = Version::new(1, 0, 0, Some("alpha".to_string()));
    let v3 = Version::new(1, 0, 0, None);
    let v4 = Version::new(1, 0, 1, None);

    assert!(v1 < v2);
    assert!(v2 < v3);
    assert!(v3 < v4);
    assert!(v1 < v3);
    assert!(v1 < v4);
    assert!(v2 < v4);
}
