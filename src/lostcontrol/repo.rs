use std::path::PathBuf;
use std::io::{BufReader, BufRead, Write};
use serde::{self, Deserialize, Serialize};
use serde_yaml::{self};

use crate::lostcontrol::{BranchConfig, Commit, CURRENT_CONFIG_VERSION, dprintln};
use crate::lostcontrol::fs_operations::*;

const DEFAULT_BRANCH: &str = "master";
const DEFAULT_CONFIG_FILE: &str = ".lostcontrol.conf";
const DEFAULT_REPOS_DIR: &str = ".lostcontrol";


#[derive(Debug, Serialize, Deserialize)]
pub struct Repo {
    #[serde(skip)]
    metafile_path: PathBuf,
    #[serde(skip)]
    repos_dir: PathBuf,
    #[serde(skip)]
    closed: bool,
    #[serde(skip)]
    modified: bool,
    pub name: String, 
    pub current_branch: String,
    pub branches: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub ignored_files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub ignored_dirs: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub staged_files: Vec<String>,
}

// A repository struct should store the repo name, the current branch and the available branches
impl Repo {
    fn filter_paths(&mut self, entries: &Vec<PathBuf>) -> Vec<String> {
        let mut files: Vec<String> = vec![];
        for entry in entries.iter() {
            if entry.is_file() {
                files.push(entry.to_str().unwrap().to_string());
            }
            else if entry.is_dir(){
                let dir_entries = expand_directory(entry, &self.ignored_dirs);
                for dir_entry in dir_entries.iter() {
                    files.push(dir_entry.to_str().unwrap().to_string());
                }
            }
            else {
                dprintln!("[WARN] File {} is not a file or directory, ignoring it for now", entry.to_str().unwrap());
            }
        }
        files
    }

    fn format_branch_dir(&self, branch: &BranchConfig, id: usize) -> String {
        format!("{}-commit-{}", branch.name, id)
    }

    pub fn new(name: &String, root_path: &PathBuf) -> Result<Repo, ()> {
        // write a new metafile
        let mf_path = root_path.join(DEFAULT_CONFIG_FILE);
        let mf_repos_dir = root_path.join(DEFAULT_REPOS_DIR);

        if mf_path.exists() {
            return Err(());
        }
        if mf_repos_dir.exists()  {
            return Err(());
        }

        let config = Repo {
            metafile_path: mf_path.clone(),
            repos_dir: mf_repos_dir.clone(),
            closed: false,
            modified: true,
            name: name.to_string(),
            current_branch: String::from(DEFAULT_BRANCH),
            branches: vec![String::from(DEFAULT_BRANCH)],
            ignored_files: vec![
                format!("./{}", mf_path.to_str().unwrap())
            ],
            ignored_dirs: vec![
                format!("./{}",mf_repos_dir.to_str().unwrap())
            ],
            staged_files: vec![],
        };
        dprintln!("[INFO] Metafile {} created!", DEFAULT_CONFIG_FILE);
        Ok(config)
    }

    pub fn from_file(dir: Option<&String>) -> Result<Repo, ()> {
        // if no directory is specified, use the current directory
        let mf_path_base = match dir {
            Some(dir) => std::fs::canonicalize(PathBuf::from(dir)).unwrap(),
            None => std::env::current_dir().unwrap(),
        };
        let mf_path = mf_path_base.join(DEFAULT_CONFIG_FILE);
        let mf_repos_dir = mf_path_base.join(DEFAULT_REPOS_DIR);

        if !mf_path.exists() {
            return Err(());
        }

        let mut metafile = match std::fs::File::open(&mf_path) {
            Ok(file) => file,
            Err(e) => panic!("Cannot load metafile {}: {}", DEFAULT_CONFIG_FILE, e)
        };
        let mut version = String::new();
        let mut contents = String::new();

        let buf_reader = BufReader::new(&mut metafile);
        for (index, line) in buf_reader.lines().enumerate() {
            match index {
                0 => version = line.unwrap(),
                _ => { 
                    let line_string: String = String::from(line.unwrap()) + "\n";
                    contents.push_str(&line_string.as_str());
                }
            }
        }

        if version != CURRENT_CONFIG_VERSION {
            dprintln!("[ERROR] Metafile version {} is not supported!", version);
            return Err(());
        }

        let mut config: Repo = serde_yaml::from_str(&contents).unwrap();
        config.metafile_path = mf_path.clone();
        config.repos_dir = mf_repos_dir.clone();
        dprintln!("[INFO] Metafile for repository {} loaded!", config.name);
        Ok(config)
    }

    pub fn stage_files(&mut self, entries: &Vec<PathBuf>){
        if self.closed {
            return;
        }

        let files: Vec<String> = self.filter_paths(entries);
        for file in files.iter() {
            if !self.staged_files.contains(&file) {
                if !self.ignored_files.contains(&file) {
                    self.staged_files.push(file.clone());
                    dprintln!("[INFO] File {} staged!", file);
                }
                else {
                    dprintln!("[INFO] File {} is on ignore list, skipping...", file);
                }
            }
            else {
                dprintln!("[INFO] File {} is already staged, skipping...", file);
            }
        }
        self.modified = true;
    }

