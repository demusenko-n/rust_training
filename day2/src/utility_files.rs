use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
#[error("I/O error on {path}: {source}")]
pub struct IoErrWithPath {
    pub path: PathBuf,
    #[source]
    pub source: io::Error,
}

#[derive(Debug, Default)]
pub struct IoErrorCollector {
    pub errors: Vec<IoErrWithPath>,
}

impl IoErrorCollector {
    pub fn push_err(&mut self, path: PathBuf, source: io::Error) {
        self.errors.push(IoErrWithPath { path, source });
    }
}

#[derive(Debug, Default)]
pub struct GetFilenamesResult {
    pub files: Vec<PathBuf>,
    pub errors: IoErrorCollector,
}

fn get_all_filenames(path: &Path, max_depth: u32) -> io::Result<GetFilenamesResult> {
    let mut res = GetFilenamesResult::default();

    if !fs::symlink_metadata(path)?.file_type().is_dir() {
        res.files.push(path.to_path_buf());
    } else if max_depth > 0 {
        get_filenames_recursive(path, max_depth, &mut res)?;
    }

    Ok(res)
}

fn get_filenames_recursive(
    path: &Path,
    max_depth: u32,
    out: &mut GetFilenamesResult,
) -> io::Result<()> {
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = match entry {
            Ok(v) => v,
            Err(e) => {
                out.errors.push_err(path.to_path_buf(), e);
                continue;
            }
        };

        let entry_path = entry.path();

        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                out.errors.push_err(entry_path, e);
                continue;
            }
        };

        if ft.is_dir() {
            if max_depth > 1 {
                if let Err(e) = get_filenames_recursive(&entry_path, max_depth - 1, out) {
                    out.errors.push_err(entry_path, e);
                }
            }
        } else {
            out.files.push(entry_path);
        }
    }

    Ok(())
}

fn index_string(text: &str) -> HashMap<&str, Vec<usize>> {
    let mut map: HashMap<&str, Vec<usize>> = HashMap::new();
    let start = text.as_ptr().addr();

    for word in text.split_whitespace() {
        let offset = word.as_ptr().addr();
        map.entry(word).or_default().push(offset - start);
    }

    map
}

pub fn index_file(path: &Path) -> io::Result<HashMap<String, Vec<usize>>> {
    let file_content = fs::read_to_string(path)?;
    let index_result = index_string(&file_content);

    Ok(index_result
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect())
}

fn process_map(
    dest: &mut HashMap<String, HashMap<PathBuf, Vec<usize>>>,
    src: HashMap<String, Vec<usize>>,
    filename: PathBuf,
) {
    // Add src to dest using str as key
    for (word, v) in src {
        dest.entry(word).or_default().insert(filename.clone(), v);
    }
}

#[derive(Debug, Default)]
pub struct IndexDirectoryResult {
    pub map: HashMap<String, HashMap<PathBuf, Vec<usize>>>,
    pub errors: IoErrorCollector,
}

pub fn index_directory(path: &Path, max_depth: u32) -> io::Result<IndexDirectoryResult> {
    let filenames_res = get_all_filenames(path, max_depth)?;

    let mut result = IndexDirectoryResult {
        map: HashMap::new(),
        errors: filenames_res.errors,
    };

    for file in filenames_res.files {
        match index_file(&file) {
            Ok(v) => process_map(&mut result.map, v, file),
            Err(e) => result.errors.push_err(file, e),
        }
    }

    Ok(result)
}
