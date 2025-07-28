use data_lib::license::LicenseData;
use serde_yaml;
use strsim::jaro;

use regex::Regex;

pub struct LicenseCleaned {
    name: String,
    text: String,
}

pub struct LicenseComparer {
    licenses: Vec<LicenseCleaned>,
    copyright_re: Regex,
    mit_re: Regex,
    gpl_3_re: Regex,
    lgpl_3_re: Regex,
    agpl_3_re: Regex,
}

impl Default for LicenseComparer {
    fn default() -> Self {
        LicenseComparer::new()
    }
}

impl LicenseComparer {
    pub fn new() -> Self {
        LicenseComparer {
            licenses: Vec::new(),
            copyright_re: Regex::new(
                r"^(?:copyright)\s*?(?:&copy;|\(c\)|&#(?:169|xa9;)|©)\s*[0-9]{4}.*$",
            )
            .unwrap(),
            mit_re: Regex::new(r"^mit\s+license").unwrap(),
            gpl_3_re: Regex::new(r"^gnu\s+general\s+public\s+license\s+version\s+3").unwrap(),
            lgpl_3_re: Regex::new(r"^gnu\s+lesser\s+general\s+public\s+license\s+version\s+3")
                .unwrap(),
            agpl_3_re: Regex::new(r"^gnu\s+affero\s+general\s+public\s+license\s+version\s+3")
                .unwrap(),
        }
    }

    pub fn init(&mut self) {
        let dir = std::fs::read_dir("../choosealicense.com/_licenses")
            .expect("Failed to read licenses directory");

        self.licenses = dir
            .filter_map(|entry| {
                let entry = entry.ok()?;

                let data = std::fs::read_to_string(entry.path()).ok()?;
                let parts: Vec<&str> = data.split("---").collect();
                if parts.len() <= 2 {
                    return None;
                }

                let frontmatter = serde_yaml::from_str::<LicenseData>(parts[1]).ok()?;

                Some(LicenseCleaned {
                    name: frontmatter.spdx_id,
                    text: parts[2].trim().to_lowercase().to_string(),
                })
            })
            .collect();

        println!("Loaded {} licenses", self.licenses.len());
    }

    /**
     * Returns the spdx-id of the best matching license or undefined if no match is found.
     */
    pub fn compare(&self, _plugin_id: &str, license: &str) -> Option<String> {
        let lower_case_license = license.to_lowercase();
        let lower_case_license = lower_case_license.trim();

        // println!("Comparing license: {}", lower_case_license);

        if self.mit_re.is_match(lower_case_license) {
            return Some("MIT".to_string());
        }

        if self.gpl_3_re.is_match(lower_case_license) {
            return Some("GPL-3.0".to_string());
        }

        if self.lgpl_3_re.is_match(lower_case_license) {
            return Some("LGPL-3.0".to_string());
        }

        if self.agpl_3_re.is_match(lower_case_license) {
            return Some("AGPL-3.0".to_string());
        }

        // we test if the license contains only a copyright notice like "Copyright (c) 2024 Moritz Jung"
        if self.copyright_re.is_match(lower_case_license) {
            println!("explicitly unlicensed: {license}");
            // if so we assume that the author reserves all rights
            return Some("explicitly unlicensed".to_string());
        }

        let exact_match = self.licenses.iter().find(|l| l.text == lower_case_license);
        if let Some(license) = exact_match {
            return Some(license.name.clone());
        }

        let mut scores = self
            .licenses
            .iter()
            .map(|l| {
                // jaro seems faster than normalized_levenshtein
                let score = jaro(lower_case_license, &l.text);
                (score, l.name.clone())
            })
            .collect::<Vec<_>>();

        scores.sort_by(|a, b| b.0.total_cmp(&a.0));

        // dbg!(plugin_id, &scores);

        if let Some((score, name)) = scores.first()
            && score > &0.85
        {
            return Some(name.clone());
        }

        None
    }
}
