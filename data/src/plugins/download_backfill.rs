use data_lib::{date::Date, spline::NaturalCubicSpline};
use hashbrown::{HashMap, HashSet};

use crate::plugins::{BorrowedPluginData, PluginDownloadStats};
use crate::progress::should_log_progress;

#[derive(Clone)]
struct DateRange {
    start: Date,
    end: Date,
}

impl DateRange {
    fn new(start: Date, end: Date) -> Self {
        Self { start, end }
    }

    fn contains(&self, date: &Date) -> bool {
        date >= &self.start && date <= &self.end
    }
}

struct ExcludedDates {
    dates: HashSet<Date>,
    ranges: Vec<DateRange>,
}

impl ExcludedDates {
    fn new(dates: Vec<Date>, ranges: Vec<DateRange>) -> Self {
        Self {
            dates: dates.into_iter().collect(),
            ranges,
        }
    }

    fn contains(&self, date: &Date) -> bool {
        self.dates.contains(date) || self.ranges.iter().any(|range| range.contains(date))
    }
}

struct DownloadPoint {
    date: Date,
    day: i32,
    downloads: u32,
}

fn dedup_points_by_day_max(points: &mut Vec<DownloadPoint>) {
    if points.len() < 2 {
        return;
    }

    // Requires: sorted by day ascending.
    let mut write_idx = 0usize;
    for read_idx in 1..points.len() {
        if points[read_idx].day == points[write_idx].day {
            if points[read_idx].downloads > points[write_idx].downloads {
                points[write_idx].downloads = points[read_idx].downloads;
            }
        } else {
            write_idx += 1;
            points[write_idx] = DownloadPoint {
                date: points[read_idx].date.clone(),
                day: points[read_idx].day,
                downloads: points[read_idx].downloads,
            };
        }
    }

    points.truncate(write_idx + 1);
}

struct SampleDates {
    dates: Vec<Date>,
    days_set: HashSet<i32>,
}

impl SampleDates {
    fn build(start: &Date, end: &Date, excluded: &ExcludedDates) -> Self {
        // Anchor weekly sampling to Mondays (week_day == 0)
        let mut first_monday = start.clone();
        first_monday.advance_to_weekday(0);

        let dates: Vec<Date> = if &first_monday <= end {
            first_monday
                .iterate_weekly_to(end)
                .filter(|date| !excluded.contains(date))
                .collect()
        } else {
            Vec::new()
        };
        let days_set = dates
            .iter()
            .map(|date| date.days_since_epoch() as i32)
            .collect();

        Self { dates, days_set }
    }

    fn is_empty(&self) -> bool {
        self.dates.is_empty()
    }
}

fn build_points_by_plugin(
    plugin_data: &[BorrowedPluginData],
    download_stats: &[PluginDownloadStats],
    excluded: &ExcludedDates,
) -> Vec<Vec<DownloadPoint>> {
    let mut index_by_id = HashMap::with_capacity(plugin_data.len());
    for (idx, plugin) in plugin_data.iter().enumerate() {
        index_by_id.insert(plugin.id.as_str(), idx);
    }

    let mut points_by_plugin = Vec::with_capacity(plugin_data.len());
    for _ in 0..plugin_data.len() {
        points_by_plugin.push(Vec::new());
    }

    for stats in download_stats.iter() {
        let date = stats.get_date();
        if excluded.contains(&date) {
            continue;
        }
        let day = date.days_since_epoch() as i32;

        for (id, entry) in stats.entries.iter() {
            if let Some(&idx) = index_by_id.get(id.as_str()) {
                points_by_plugin[idx].push(DownloadPoint {
                    date: date.clone(),
                    day,
                    downloads: entry.downloads,
                });
            }
        }
    }

    points_by_plugin
}

fn segment_indices(points: &[DownloadPoint]) -> Vec<(usize, usize)> {
    let mut seg_start = 0usize;
    let mut segments: Vec<(usize, usize)> = Vec::new();

    for i in 0..points.len() - 1 {
        let cur = points[i].day;
        let next = points[i + 1].day;
        if (next - cur).abs() > 7 {
            segments.push((seg_start, i));
            seg_start = i + 1;
        }
    }

    segments.push((seg_start, points.len() - 1));
    segments
}

