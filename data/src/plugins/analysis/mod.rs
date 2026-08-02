use std::{
    collections::{BTreeMap, VecDeque},
    path::Path,
    sync::{
        Arc, Condvar, Mutex,
        mpsc::{Receiver, sync_channel},
    },
};

use data_lib::plugin::{PluginData, PluginExtraData};
use hashbrown::HashMap;
use rayon::ThreadPoolBuilder;

use self::{
    pipeline::analyze_plugin,
    repo_analysis::{read_plugin_version_deprecations, read_removed_plugins},
    run_stats::{ExtraPluginResult, ExtraRunStats},
};

use crate::{
    constants::{PLUGIN_RELEASE_ENRICHMENT_STATE_PATH, PLUGIN_REPO_DATA_PATH},
    file_utils::{DATA_CHUNK_SIZE, write_chunks_atomic_results},
    plugins::{
        data::read_plugin_data_chunks, license::license_compare::LicenseComparer,
        release_acquisition::PluginReleaseState, stats_helper::HelperPluginStore,
    },
    state::read_json_or_default,
};

mod mainjs;
mod output;
mod pipeline;
mod repo;
mod repo_analysis;
mod run_stats;
mod types;

const EXTRA_ANALYSIS_THREADS_ENV: &str = "EXTRA_ANALYSIS_THREADS";
const ANALYSIS_QUEUE_LIMIT: usize = 200;

pub fn extract_analysis_data() -> Result<(), Box<dyn std::error::Error>> {
    let removed_plugins = read_removed_plugins()?;
    let removed_reason_by_id = removed_plugins
        .into_iter()
        .map(|entry| (entry.id, entry.reason))
        .collect::<HashMap<_, _>>();

    let deprecated_versions_by_plugin = read_plugin_version_deprecations()?;

    let release_state: PluginReleaseState =
        read_json_or_default(Path::new(PLUGIN_RELEASE_ENRICHMENT_STATE_PATH));
    let helper_store = HelperPluginStore::read()?;

    let mut license_comparer = LicenseComparer::new();
    license_comparer.init();

    let default_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let thread_count = configured_thread_count(EXTRA_ANALYSIS_THREADS_ENV, default_threads);

    println!(
        "Extra data: processing plugins (analysis phase, threads: {}, queue limit: {})",
        thread_count, ANALYSIS_QUEUE_LIMIT
    );

    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .expect("Failed to build extra analysis thread pool");

    let job_queue = Arc::new(BoundedJobQueue::new(ANALYSIS_QUEUE_LIMIT));
    let (result_tx, result_rx) = sync_channel(ANALYSIS_QUEUE_LIMIT);
    let producer_error = Arc::new(Mutex::new(None));

    let producer_queue = Arc::clone(&job_queue);
    let producer_error_for_thread = Arc::clone(&producer_error);
    let producer = std::thread::spawn(move || {
        let result = produce_analysis_jobs(Arc::clone(&producer_queue), read_plugin_data_chunks());
        finish_producer(&producer_queue, &producer_error_for_thread, result);
    });

    let mut run_stats = ExtraRunStats::default();
    let removed_reason_by_id_ref = &removed_reason_by_id;
    let deprecated_versions_by_plugin_ref = &deprecated_versions_by_plugin.0;
    let license_comparer_ref = &license_comparer;
    let release_state_ref = &release_state;
    let helper_store_ref = &helper_store;
    let worker_result = thread_pool.scope(|scope| {
        for _ in 0..thread_count {
            let worker_queue = Arc::clone(&job_queue);
            let worker_result_tx = result_tx.clone();
            scope.spawn(move |_| {
                loop {
                    let Some(job) = worker_queue.pop() else {
                        break;
                    };
                    let result = analyze_job(
                        job,
                        removed_reason_by_id_ref,
                        deprecated_versions_by_plugin_ref,
                        license_comparer_ref,
                        release_state_ref,
                        helper_store_ref,
                    );
                    if worker_result_tx.send(result).is_err() {
                        worker_queue.close();
                        break;
                    }
                }
            });
        }
        drop(result_tx);

        let output = AnalysisOutput::new(result_rx, producer_error, &mut run_stats);
        write_chunks_atomic_results(Path::new(PLUGIN_REPO_DATA_PATH), output)
            .map_err(|error| error.to_string())
    });

    let producer_result = producer.join().map_err(|_| {
        Box::new(std::io::Error::other(
            "analysis job producer thread panicked",
        )) as Box<dyn std::error::Error>
    });
    if let Err(error) = worker_result {
        return Err(Box::new(std::io::Error::other(error)));
    }
    producer_result?;

    println!("Extra data summary:");
    println!("  Removed plugins skipped: {}", run_stats.removed_skipped);
    println!(
        "  Repo extraction failures: {}",
        run_stats.repo_extract_failed
    );
    println!(
        "  Missing release acquisition state: {}",
        run_stats.release_state_missing
    );
    println!(
        "  Release main.js scans (success): {}",
        run_stats.release_main_js_scanned
    );
    println!(
        "  Release main.js scans (failed/skip): {}",
        run_stats.release_main_js_scan_failed
    );

    Ok(())
}

