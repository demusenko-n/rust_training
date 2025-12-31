use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

use async_recursion::async_recursion;

pub async fn transmit_all_filenames_async(
    path: PathBuf,
    max_depth: u32,
    sender: tokio::sync::mpsc::Sender<PathBuf>,
) -> io::Result<()> {
    if !tokio::fs::symlink_metadata(&path)
        .await?
        .file_type()
        .is_dir()
    {
        let _ = sender.send(path).await;
    } else if max_depth > 0 {
        transmit_filenames_recursive_async(&path, max_depth, &sender).await?;
    }

    Ok(())
}

#[async_recursion]
async fn transmit_filenames_recursive_async(
    path: &Path,
    max_depth: u32,
    sender: &tokio::sync::mpsc::Sender<PathBuf>,
) -> io::Result<()> {
    let mut entries = tokio::fs::read_dir(path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        let ft = match entry.file_type().await {
            Ok(ft) => ft,
            // For simplicity - continue, but it would be better to send errors as well.
            Err(_) => continue,
        };

        if ft.is_dir() {
            if max_depth > 1 {
                // For simplicity - ignore error, but it would be better to send errors as well.
                let _ =
                    transmit_filenames_recursive_async(&entry_path, max_depth - 1, sender).await;
            }
        } else if sender.send(entry_path).await.is_err() {
            break;
        }
    }

    Ok(())
}

pub async fn index_file_async(path: &Path) -> io::Result<HashMap<String, Vec<usize>>> {
    let file_content = tokio::fs::read_to_string(path).await?;

    tokio::task::spawn_blocking(move || index_string(&file_content))
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

fn index_string(text: &str) -> HashMap<String, Vec<usize>> {
    let mut map: HashMap<String, Vec<usize>> = HashMap::new();
    let start = text.as_ptr().addr();

    for word in text.split_whitespace() {
        let offset = word.as_ptr().addr();
        map.entry(word.to_string())
            .or_default()
            .push(offset - start);
    }

    map
}