fn update_from_single_point(
    entry: &mut BorrowedPluginData,
    point: &DownloadPoint,
    sample_days_set: &HashSet<i32>,
) {
    // Even when the only available sample is mid-week (not Monday), we still want to represent
    // that week in output as long as it is within the requested sampling range.
    let week_start = point.date.week_start();
    let week_start_day = week_start.days_since_epoch() as i32;
    if sample_days_set.contains(&week_start_day) {
        entry
            .download_history
            .0
            .insert(week_start.to_fancy_string(), point.downloads);
    }
}

fn weekly_values_for_segment(
    points: &[DownloadPoint],
    sidx: usize,
    eidx: usize,
    samples: &SampleDates,
) -> Vec<(Date, u32)> {
    // Only emit weeks that have at least one real input point.
    // This mirrors missing weeks in the output, while still allowing the spline to smooth over
    // small intra-week gaps.

    // monday_day -> (monday_date, closest_delta_days, fallback_downloads)
    let mut weeks: HashMap<i32, (Date, i32, u32)> = HashMap::new();
    for point in points[sidx..=eidx].iter() {
        let monday = point.date.week_start();
        let monday_day = monday.days_since_epoch() as i32;
        if !samples.days_set.contains(&monday_day) {
            continue;
        }

        // point is within its own week: monday_day..monday_day+6
        let delta = (point.day - monday_day).abs();
        weeks
            .entry(monday_day)
            .and_modify(|(_, best_delta, best_downloads)| {
                if delta < *best_delta
                    || (delta == *best_delta && point.downloads > *best_downloads)
                {
                    *best_delta = delta;
                    *best_downloads = point.downloads;
                }
            })
            .or_insert((monday, delta, point.downloads));
    }

    if weeks.is_empty() {
        return Vec::new();
    }

    let seg_start_day = points[sidx].day;
    let seg_end_day = points[eidx].day;

    let spline = if eidx > sidx {
        let xs: Vec<f64> = points[sidx..=eidx].iter().map(|p| p.day as f64).collect();
        let ys: Vec<f64> = points[sidx..=eidx]
            .iter()
            .map(|p| p.downloads as f64)
            .collect();
        NaturalCubicSpline::from_points(&xs, &ys)
    } else {
        None
    };

    let mut out = Vec::with_capacity(weeks.len());
    for (monday_day, (monday_date, _best_delta, fallback_downloads)) in weeks.into_iter() {
        let value = if let Some(spline) = &spline {
            if monday_day >= seg_start_day && monday_day <= seg_end_day {
                spline
                    .evaluate(monday_day as f64)
                    .filter(|v| v.is_finite() && !v.is_sign_negative())
                    .map(|v| v.round() as u32)
                    .unwrap_or(fallback_downloads)
            } else {
                // Segment begins mid-week after a gap: don't extrapolate across missing data.
                fallback_downloads
            }
        } else {
            fallback_downloads
        };

        out.push((monday_date, value));
    }

    out
}

fn update_from_segment_spline(
    entry: &mut BorrowedPluginData,
    points: &[DownloadPoint],
    sidx: usize,
    eidx: usize,
    samples: &SampleDates,
) {
    for (week_start, downloads) in
        weekly_values_for_segment(points, sidx, eidx, samples).into_iter()
    {
        entry
            .download_history
            .0
            .insert(week_start.to_fancy_string(), downloads);
    }
}