    pub fn unstage_files(&mut self, entries: &Vec<PathBuf>){
        let files: Vec<String> = self.filter_paths(entries);
        self.staged_files.retain(|x| !files.contains(x));
        for file in files.iter() {
            dprintln!("[INFO] File {} unstaged!", file);
        }
        self.modified = true;
    }

    pub fn unstage_all(&mut self){
        self.staged_files.clear();
        self.modified = true;
    }

    pub fn commit(&mut self, commit_msg: String) -> Result<usize, ()>{
        if self.closed {
            dprintln!("[WARN] Repository {} is closed, skipping commit!", self.name);
            return Err(());
        }

        if self.staged_files.is_empty() {
            dprintln!("[WARN] No staged files, skipping commit!");
            return Err(());
        }

        // create a config path with this pattern: DEFAULT_REPOS_DIR/<current_branch>/<current_branch>.conf
        let staged_files_count = self.staged_files.len();
        let branch_path = self.repos_dir.clone().join(self.current_branch.clone());

        let mut branch_config = self.get_branch(&self.current_branch).unwrap();

        let commit = Commit::new(
            branch_config.commit_count() + 1 as usize, 
            commit_msg, 
            self.staged_files.clone()
        );

        // now we create the commit archive file
        let commit_path = branch_path.join(self.format_branch_dir(&branch_config, commit.id));
        
        dprintln!("[INFO] Writing staged files to commit directory {}...", commit_path.display());
        match std::fs::create_dir(&commit_path) {
            Ok(_) => {},
            Err(e) => {
                dprintln!("[ERROR] Cannot create commit directory {}: {}", commit_path.display(), e);
                return Err(());
            }
        };
        for file in self.staged_files.iter() {
            let staged_file_path = PathBuf::from(&file);
            let commit_file_path = commit_path.join(&staged_file_path);

            dprintln!("[INFO] Copying staged file {} to commit directory {}...", staged_file_path.display(), commit_file_path.display());
            if !commit_file_path.parent().unwrap().exists() {
                match std::fs::create_dir_all(commit_file_path.parent().unwrap()) {
                    Ok(_) => {},
                    Err(e) => {
                        dprintln!("[ERROR] Cannot create commit directory {}: {}", commit_file_path.parent().unwrap().display(), e);
                        return Err(());
                    }
                }
            }
            match std::fs::copy(&staged_file_path, &commit_file_path) {
                Ok(_) => {},
                Err(e) => {
                    dprintln!("[ERROR] Failed to copy file to commit directory {}: {}", &staged_file_path.display(), e);
                    dprintln!("[ERROR] Aborting commit!");
                    return Err(());
                }
            }
        }

        branch_config.push_commit(commit);
        self.staged_files.clear();
        self.modified = true;
        Ok(staged_files_count)
    }

    pub fn remove_commit(&mut self, commit_id: usize) -> Result<(), ()>{
        if self.closed {
            dprintln!("[WARN] Repository {} is closed, skipping commit!", self.name);
            return Err(());
        }

        let branch_path = self.repos_dir.clone().join(self.current_branch.clone());
        let mut branch_config = self.get_branch(&self.current_branch).unwrap();

        // remove commit directory
        let commit_path = branch_path.join(self.format_branch_dir(&branch_config, commit_id));
        dprintln!("[INFO] Removing commit directory {}...", commit_path.display());
        match std::fs::remove_dir_all(&commit_path) {
            Ok(_) => {},
            Err(e) => {
                dprintln!("[ERROR] Cannot remove commit directory {}: {}", commit_path.display(), e);
                return Err(());
            }
        };

        match branch_config.remove_commit(commit_id) {
            Ok(_) => {
                self.modified = true;
                Ok(())
            },
            Err(()) => Err(())
        }
    }

