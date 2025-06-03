use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bundler {
    Esbuild,
    Rollup,
    Webpack,
    Vite,
    Turbo,
}

impl Bundler {
    pub fn get_package_name(&self) -> Vec<&'static str> {
        match self {
            Bundler::Esbuild => vec!["esbuild"],
            Bundler::Rollup => vec!["rollup"],
            Bundler::Webpack => vec!["webpack"],
            Bundler::Vite => vec!["vite"],
            Bundler::Turbo => vec!["turbo"],
        }
    }

    pub fn get_identifier(&self) -> &'static str {
        match self {
            Bundler::Esbuild => "ESBuild",
            Bundler::Rollup => "Rollup",
            Bundler::Webpack => "Webpack",
            Bundler::Vite => "Vite",
            Bundler::Turbo => "Turbo",
        }
    }

    pub fn from_identifier(identifier: &str) -> Option<Self> {
        match identifier {
            "ESBuild" => Some(Bundler::Esbuild),
            "Rollup" => Some(Bundler::Rollup),
            "Webpack" => Some(Bundler::Webpack),
            "Vite" => Some(Bundler::Vite),
            "Turbo" => Some(Bundler::Turbo),
            _ => None,
        }
    }

    pub fn matches_dependency(&self, dependency: &str) -> bool {
        self.get_package_name()
            .iter()
            .any(|name| dependency == *name)
    }

    pub fn iter_variants() -> impl Iterator<Item = Bundler> {
        [
            Bundler::Esbuild,
            Bundler::Rollup,
            Bundler::Webpack,
            Bundler::Vite,
            Bundler::Turbo,
        ]
        .iter()
        .cloned()
    }

    pub fn find_bundlers(dependencies: &[&String]) -> Vec<Self> {
        let mut package_managers = Vec::new();

        for dependency in dependencies {
            for tf in Bundler::iter_variants() {
                if tf.matches_dependency(dependency) {
                    if !package_managers.contains(&tf) {
                        package_managers.push(tf);
                    }
                }
            }
        }

        package_managers
    }
}

impl Serialize for Bundler {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let identifier = self.get_identifier();
        serializer.serialize_str(identifier)
    }
}

impl<'de> Deserialize<'de> for Bundler {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let identifier = String::deserialize(deserializer)?;
        Bundler::from_identifier(&identifier).ok_or_else(|| {
            serde::de::Error::custom(format!("Unknown bundler identifier: {}", identifier))
        })
    }
}
