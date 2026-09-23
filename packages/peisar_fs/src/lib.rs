//! Filesystem helpers rooted at a configurable working directory.

use peisar_log::{error, info};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

/// Provides filesystem operations relative to a configured root directory.
pub struct PeisarFs {
    cwd: PathBuf,
}

impl PeisarFs {
    /// Creates a filesystem helper rooted at `root`.
    ///
    /// When `root` is `None`, the process's current directory is used. If the
    /// current directory cannot be read, the helper falls back to `"."`.
    pub fn new(root: Option<PathBuf>) -> Self {
        Self {
            cwd: if root.is_none() {
                env::current_dir().unwrap_or(PathBuf::from("."))
            } else {
                root.unwrap_or(PathBuf::from("."))
            },
        }
    }

    /// Returns whether `target_path` exists below the configured root.
    pub fn exixts<P: AsRef<Path>>(&self, target_path: P) -> bool {
        let final_target_path = self.cwd.join(target_path.as_ref());
        let r = fs::exists(final_target_path);
        if r.is_err() {
            let message = format!("Error when checking target {:?}", target_path.as_ref());
            error(&message, true);
        }
        r.unwrap()
    }

    /// Creates `dir_path` and all missing parent directories.
    ///
    /// If the directory already exists, no filesystem operation is performed.
    pub fn mkdir<P: AsRef<Path>>(&self, dir_path: P) {
        let final_dir_path = self.cwd.join(dir_path.as_ref());
        if self.exixts(dir_path.as_ref()) {
            let message = format!(
                "Directory {:?} already exists, nothing to create.",
                dir_path.as_ref()
            );
            info(&message);
        } else {
            fs::create_dir_all(final_dir_path).ok();
        }
    }

    /// Writes `content` to `file_path`, creating missing parent directories.
    pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(&self, file_path: P, content: C) {
        let final_file_path = self.cwd.join(file_path);
        if let Some(parent) = final_file_path.parent() {
            if !fs::exists(parent).unwrap() {
                self.mkdir(parent);
            }
        }
        fs::write(final_file_path, content).ok();
    }

    /// Reads `file_path` as UTF-8 text.
    ///
    /// This method reports a missing or unreadable file through `peisar_log`
    /// and exits the process when an error occurs.
    pub fn read_file<P: AsRef<Path>>(&self, file_path: P) -> String {
        let final_file_path = self.cwd.join(file_path.as_ref());
        if !fs::exists(final_file_path.clone()).unwrap() {
            let message = format!("File {:?} dose not exists.", file_path.as_ref());
            error(&message, true);
        }
        let contents = fs::read_to_string(final_file_path);
        if contents.is_err() {
            let message = format!("Error when reading file {:?}", file_path.as_ref());
            error(&message, true);
        }
        contents.unwrap()
    }

    /// Recursively lists files below `dir_path`.
    ///
    /// When `ext` is `Some`, only files with that exact extension are
    /// returned. When it is `None`, all files are returned. Returned paths are
    /// rooted at the configured directory.
    pub fn read_dir<P: AsRef<Path>>(&self, dir_path: P, ext: Option<&str>) -> Vec<PathBuf> {
        let mut entries: Vec<PathBuf> = Vec::new();
        let final_dir_path = self.cwd.join(dir_path.as_ref());
        if let Err(err) = find_files_by_extension(&final_dir_path, &mut entries, ext) {
            let message = format!(
                "Error when reading directory {:?}: {}",
                dir_path.as_ref(),
                err
            );
            error(&message, true);
        }
        entries
    }
}

/// Recursively searches a directory for files matching a specific extension.
fn find_files_by_extension(
    dir: &Path,
    results: &mut Vec<PathBuf>,
    ext: Option<&str>,
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Proactively recurse into subdirectories
                find_files_by_extension(&path, results, ext)?;
            } else if path.is_file() {
                // Check if the file extension matches the parameter
                if let Some(file_ext) = path.extension() {
                    if ext.is_none() {
                        results.push(path);
                    } else {
                        if file_ext == ext.unwrap() {
                            results.push(path);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::PeisarFs;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temporary_root() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("peisar_fs_test_{}_{}", std::process::id(), suffix))
    }

    #[test]
    fn creates_directories_and_reads_and_writes_files() {
        let root = temporary_root();
        let filesystem = PeisarFs::new(Some(root.clone()));

        filesystem.mkdir("nested/dir");
        assert!(filesystem.exixts("nested/dir"));

        filesystem.write_file("nested/dir/file.txt", "hello");
        assert!(filesystem.exixts("nested/dir/file.txt"));
        assert_eq!(filesystem.read_file("nested/dir/file.txt"), "hello");

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recursively_lists_all_files_or_only_matching_extensions() {
        let root = temporary_root();
        let filesystem = PeisarFs::new(Some(root.clone()));

        filesystem.write_file("one.txt", "one");
        filesystem.write_file("nested/two.rs", "two");
        filesystem.write_file("nested/three.txt", "three");

        let mut all_files = filesystem.read_dir(".", None);
        all_files.sort();
        assert_eq!(all_files.len(), 3);

        let mut text_files = filesystem.read_dir(".", Some("txt"));
        text_files.sort();
        assert_eq!(
            text_files,
            vec![root.join("nested/three.txt"), root.join("one.txt")]
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn returns_an_empty_list_for_a_missing_directory() {
        let root = temporary_root();
        let filesystem = PeisarFs::new(Some(root.clone()));

        assert!(filesystem.read_dir("missing", None).is_empty());

        fs::remove_dir_all(root).unwrap_or(());
    }
}