    // TODO: Remove files that aren't part of the commit
    // TODO: Check if commit id is valid
    pub fn restore_commit(&self, commit_id: usize) -> Result<(), ()>{
        if self.closed {
            dprintln!("[WARN] Repository {} is closed, skipping close!", self.name);
            return Err(());
        }

        let restore_path = std::fs::canonicalize(std::env::current_dir().unwrap()).unwrap();
        let branch_path = self.repos_dir.clone().join(self.current_branch.clone());
        let commit_path = branch_path.join(self.format_branch_dir(&self.get_branch(&self.current_branch).unwrap(), commit_id));
        dprintln!("[INFO] Restoring commit directory {}...", commit_path.display());
        let commit_files = expand_directory(&commit_path, &Vec::new());
        let restore_dir_files = expand_directory(&restore_path, &self.ignored_files);
        /*
        for rest_file in restore_dir_files.iter() {
            if !commit_files.contains(rest_file) && !self.ignored_files.contains(&rest_file.to_str().unwrap().to_string()) {
                dprintln!("[INFO] Removing file {} from restore directory...", rest_file.display());
                match std::fs::remove_file(rest_file) {
                    Ok(_) => {},
                    Err(e) => {
                        dprintln!("[ERROR] Cannot remove file {}: {}", rest_file.display(), e);
                        return Err(());
                    }
                }
                if rest_file.parent().unwrap().read_dir().unwrap().next().is_none() {
                    dprintln!("[INFO] Removing empty directory {}...", rest_file.parent().unwrap().display());
                    match std::fs::remove_dir(rest_file.parent().unwrap()) {
                        Ok(_) => {},
                        Err(e) => {
                            dprintln!("[ERROR] Cannot remove directory {}: {}", rest_file.parent().unwrap().display(), e);
                            return Err(());
                        }
                    }
                }
            }
        }*/

        for file in commit_files.iter() {
            let commit_file_path = &file;
            let commit_file_diff = match pathdiff::diff_paths(&commit_file_path, &commit_path) {
                Some(diff) => diff,
                None => {
                    dprintln!("[ERROR] Cannot diff file {}", commit_file_path.display());
                    return Err(());
                }
            };

            let restore_file_path = restore_path.join(&commit_file_diff);
            // if the file exists in the restore directory, but not in the commit files, remove it

            dprintln!("[INFO] Copying commit file {} to restore directory {}...", commit_file_path.display(), restore_file_path.display());
            if !restore_file_path.parent().unwrap().exists() {
                match std::fs::create_dir_all(restore_file_path.parent().unwrap()) {
                    Ok(_) => {},
                    Err(e) => {
                        dprintln!("[ERROR] Cannot create restore directory {}: {}", restore_file_path.parent().unwrap().display(), e);
                        return Err(());
                    }
                }
            }
            match std::fs::copy(&commit_file_path, &restore_file_path) {
                Ok(_) => {},
                Err(e) => {
                    dprintln!("[ERROR] Failed to copy file to restore directory {}: {}", &commit_file_path.display(), e);
                    dprintln!("[ERROR] Aborting commit!");
                    return Err(());
                }
            }
        }
        return Ok(());
    }

    pub fn get_branch(&self, branch: &String) -> Result<BranchConfig, ()>{
        if self.closed {
            dprintln!("[WARN] Repository {} is closed, skipping commit!", self.name);
            return Err(());
        }

        let branch_path = self.repos_dir.clone().join(&branch);
        let mut branch_config_path = branch_path.join(&branch);
        branch_config_path.set_extension("conf");

        match BranchConfig::from_file(&branch_config_path) {
            Ok(config) => return Ok(config),
            Err(()) => return Err(())
        };
    }

    pub fn get_branches(&self) -> Result<Vec<BranchConfig>, ()> {
        if self.closed {
            dprintln!("[WARN] Repository {} is closed, skipping commit!", self.name);
            return Err(());
        }

        let mut branches: Vec<BranchConfig> = Vec::new();
        let branch_base = self.repos_dir.clone();
        for branch in self.branches.iter() {
            let mut branch_config_path = branch_base.join(&branch).join(&branch);
            branch_config_path.set_extension("conf");

            match BranchConfig::from_file(&branch_config_path) {
                Ok(config) => branches.push(config),
                Err(()) => return Err(())
            };
        }
        Ok(branches)
    }

    pub fn close(&mut self) {
        if self.closed || !self.modified {
            return;
        }
        //dprintln!("[INFO] Closing repository {} and writing to {}", self.name, self.metafile_path.display());
        let config_str = serde_yaml::to_string(&self).unwrap();
        let mut metafile = match std::fs::File::create(&self.metafile_path) {
            Ok(file) => file,
            Err(e) => panic!("Cannot create metafile {}: {}", DEFAULT_CONFIG_FILE, e)
        };

        write!(metafile, "{}\n", CURRENT_CONFIG_VERSION).unwrap();
        write!(metafile, "{}", config_str).unwrap();
        //dprintln!("[INFO] Metafile {} updated!", DEFAULT_CONFIG_FILE);

        if !self.repos_dir.exists() {
            match std::fs::create_dir(&self.repos_dir) {
                Ok(_) => dprintln!("[INFO] Repository directory {} created!", self.repos_dir.display()),
                Err(e) => panic!("Cannot create repository directory {}: {}", self.repos_dir.display(), e)
            }
        }

        for branch in self.branches.iter() {
            if !self.repos_dir.join(&branch).exists() {
                match std::fs::create_dir(&self.repos_dir.join(&branch)) {
                    Ok(_) => dprintln!("[INFO] Repository directory {} created!", self.repos_dir.join(&branch).display()),
                    Err(e) => panic!("Cannot create repository directory {}: {}", self.repos_dir.join(&branch).display(), e)
                }
                BranchConfig::new(branch.clone(), &self.repos_dir);
            }
            
        }

        self.closed = true;
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        self.close();
    }
}
