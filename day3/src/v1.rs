use std::{
    collections::HashMap,
    io,
    num::NonZero,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    thread::{self},
};

use day2;

type IndexFileThreadResult = Vec<(PathBuf, Result<HashMap<String, Vec<usize>>, io::Error>)>;

pub fn index_directory_thr(
    path: &Path,
    max_depth: u32,
    max_threads: NonZero<usize>,
) -> io::Result<day2::IndexDirectoryResult> {
    let filenames_res = day2::get_all_filenames(path, max_depth)?;
    let filenames = filenames_res.files;

    if max_threads.get() <= 2 {
        return day2::index_directory(path, max_depth);
    }

    let mut result = day2::IndexDirectoryResult {
        map: HashMap::new(),
        errors: filenames_res.errors,
    };

    let next_task_index = AtomicUsize::new(0);

    thread::scope(|s| {
        let spawned_threads = (max_threads.get() - 1).min(filenames.len());
        tracing::debug!("Spawning {spawned_threads} threads");
        let mut handles = Vec::with_capacity(spawned_threads);

        for _ in 0..spawned_threads {
            let filenames = &filenames;
            let next_task_index = &next_task_index;

            handles.push(s.spawn(move || {
                tracing::debug!("Thread #{:?} started", std::thread::current().id());
                let mut thread_results: IndexFileThreadResult = Vec::new();

                loop {
                    let i = next_task_index.fetch_add(1, Ordering::Relaxed);
                    if i >= filenames.len() {
                        tracing::debug!(
                            "No tasks left for thread #{:?}",
                            std::thread::current().id()
                        );
                        break;
                    }
                    let filename = &filenames[i];

                    tracing::debug!(
                        "Thread #{:?} starts reading file {:?}",
                        std::thread::current().id(),
                        &filename
                    );
                    thread_results.push((filename.to_path_buf(), day2::index_file(&filename)))
                }

                thread_results
            }));
        }

        for h in handles {
            let thread_results = h.join().unwrap();

            for (filename, res) in thread_results {
                match res {
                    Ok(res) => day2::process_map(&mut result.map, res, filename),
                    Err(e) => result.errors.push_err(filename, e),
                }
            }
        }
    });

    Ok(result)
}
