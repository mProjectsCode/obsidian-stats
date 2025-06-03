use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestingFramework {
    Jest,
    Mocha,
    Vitest,
    BunTest,
}

impl TestingFramework {
    pub fn get_package_name(&self) -> Vec<&'static str> {
        match self {
            TestingFramework::Jest => vec!["jest"],
            TestingFramework::Mocha => vec!["mocha"],
            TestingFramework::Vitest => vec!["vitest"],
            TestingFramework::BunTest => vec!["@types/bun", "bun-types"],
        }
    }

    pub fn get_identifier(&self) -> &'static str {
        match self {
            TestingFramework::Jest => "Jest",
            TestingFramework::Mocha => "Mocha",
            TestingFramework::Vitest => "Vitest",
            TestingFramework::BunTest => "BunTest",
        }
    }

    pub fn from_identifier(identifier: &str) -> Option<Self> {
        match identifier {
            "Jest" => Some(TestingFramework::Jest),
            "Mocha" => Some(TestingFramework::Mocha),
            "Vitest" => Some(TestingFramework::Vitest),
            "BunTest" => Some(TestingFramework::BunTest),
            _ => None,
        }
    }

    pub fn matches_dependency(&self, dependency: &str) -> bool {
        self.get_package_name()
            .iter()
            .any(|name| dependency == *name)
    }

    pub fn iter_variants() -> impl Iterator<Item = TestingFramework> {
        [
            TestingFramework::Jest,
            TestingFramework::Mocha,
            TestingFramework::Vitest,
            TestingFramework::BunTest,
        ]
        .iter()
        .cloned()
    }

    pub fn find_testing_frameworks(dependencies: &[&String]) -> Vec<Self> {
        let mut package_managers = Vec::new();

        for dependency in dependencies {
            for tf in TestingFramework::iter_variants() {
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

impl Serialize for TestingFramework {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.get_identifier())
    }
}

impl<'de> serde::Deserialize<'de> for TestingFramework {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let identifier = String::deserialize(deserializer)?;
        TestingFramework::from_identifier(&identifier).ok_or_else(|| {
            serde::de::Error::custom(format!("Unknown testing framework: {}", identifier))
        })
    }
}
