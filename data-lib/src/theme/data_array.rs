use std::ops::Index;

use wasm_bindgen::prelude::*;

use crate::{
    common::{CountMonthlyDataPoint, OverviewDataPoint, RemovedByReleaseDataPoint},
    date::Date,
    theme::ThemeData,
};

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct ThemeDataArray {
    #[wasm_bindgen(skip)]
    pub data: Vec<ThemeData>,
}

impl ThemeDataArray {
    pub fn new(data: Vec<ThemeData>) -> Self {
        Self { data }
    }
}

impl Index<usize> for ThemeDataArray {
    type Output = ThemeData;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

#[wasm_bindgen]
impl ThemeDataArray {
    pub fn view(&self) -> ThemeDataArrayView {
        ThemeDataArrayView::new(self.data.len())
    }
}

/// A view into a `FullPluginDataArray` that allows access to the underlying data without cloning.
#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct ThemeDataArrayView {
    #[wasm_bindgen(skip)]
    pub data: Vec<usize>,
}

impl ThemeDataArrayView {
    pub fn new(len: usize) -> Self {
        Self {
            data: (0..len).collect(),
        }
    }
}

#[wasm_bindgen]
impl ThemeDataArrayView {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, data: &ThemeDataArray, index: usize) -> ThemeData {
        data[self.data[index]].clone()
    }

    pub fn get_ids(&self, data: &ThemeDataArray) -> Vec<String> {
        self.data.iter().map(|&index| data[index].id()).collect()
    }

    pub fn get_by_id(&self, data: &ThemeDataArray, id: &str) -> Option<ThemeData> {
        self.data.iter().find_map(|&index| {
            let item = &data[index];
            if item.id() == id {
                Some(item.clone())
            } else {
                None
            }
        })
    }

    pub fn to_vec(&self, data: &ThemeDataArray) -> Vec<ThemeData> {
        self.data.iter().map(|&index| data[index].clone()).collect()
    }

    pub fn overview(&self, data: &ThemeDataArray) -> Vec<OverviewDataPoint> {
        self.data
            .iter()
            .map(|&index| {
                let plugin_data = &data[index];
                OverviewDataPoint {
                    id: plugin_data.id(),
                    name: plugin_data.name(),
                    author: plugin_data.author(),
                    repo: plugin_data.current_entry.repo.clone(),
                    repo_url: plugin_data.repo_url(),
                    added_commit: plugin_data.added_commit(),
                    removed_commit: plugin_data.removed_commit(),
                }
            })
            .collect()
    }

    pub fn monthly_count(&self, data: &ThemeDataArray) -> Vec<CountMonthlyDataPoint> {
        let mut plugin_count: i32 = 0;
        let mut plugin_count_with_removed: i32 = 0;

        let start_date = Date::new(2020, 11, 1);
        let end_date = Date::now();

        start_date
            .iterate_monthly_to(&end_date)
            .map(|date| {
                let mut new_themes = 0;
                let mut removed_themes = 0;

                for index in &self.data {
                    let theme_data = &data[*index];
                    if theme_data.released_in_month(&date) {
                        new_themes += 1;
                    }
                    if theme_data.removed_in_month(&date) {
                        removed_themes += 1;
                    }
                }

                plugin_count += new_themes - removed_themes;
                plugin_count_with_removed += new_themes;

                CountMonthlyDataPoint {
                    date: date.to_fancy_string(),
                    total: plugin_count.max(0) as u32,
                    total_with_removed: plugin_count_with_removed.max(0) as u32,
                    new: new_themes.max(0) as u32,
                    new_removed: removed_themes.max(0) as u32,
                }
            })
            .collect()
    }

    pub fn removed_by_release_month(
        &self,
        data: &ThemeDataArray,
    ) -> Vec<RemovedByReleaseDataPoint> {
        let start_date = Date::new(2020, 11, 1);
        let end_date = Date::now();

        start_date
            .iterate_monthly_to(&end_date)
            .map(|date| {
                let mut removed_count = 0;
                let mut count = 0;

                self.data.iter().for_each(|&index| {
                    let theme_data = &data[index];
                    if theme_data.released_in_month(&date) {
                        count += 1;
                        if theme_data.removed_commit.is_some() {
                            removed_count += 1;
                        }
                    }
                });

                RemovedByReleaseDataPoint {
                    date: date.to_fancy_string(),
                    percentage: if count > 0 {
                        (removed_count as f64 / count as f64) * 100.0
                    } else {
                        0.0
                    },
                }
            })
            .collect()
    }
}