fn configured_thread_count(env_var: &str, default_threads: usize) -> usize {
    std::env::var(env_var)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|count| *count > 0)
        .unwrap_or(default_threads)
}

struct AnalysisJob {
    chunk_id: usize,
    plugin_index: usize,
    chunk_len: usize,
    plugin: PluginData,
}

struct AnalysisResult {
    chunk_id: usize,
    plugin_index: usize,
    chunk_len: usize,
    result: ExtraPluginResult,
}

struct BoundedJobQueue {
    state: Mutex<JobQueueState>,
    not_empty: Condvar,
    not_full: Condvar,
    limit: usize,
}

struct JobQueueState {
    jobs: VecDeque<AnalysisJob>,
    closed: bool,
}

impl BoundedJobQueue {
    fn new(limit: usize) -> Self {
        Self {
            state: Mutex::new(JobQueueState {
                jobs: VecDeque::with_capacity(limit),
                closed: false,
            }),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            limit,
        }
    }

    fn wait_for_capacity(&self, requested: usize) -> bool {
        let mut state = self.state.lock().unwrap();
        while !state.closed && state.jobs.len() + requested > self.limit {
            state = self.not_full.wait(state).unwrap();
        }
        !state.closed
    }

    fn push(&self, jobs: Vec<AnalysisJob>) -> bool {
        let mut state = self.state.lock().unwrap();
        if state.closed || state.jobs.len() + jobs.len() > self.limit {
            return false;
        }
        state.jobs.extend(jobs);
        self.not_empty.notify_all();
        true
    }

    fn pop(&self) -> Option<AnalysisJob> {
        let mut state = self.state.lock().unwrap();
        while state.jobs.is_empty() && !state.closed {
            state = self.not_empty.wait(state).unwrap();
        }
        let job = state.jobs.pop_front();
        if job.is_some() {
            self.not_full.notify_one();
        }
        job
    }

    fn close(&self) {
        let mut state = self.state.lock().unwrap();
        state.closed = true;
        self.not_empty.notify_all();
        self.not_full.notify_all();
    }
}

fn produce_analysis_jobs(
    queue: Arc<BoundedJobQueue>,
    chunks: Result<crate::file_utils::ChunkReader<PluginData>, Box<dyn std::error::Error>>,
) -> Result<(), String> {
    let result = (|| {
        let mut chunks = chunks.map_err(|error| error.to_string())?;
        let mut chunk_id = 0;

        loop {
            if !queue.wait_for_capacity(DATA_CHUNK_SIZE) {
                return Ok(());
            }

            let Some(chunk) = chunks.next() else {
                break;
            };
            let chunk = chunk.map_err(|error| error.to_string())?;
            if chunk.is_empty() {
                continue;
            }

            let chunk_len = chunk.len();
            let jobs = chunk
                .into_iter()
                .enumerate()
                .map(|(plugin_index, plugin)| AnalysisJob {
                    chunk_id,
                    plugin_index,
                    chunk_len,
                    plugin,
                })
                .collect();
            if !queue.push(jobs) {
                return Ok(());
            }
            chunk_id += 1;
        }

        Ok(())
    })();
    result
}

fn finish_producer(
    queue: &BoundedJobQueue,
    producer_error: &Mutex<Option<String>>,
    result: Result<(), String>,
) {
    if let Err(error) = result {
        *producer_error.lock().unwrap() = Some(error);
    }
    // Workers use queue closure as the signal to finish, and the output side
    // may see the result channel disconnect as soon as the workers exit.
    queue.close();
}

fn analyze_job(
    job: AnalysisJob,
    removed_reason_by_id: &HashMap<String, String>,
    deprecated_versions_by_plugin: &HashMap<String, Vec<String>>,
    license_comparer: &LicenseComparer,
    release_state: &PluginReleaseState,
    helper_store: &HelperPluginStore,
) -> AnalysisResult {
    let plugin = job.plugin;
    let removal_reason = removed_reason_by_id.get(&plugin.id).cloned();
    let deprecated_versions = deprecated_versions_by_plugin
        .get(&plugin.id)
        .cloned()
        .unwrap_or_default();

    let mut stats = ExtraRunStats::default();
    let repo = if plugin.removed_commit.is_none() {
        match analyze_plugin(
            &plugin,
            license_comparer,
            release_state,
            helper_store,
            &mut stats,
        ) {
            Ok(repo_data) => Ok(repo_data),
            Err(err) => {
                stats.repo_extract_failed += 1;
                println!("Failed to analyze plugin {}: {}", plugin.id, err);
                Err(err)
            }
        }
    } else {
        stats.removed_skipped += 1;
        Err(format!(
            "Plugin {} was removed, skipping repository extraction",
            plugin.id
        ))
    };

    AnalysisResult {
        chunk_id: job.chunk_id,
        plugin_index: job.plugin_index,
        chunk_len: job.chunk_len,
        result: ExtraPluginResult {
            data: PluginExtraData {
                id: plugin.id,
                repo,
                removal_reason,
                deprecated_versions,
            },
            stats,
        },
    }
}