pub fn backfill_download_history(
    plugin_data: &mut [BorrowedPluginData],
    download_stats: &[PluginDownloadStats],
) {
    println!("Updating weekly download stats...");
    let end_date = Date::now();

    // Something in May 2024 is broken in source data (for example advanced-canvas).
    let excluded = ExcludedDates::new(
        Vec::new(),
        vec![DateRange::new(
            Date::new(2024, 5, 18),
            Date::new(2024, 5, 28),
        )],
    );

    let mut points_by_plugin = build_points_by_plugin(plugin_data, download_stats, &excluded);
    let total_plugins = plugin_data.len();

    for (idx, entry) in plugin_data.iter_mut().enumerate() {
        // This function owns the weekly representation; ensure we don't accumulate stale data.
        entry.download_history.0.clear();

        let sample_start = entry.added_commit.date.clone();
        let sample_end = entry
            .removed_commit
            .map_or_else(|| end_date.clone(), |c| c.date.clone());

        let points = &mut points_by_plugin[idx];
        points.retain(|point| point.date >= sample_start && point.date <= sample_end);

        if points.is_empty() {
            maybe_log_progress(total_plugins, idx);
            continue;
        }

        // Ensure deterministic order and eliminate duplicates (same day can appear multiple times
        // due to multiple commits on the same date).
        points.sort_by_key(|a| a.day);
        dedup_points_by_day_max(points);

        let samples = SampleDates::build(&sample_start, &sample_end, &excluded);
        if samples.is_empty() {
            maybe_log_progress(total_plugins, idx);
            continue;
        }

        for (sidx, eidx) in segment_indices(points).into_iter() {
            let seg_len = eidx - sidx + 1;
            if seg_len == 0 {
                continue;
            }

            if seg_len == 1 {
                update_from_single_point(entry, &points[sidx], &samples.days_set);
                continue;
            }

            update_from_segment_spline(entry, points, sidx, eidx, &samples);
        }

        maybe_log_progress(total_plugins, idx);
    }

    for entry in plugin_data.iter_mut() {
        if let Some(max_downloads) = entry.download_history.0.values().copied().max() {
            entry.download_count = max_downloads;
        }
    }
}

fn maybe_log_progress(total_plugins: usize, idx: usize) {
    if should_log_progress(idx + 1, total_plugins) {
        println!(
            "  Download spline progress: {} / {}",
            idx + 1,
            total_plugins
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_lib::date::Date;

    #[test]
    fn sample_dates_are_mondays() {
        let start = Date::new(2026, 4, 28);
        let end = Date::new(2026, 5, 31);
        let excluded = ExcludedDates::new(Vec::new(), Vec::new());
        let samples = SampleDates::build(&start, &end, &excluded);
        assert!(!samples.dates.is_empty());
        for d in samples.dates.iter() {
            assert_eq!(d.week_day(), 0);
            assert!(*d >= start);
            assert!(*d <= end);
        }
    }

    #[test]
    fn segment_indices_respects_gaps() {
        let p = vec![
            DownloadPoint {
                date: Date::new(2026, 1, 1),
                day: 0,
                downloads: 10,
            },
            DownloadPoint {
                date: Date::new(2026, 1, 6),
                day: 5,
                downloads: 20,
            },
            DownloadPoint {
                date: Date::new(2026, 1, 14),
                day: 13,
                downloads: 30,
            },
        ];
        let segs = segment_indices(&p);
        assert_eq!(segs, vec![(0usize, 1usize), (2usize, 2usize)]);
    }

    #[test]
    fn dedup_points_by_day_keeps_max_downloads() {
        let mut p = vec![
            DownloadPoint {
                date: Date::new(2026, 1, 1),
                day: 10,
                downloads: 5,
            },
            DownloadPoint {
                date: Date::new(2026, 1, 1),
                day: 10,
                downloads: 7,
            },
            DownloadPoint {
                date: Date::new(2026, 1, 2),
                day: 11,
                downloads: 3,
            },
        ];
        p.sort_by(|a, b| a.day.cmp(&b.day));
        dedup_points_by_day_max(&mut p);
        assert_eq!(p.len(), 2);
        assert_eq!(p[0].day, 10);
        assert_eq!(p[0].downloads, 7);
    }

    #[test]
    fn weekly_values_include_week_even_if_monday_before_segment_start() {
        // Segment starts mid-week (Wednesday), but we still emit the week-start Monday.
        let monday = Date::new(2026, 5, 4);
        let wed = Date::new(2026, 5, 6);
        let thu = Date::new(2026, 5, 7);

        let excluded = ExcludedDates::new(Vec::new(), Vec::new());
        let samples = SampleDates::build(&monday, &Date::new(2026, 5, 31), &excluded);

        let points = vec![
            DownloadPoint {
                date: wed.clone(),
                day: wed.days_since_epoch() as i32,
                downloads: 100,
            },
            DownloadPoint {
                date: thu.clone(),
                day: thu.days_since_epoch() as i32,
                downloads: 110,
            },
        ];

        let out = weekly_values_for_segment(&points, 0, 1, &samples);
        assert!(out.iter().any(|(d, _)| *d == monday));
    }
}
