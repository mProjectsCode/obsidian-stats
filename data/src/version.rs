use std::cell::LazyCell;

use chumsky::{
    cache::{Cache, Cached},
    prelude::*,
};

fn version_parser<'a>() -> impl Parser<'a, &'a str, Version> {
    let number = text::int(10).map(|s: &str| s.parse::<u32>().unwrap());
    let pre_release = just('-')
        .then(text::ident())
        .map(|(_, s): (_, &str)| s.to_string());

    let version = group((
        number,
        just('.'),
        number,
        just('.'),
        number,
        pre_release.or_not(),
    ))
    .map(|(major, _, minor, _, patch, pre)| Version {
        major,
        minor,
        patch,
        pre_release: pre,
    });

    version
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
}

impl Version {
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

    pub fn to_string(&self) -> String {
        let mut version = format!("{}.{}.{}", self.major, self.minor, self.patch);
        if let Some(ref pre) = self.pre_release {
            version.push_str(&format!("-{}", pre));
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
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.major, self.minor, self.patch, &self.pre_release).cmp(&(
            other.major,
            other.minor,
            other.patch,
            &other.pre_release,
        ))
    }
}
