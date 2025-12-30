use std::{
    collections::HashMap,
    fs, io,
    num::NonZero,
    path::{Path, PathBuf},
    thread::{self},
};

use crossbeam_channel as crossbeam;

use day2::{self, IoErrorCollector};

fn transmit_all_filenames(
    path: &Path,
    max_depth: u32,
    mut sender: crossbeam::Sender<PathBuf>,
) -> io::Result<day2::GetFilenamesResult> {
    let mut res = day2::GetFilenamesResult::default();

    if !fs::symlink_metadata(path)?.file_type().is_dir() {
        res.files.push(path.to_path_buf());
    } else if max_depth > 0 {
        transmit_filenames_recursive(path, max_depth, &mut sender)?;
    }

    Ok(res)
}

fn transmit_filenames_recursive(
    path: &Path,
    max_depth: u32,
    sender: &mut crossbeam::Sender<PathBuf>,
) -> io::Result<()> {
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = match entry {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("{path:?}: {e}");
                continue;
            }
        };

        let entry_path = entry.path();

        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                tracing::error!("{entry_path:?}: {e}");
                continue;
            }
        };

        if ft.is_dir() {
            if max_depth > 1 {
                if let Err(e) =
                    transmit_filenames_recursive(&entry_path, max_depth - 1, sender)
                {
                    tracing::error!("{entry_path:?}: {e}");
                }
            }
        } else {
            tracing::debug!("Detected file: {entry_path:?}");
            let _ = sender.send(entry_path);
        }
    }

    Ok(())
}

type IndexFileThreadResult = (PathBuf, Result<HashMap<String, Vec<usize>>, io::Error>);

pub fn index_directory_thr(
    path: &Path,
    max_depth: u32,
    max_threads: NonZero<usize>,
) -> io::Result<day2::IndexDirectoryResult> {
    if max_threads.get() == 1 {
        return day2::index_directory(path, max_depth);
    }

    thread::scope(|s| -> io::Result<day2::IndexDirectoryResult> {
        let (task_tx, task_rx) = crossbeam::unbounded::<PathBuf>();
        let (res_tx, res_rx) = crossbeam::unbounded::<IndexFileThreadResult>();

        let workers_num = max_threads.get() - 1;
        tracing::debug!("Spawning {workers_num} threads");
        let mut handles = Vec::with_capacity(workers_num);

        for _ in 0..workers_num {
            let task_rx = task_rx.clone();
            let res_tx = res_tx.clone();

            handles.push(s.spawn(move || {
                for filename in task_rx {
                    tracing::debug!(
                        "Thread #{:?} started processing {filename:?}",
                        std::thread::current().id()
                    );
                    let result = day2::index_file(&filename);
                    let _ = res_tx.send((filename, result));
                }
            }));
        }
        drop(res_tx);
        drop(task_rx);

        transmit_all_filenames(path, max_depth, task_tx)?;

        let mut result = day2::IndexDirectoryResult {
            map: HashMap::new(),
            errors: IoErrorCollector::default(),
        };

        for (filename, res) in res_rx {
            match res {
                Ok(res) => day2::process_map(&mut result.map, res, filename),
                Err(e) => result.errors.push_err(filename, e),
            }
        }

        Ok(result)
    })
}
