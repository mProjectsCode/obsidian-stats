use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    common::{
        FILE_EXT_INCLUDED, LOC_EXCLUDED, NamedDataPoint, increment_named_data_points, to_percentage,
    },
    plugin::PluginRepoDataPoints,
};

use super::{PluginDataArray, PluginDataArrayView};

#[wasm_bindgen]
impl PluginDataArrayView {
    pub fn repo_data_points(&self, data: &PluginDataArray) -> PluginRepoDataPoints {
        let mut points = PluginRepoDataPoints::default();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            for package_manager in &repo_data.package_managers {
                increment_named_data_points(
                    &mut points.package_managers,
                    package_manager.get_identifier(),
                    1.0,
                );
            }
            if repo_data.package_managers.is_empty() {
                points.no_package_managers += 1.0;
            }

            for bundler in &repo_data.bundlers {
                increment_named_data_points(&mut points.bundlers, bundler.get_identifier(), 1.0);
            }
            if repo_data.bundlers.is_empty() {
                points.no_bundlers += 1.0;
            }

            for testing_framework in &repo_data.testing_frameworks {
                increment_named_data_points(
                    &mut points.testing_frameworks,
                    testing_framework.get_identifier(),
                    1.0,
                );
            }
            if repo_data.testing_frameworks.is_empty() {
                points.no_testing_frameworks += 1.0;
            }

            for dependency in &repo_data.dependencies {
                increment_named_data_points(&mut points.dependencies, dependency, 1.0);
            }
            for dev_dependency in &repo_data.dev_dependencies {
                increment_named_data_points(&mut points.dependencies, dev_dependency, 1.0);
            }

            if repo_data.uses_typescript {
                points.typescript += 1.0;
            }
            if repo_data.has_beta_manifest {
                points.beta_manifest += 1.0;
            }
        });

        let total_plugins = self.indices.len() as f64;
        points.package_managers.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, total_plugins);
        });
        points.bundlers.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, total_plugins);
        });
        points.testing_frameworks.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, total_plugins);
        });
        points.dependencies.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, total_plugins);
        });
        to_percentage(&mut points.no_package_managers, total_plugins);
        to_percentage(&mut points.no_bundlers, total_plugins);
        to_percentage(&mut points.no_testing_frameworks, total_plugins);
        to_percentage(&mut points.typescript, total_plugins);
        to_percentage(&mut points.beta_manifest, total_plugins);

        points
    }

    /// Named data points for mismatched data between the plugin's repo data and the current entry in the community list.
    /// The data is in percentage form.
    pub fn mismatched_data(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();
        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };
            let Some(manifest) = repo_data.manifest.as_ref() else {
                return;
            };

            if manifest.description.as_deref() != Some(item.data.current_entry.description.as_str())
            {
                increment_named_data_points(&mut points, "Description mismatch", 1.0);
            }
            if manifest.name.as_deref() != Some(item.data.current_entry.name.as_str()) {
                increment_named_data_points(&mut points, "Name mismatch", 1.0);
            }
            if manifest.author.as_deref() != Some(item.data.current_entry.author.as_str()) {
                increment_named_data_points(&mut points, "Author mismatch", 1.0);
            }
        });

        points.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, self.indices.len() as f64);
        });
        points
    }

    /// Usage percentages of optional manifest fields across all plugins in the view.
    pub fn optional_manifest_fields(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();
        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };
            let Some(manifest) = repo_data.manifest.as_ref() else {
                return;
            };

            if manifest.funding_url.is_some() {
                increment_named_data_points(&mut points, "Has funding URL", 1.0);
            }
            if manifest.author_url.is_some() {
                increment_named_data_points(&mut points, "Has author URL", 1.0);
            }
            if manifest.help_url.is_some() {
                increment_named_data_points(&mut points, "Has help URL", 1.0);
            }
        });

        points.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, self.indices.len() as f64);
        });
        points
    }

    pub fn desktop_only_data(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();
        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                increment_named_data_points(&mut points, "Unknown", 1.0);
                return;
            };
            let Some(manifest) = repo_data.manifest.as_ref() else {
                increment_named_data_points(&mut points, "Unknown", 1.0);
                return;
            };

            match manifest.is_desktop_only {
                Some(true) => {
                    increment_named_data_points(&mut points, "Desktop only", 1.0);
                }
                Some(false) => {
                    increment_named_data_points(&mut points, "Mobile compatible", 1.0);
                }
                None => {
                    increment_named_data_points(&mut points, "Not specified", 1.0);
                }
            }
        });

        points.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, self.indices.len() as f64);
        });
        points
    }

    pub fn lines_of_code_by_language(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            repo_data.lines_of_code.iter().for_each(|(lang, count)| {
                increment_named_data_points(&mut points, lang, *count as f64);
            });
        });

        points
            .into_iter()
            .filter(|point| point.value > 10_000.0 && !LOC_EXCLUDED.contains(&point.name.as_str()))
            .collect()
    }

    pub fn lines_of_code_by_language_usage(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            repo_data.lines_of_code.iter().for_each(|(lang, _)| {
                increment_named_data_points(&mut points, lang, 1.0);
            });
        });

        points
            .into_iter()
            .filter(|point| !LOC_EXCLUDED.contains(&point.name.as_str()))
            .collect()
    }

    pub fn file_count_by_extension(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            repo_data.file_type_counts.iter().for_each(|(ext, count)| {
                increment_named_data_points(&mut points, ext, *count as f64);
            });
        });

        points
            .into_iter()
            .filter(|point| FILE_EXT_INCLUDED.contains(&point.name.to_lowercase().as_str()))
            .collect()
    }

    pub fn lines_of_code_distribution(&self, data: &PluginDataArray) -> Vec<u32> {
        let mut tmp: Vec<_> = self
            .iter_data(data)
            .map(|item| {
                let Some(repo_data) = item.repo_data() else {
                    return 0;
                };

                repo_data
                    .lines_of_code
                    .iter()
                    .filter(|(lang, _)| !LOC_EXCLUDED.contains(&lang.as_str()))
                    .map(|(_, loc)| loc)
                    .sum::<usize>() as u32
            })
            .filter(|&count| count > 0)
            .collect();

        tmp.sort_by(|a, b| b.cmp(a));
        tmp
    }

    pub fn file_count_distribution(&self, data: &PluginDataArray) -> Vec<u32> {
        let mut tmp: Vec<_> = self
            .iter_data(data)
            .map(|item| {
                let Some(repo_data) = item.repo_data() else {
                    return 0;
                };

                repo_data
                    .file_type_counts
                    .iter()
                    .filter(|(ext, _)| FILE_EXT_INCLUDED.contains(&ext.to_lowercase().as_str()))
                    .map(|(_, count)| count)
                    .sum::<usize>() as u32
            })
            .filter(|&count| count > 0)
            .collect();

        tmp.sort_by(|a, b| b.cmp(a));
        tmp
    }

    pub fn release_size_distribution(&self, data: &PluginDataArray) -> Vec<u32> {
        let mut tmp: Vec<_> = self
            .iter_data(data)
            .filter_map(|item| {
                let repo_data = item.repo_data()?;
                repo_data
                    .latest_release_main_js_size_bytes
                    .map(|size| size.min(u32::MAX as u64) as u32)
            })
            .filter(|&size| size > 0)
            .collect();

        tmp.sort_by(|a, b| b.cmp(a));
        tmp
    }

    /// Return plugin IDs sorted by latest release `main.js` size, largest first.
    pub fn top_release_size_plugin_ids(&self, data: &PluginDataArray, count: usize) -> Vec<String> {
        top_release_sizes(self, data)
            .into_iter()
            .take(count)
            .map(|(id, _)| id)
            .collect()
    }

    /// Return top plugins by latest release `main.js` size.
    /// The `name` field contains the plugin ID and the `value` field contains bytes.
    pub fn top_release_size_plugins(
        &self,
        data: &PluginDataArray,
        count: usize,
    ) -> Vec<NamedDataPoint> {
        top_release_sizes(self, data)
            .into_iter()
            .take(count)
            .map(|(id, size)| NamedDataPoint {
                name: id,
                value: size as f64,
            })
            .collect()
    }

    pub fn es_version_distribution(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            match &repo_data.estimated_target_es_version {
                Some(version) => increment_named_data_points(&mut points, version, 1.0),
                None => increment_named_data_points(&mut points, "Unknown", 1.0),
            }
        });

        points
    }

    pub fn release_fetch_status_distribution(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            match repo_data.latest_release_fetch_status.as_deref() {
                Some(status) if !status.is_empty() => {
                    increment_named_data_points(&mut points, status, 1.0)
                }
                _ => increment_named_data_points(&mut points, "unknown", 1.0),
            }
        });

        points
    }

    pub fn main_js_minified_distribution(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            match repo_data.main_js_is_probably_minified {
                Some(true) => increment_named_data_points(&mut points, "Probably minified", 1.0),
                Some(false) => {
                    increment_named_data_points(&mut points, "Probably not minified", 1.0)
                }
                None => increment_named_data_points(&mut points, "Unknown", 1.0),
            }
        });

        points
    }

    pub fn main_js_minification_score_distribution(&self, data: &PluginDataArray) -> Vec<f32> {
        let mut tmp: Vec<_> = self
            .iter_data(data)
            .filter_map(|item| {
                let repo_data = item.repo_data()?;
                repo_data
                    .main_js_minification_score
                    .map(|score| score.clamp(0.0, 1.0))
            })
            .collect();

        tmp.sort_by(|a, b| b.total_cmp(a));
        tmp
    }

    pub fn main_js_sourcemap_distribution(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            match repo_data.main_js_includes_sourcemap_comment {
                Some(true) => increment_named_data_points(
                    &mut points,
                    "Includes sourceMappingURL comment",
                    1.0,
                ),
                Some(false) => {
                    increment_named_data_points(&mut points, "No sourceMappingURL comment", 1.0)
                }
                None => increment_named_data_points(&mut points, "Unknown", 1.0),
            }
        });

        points
    }

    pub fn main_js_base64_blob_distribution(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        boolean_count_distribution(
            self,
            data,
            |repo| repo.main_js_large_base64_blob_count,
            "Has large base64 blobs",
            "No large base64 blobs",
        )
    }

    pub fn main_js_worker_usage_distribution(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        boolean_count_distribution(
            self,
            data,
            |repo| repo.main_js_worker_usage_count,
            "Uses Worker APIs",
            "No Worker API usage",
        )
    }

    pub fn main_js_webassembly_usage_distribution(
        &self,
        data: &PluginDataArray,
    ) -> Vec<NamedDataPoint> {
        boolean_count_distribution(
            self,
            data,
            |repo| repo.main_js_webassembly_usage_count,
            "Uses WebAssembly APIs",
            "No WebAssembly API usage",
        )
    }

    /// Return plugin IDs whose estimated target ES version matches `version`.
    pub fn es_version_plugin_ids(&self, data: &PluginDataArray, version: &str) -> Vec<String> {
        let mut ids: Vec<String> = self
            .iter_data(data)
            .filter_map(|item| {
                let repo_data = item.repo_data()?;
                let estimated_version = repo_data.estimated_target_es_version.as_deref()?;

                if estimated_version.eq_ignore_ascii_case(version) {
                    Some(item.id())
                } else {
                    None
                }
            })
            .collect();

        ids.sort();
        ids
    }

    pub fn i18n_usage(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut data_points = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            if repo_data.has_i18n_dependencies && repo_data.has_i18n_files {
                increment_named_data_points(
                    &mut data_points,
                    "Has i18n dependencies and files",
                    1.0,
                );
            } else if repo_data.has_i18n_dependencies {
                increment_named_data_points(&mut data_points, "Has i18n dependencies", 1.0);
            } else if repo_data.has_i18n_files {
                increment_named_data_points(&mut data_points, "Has i18n files", 1.0);
            }
        });

        data_points
    }

    pub fn i18n_plugin_ids(&self, data: &PluginDataArray) -> Vec<String> {
        let mut ids = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            if repo_data.has_i18n_dependencies || repo_data.has_i18n_files {
                ids.push(item.id());
            }
        });

        ids.sort();
        ids
    }
}

fn top_release_sizes(view: &PluginDataArrayView, data: &PluginDataArray) -> Vec<(String, u64)> {
    let mut tmp: Vec<(String, u64)> = view
        .iter_data(data)
        .filter_map(|item| {
            let repo_data = item.repo_data()?;
            let size = repo_data.latest_release_main_js_size_bytes?;

            if size == 0 {
                return None;
            }

            Some((item.id(), size))
        })
        .collect();

    tmp.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    tmp
}

fn boolean_count_distribution(
    view: &PluginDataArrayView,
    data: &PluginDataArray,
    get_count: impl Fn(&crate::plugin::PluginRepoData) -> Option<u32>,
    present_label: &str,
    absent_label: &str,
) -> Vec<NamedDataPoint> {
    let mut points = Vec::new();

    view.iter_data(data).for_each(|item| {
        let Some(repo_data) = item.repo_data() else {
            return;
        };

        match get_count(repo_data) {
            Some(count) if count > 0 => {
                increment_named_data_points(&mut points, present_label, 1.0)
            }
            Some(_) => increment_named_data_points(&mut points, absent_label, 1.0),
            None => increment_named_data_points(&mut points, "Unknown", 1.0),
        }
    });

    points
}