struct AnalysisOutput<'a> {
    receiver: Receiver<AnalysisResult>,
    producer_error: Arc<Mutex<Option<String>>>,
    pending: BTreeMap<(usize, usize), AnalysisResult>,
    next_chunk_id: usize,
    next_plugin_index: usize,
    output_buffer: Vec<PluginExtraData>,
    run_stats: &'a mut ExtraRunStats,
    processed: usize,
    finished: bool,
}

impl<'a> AnalysisOutput<'a> {
    fn new(
        receiver: Receiver<AnalysisResult>,
        producer_error: Arc<Mutex<Option<String>>>,
        run_stats: &'a mut ExtraRunStats,
    ) -> Self {
        Self {
            receiver,
            producer_error,
            pending: BTreeMap::new(),
            next_chunk_id: 0,
            next_plugin_index: 0,
            output_buffer: Vec::with_capacity(DATA_CHUNK_SIZE),
            run_stats,
            processed: 0,
            finished: false,
        }
    }

    fn absorb_ready_results(&mut self) {
        while self.output_buffer.len() < DATA_CHUNK_SIZE {
            let Some(result) = self
                .pending
                .remove(&(self.next_chunk_id, self.next_plugin_index))
            else {
                break;
            };
            self.run_stats.merge(result.result.stats);
            self.output_buffer.push(result.result.data);
            self.processed += 1;
            if self.processed.is_multiple_of(DATA_CHUNK_SIZE) {
                println!("  Processed {} plugins", self.processed);
            }

            if self.next_plugin_index + 1 == result.chunk_len {
                self.next_chunk_id += 1;
                self.next_plugin_index = 0;
            } else {
                self.next_plugin_index += 1;
            }
        }
    }

    fn stream_error(&self, message: impl Into<String>) -> Box<dyn std::error::Error> {
        Box::new(std::io::Error::other(message.into()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex, mpsc::channel};

    use super::{AnalysisOutput, AnalysisResult, BoundedJobQueue, finish_producer};
    use crate::plugins::analysis::run_stats::{ExtraPluginResult, ExtraRunStats};
    use data_lib::plugin::PluginExtraData;

    #[test]
    fn output_emits_fixed_size_chunks_in_input_order() {
        let (sender, receiver) = channel();
        for plugin_index in (0..51).rev() {
            sender
                .send(AnalysisResult {
                    chunk_id: 0,
                    plugin_index,
                    chunk_len: 51,
                    result: ExtraPluginResult {
                        data: PluginExtraData {
                            id: plugin_index.to_string(),
                            repo: Err("test".to_string()),
                            removal_reason: None,
                            deprecated_versions: Vec::new(),
                        },
                        stats: ExtraRunStats::default(),
                    },
                })
                .unwrap();
        }
        drop(sender);

        let mut stats = ExtraRunStats::default();
        let mut output = AnalysisOutput::new(receiver, Arc::new(Mutex::new(None)), &mut stats);

        let first = output.next().unwrap().unwrap();
        assert_eq!(first.len(), 50);
        assert_eq!(first[0].id, "0");
        assert_eq!(first[49].id, "49");

        let second = output.next().unwrap().unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].id, "50");
        assert!(output.next().is_none());
    }

    #[test]
    fn producer_error_is_reported_when_result_channel_disconnects() {
        let (sender, receiver) = channel();
        drop(sender);

        let queue = BoundedJobQueue::new(1);
        let producer_error = Arc::new(Mutex::new(None));
        finish_producer(
            &queue,
            &producer_error,
            Err("failed to read the next plugin chunk".to_string()),
        );

        let mut stats = ExtraRunStats::default();
        let mut output = AnalysisOutput::new(receiver, producer_error, &mut stats);
        assert!(output.next().is_some_and(|result| result.is_err()));
        assert!(output.next().is_none());
    }
}

impl Iterator for AnalysisOutput<'_> {
    type Item = Result<Vec<PluginExtraData>, Box<dyn std::error::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            self.absorb_ready_results();
            if self.output_buffer.len() == DATA_CHUNK_SIZE {
                return Some(Ok(std::mem::take(&mut self.output_buffer)));
            }

            match self.receiver.recv() {
                Ok(result) => {
                    self.pending
                        .insert((result.chunk_id, result.plugin_index), result);
                }
                Err(_) => {
                    self.finished = true;
                    if let Some(error) = self.producer_error.lock().unwrap().clone() {
                        return Some(Err(self.stream_error(error)));
                    }
                    if !self.pending.is_empty() {
                        return Some(Err(self.stream_error(
                            "analysis result stream ended before the next ordered plugin",
                        )));
                    }
                    if self.output_buffer.is_empty() {
                        return None;
                    }
                    return Some(Ok(std::mem::take(&mut self.output_buffer)));
                }
            }
        }
    }
}
