use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

use crate::utility_async;
use day2::{self, IoErrorCollector};

type IndexFileResult = (PathBuf, io::Result<HashMap<String, Vec<usize>>>);

pub async fn index_directory_async(
    path: &Path,
    max_depth: u32,
) -> anyhow::Result<day2::IndexDirectoryResult> {
    // Errors are ignored for simplicity
    let (task_tx, mut task_rx) = tokio::sync::mpsc::channel::<PathBuf>(10);
    let (res_tx, mut res_rx) = tokio::sync::mpsc::channel::<IndexFileResult>(10);

    // As a separate task, search for filenames. [file searching task]
    // Finds all filenames and populates them through the task_tx
    let path_owned = path.to_path_buf();
    let task_filenames = tokio::spawn(utility_async::transmit_all_filenames_async(
        path_owned, max_depth, task_tx,
    ));

    // As a separate task, listen for task_rx [index starting task]
    // When new filename is received, schedule task of indexing.
    // Task of indexing should populate the result through res_tx
    let task_indexing_scheduling = tokio::spawn(async move {
        while let Some(value) = task_rx.recv().await {
            let tx = res_tx.clone();
            tokio::spawn(async move {
                let index_res = utility_async::index_file_async(&value).await;
                let _ = tx.send((value, index_res)).await;
            });
        }

        // We could track all created tasks. Maybe, manage max amount of concurrent tasks?
        // Also, we can call .join here for those tasks.
        // But we are listening for the channel until all senders die, so it's practically the same, works similarly to join.
    });

    let mut result = day2::IndexDirectoryResult {
        map: HashMap::new(),
        errors: IoErrorCollector::default(),
    };

    // As a separate task, listen for res_rx. [result collecting task]
    // ### We could create a task and await for it in main, but main has nothing to do, so just do it in main?
    // When new result is received, merge the map. Map belongs to that task. When finished listening, return the map.
    // main awaits for result collecting task

    while let Some((path, index_res)) = res_rx.recv().await {
        match index_res {
            // day2::process_map can be optimized better, but not worth it for current task, I think
            Ok(index_res) => day2::process_map(&mut result.map, index_res, path),
            Err(err) => result.errors.push_err(path, err),
        }
    }

    task_filenames.await??;
    task_indexing_scheduling.await?;

    Ok(result)
}
