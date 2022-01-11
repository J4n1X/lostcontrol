const CURRENT_CONFIG_VERSION: &str = "0.0.5";

mod commit;
mod repo;
mod branch_config;

pub use self::commit::Commit;
pub use self::repo::Repo;
pub use self::branch_config::BranchConfig;

macro_rules! dprintln {
    ($($arg:tt)*) => (
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    )
}

pub(crate) use dprintln;

// TODO: These functions need to be rewritten to support a more generic path format
pub mod fs_operations {
    use std::path::PathBuf;

    pub fn grab_directories(path: &PathBuf) -> Result<Vec<PathBuf>,()> {
        if path.exists() && path.is_dir() {
            let entries = std::fs::read_dir(path).unwrap();
            let mut files: Vec<PathBuf> = Vec::new();
            for entry in entries {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    files.push(path);
                }
            }
            return Ok(files);
        }
        Err(())
    }

    pub fn grab_files(path: &PathBuf) -> Result<Vec<PathBuf>, ()> {
        if path.exists() && path.is_dir() {
            let entries = std::fs::read_dir(path).unwrap();
            let mut files: Vec<PathBuf> = Vec::new();
            for entry in entries {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                }
            }
            return Ok(files);
        }
        Err(())
    }
    
    pub fn expand_directory(path: &PathBuf, ignored_dirs: &Vec<String>) -> Vec<PathBuf> {
        // get all files in directory recursively
        let mut files: Vec<PathBuf> = Vec::new();
        let mut directories: Vec<PathBuf> = vec![path.to_path_buf()];
        while directories.len() > 0 {
            let current_directory = directories.pop().unwrap();
            let current_directory_string = current_directory.to_str().unwrap().to_string();
            if ignored_dirs.contains(&current_directory_string) {
                dprintln!("[INFO] Directory {} is on ignore list, skipping...", &current_directory_string);
                continue;
            }
    
            let mut child_dirs = grab_directories(&current_directory).unwrap();
            directories.append(&mut child_dirs);
    
            let child_files = grab_files(&current_directory).unwrap();
            for file in child_files {
                //let file_string = file.to_str().unwrap().to_string();
                files.push(file);
            }
        }
        files
    }
}