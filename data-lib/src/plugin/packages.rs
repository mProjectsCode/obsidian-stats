use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Deno,
}

impl PackageManager {
    pub const fn get_lock_file_name(&self) -> &'static [&'static str] {
        match self {
            PackageManager::Npm => &["package-lock.json"],
            PackageManager::Yarn => &["yarn.lock"],
            PackageManager::Pnpm => &["pnpm-lock.yaml"],
            PackageManager::Bun => &["bun.lockb", "bun.lock"],
            PackageManager::Deno => &["lock.json"],
        }
    }

    pub fn get_identifier(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Bun => "bun",
            PackageManager::Deno => "deno",
        }
    }

    pub fn from_identifier(identifier: &str) -> Option<Self> {
        match identifier {
            "npm" => Some(PackageManager::Npm),
            "yarn" => Some(PackageManager::Yarn),
            "pnpm" => Some(PackageManager::Pnpm),
            "bun" => Some(PackageManager::Bun),
            "deno" => Some(PackageManager::Deno),
            _ => None,
        }
    }

    pub fn matches_lock_file(&self, file_name: &str) -> bool {
        self.get_lock_file_name().iter().any(|name| {
            if file_name.contains("/") {
                file_name.ends_with(format!("/{name}").as_str())
            } else {
                file_name == *name
            }
        })
    }

    pub fn iter_variants() -> impl Iterator<Item = PackageManager> {
        PackageManager::variants()
            .into_iter()
            .cloned()
    }

    pub const fn variants() -> &'static [PackageManager] {
        &[
            PackageManager::Npm,
            PackageManager::Yarn,
            PackageManager::Pnpm,
            PackageManager::Bun,
            PackageManager::Deno,
        ]
    }

    pub fn find_package_managers(file_paths: &[String]) -> Vec<Self> {
        let mut package_managers = Vec::new();

        for file_path in file_paths {
            for pm in PackageManager::iter_variants() {
                if pm.matches_lock_file(file_path) && !package_managers.contains(&pm) {
                    package_managers.push(pm);
                }
            }
        }

        package_managers
    }
}

impl Serialize for PackageManager {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.get_identifier())
    }
}

impl<'de> Deserialize<'de> for PackageManager {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let identifier = String::deserialize(deserializer)?;
        PackageManager::from_identifier(&identifier).ok_or_else(|| {
            serde::de::Error::custom(format!("Unknown package manager identifier: {identifier}"))
        })
    }
}
