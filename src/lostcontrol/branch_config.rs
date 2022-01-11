use std::path::PathBuf;
use std::io::{BufReader, BufRead, Write};
use serde::{self, Deserialize, Serialize};
use serde_yaml::{self};

use crate::lostcontrol::{Commit, CURRENT_CONFIG_VERSION, dprintln};

#[derive(Serialize, Deserialize, Debug)]
pub struct BranchConfig {
    #[serde(skip)]
    closed: bool,
    #[serde(skip)]
    modified: bool,
    #[serde(skip)]
    config_path: PathBuf,
    pub name: String,
    pub current_commit: usize,
    commits: Vec<Commit>
}

impl BranchConfig {
    pub fn new(name: String, repo_root_path: &PathBuf) -> BranchConfig {
        let mut config_path = repo_root_path.join(&name).join(&name);
        config_path.set_extension("conf");

        BranchConfig {
            closed: false,
            modified: true,
            config_path: config_path,
            name: name,
            current_commit: 0,
            commits: vec![]
        }
    }

    pub fn from_file(path: &PathBuf) -> Result<BranchConfig, ()> {
        let mut branch_config = match std::fs::File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                dprintln!("[ERROR] Cannot load Branch Config {}: {}", path.display(), e);
                return Err(());
            }
        };
        let mut version = String::new();
        let mut contents = String::new();

        let buf_reader = BufReader::new(&mut branch_config);
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
            dprintln!("[ERROR] Branch Config {}: Branch Config version {} is not supported", path.display(), version);
            return Err(())
        }

        let mut config: BranchConfig = match serde_yaml::from_str(&contents) {
            Ok(config) => config,
            Err(_) =>{
                dprintln!("[ERROR] Branch Config {}: Cannot parse Branch Config", path.display());
                return Err(());
            }
        };
        config.config_path = path.to_path_buf();
        Ok(config)
    }

    pub fn push_commit(&mut self, commit: Commit) {
        if self.closed {
            return;
        }
        self.current_commit = commit.id;
        self.commits.push(commit);
        self.modified = true;
    }

    pub fn remove_commit(&mut self, commit_id: usize) -> Result<(), ()> {
        if self.commits.len() == 0 {
            dprintln!("[ERROR] Cannot remove commit {} from branch {}: Branch is empty", commit_id, self.name);
            return Err(());
        }
        let mut index = 0;
        for commit in self.commits.iter() {
            if commit.id == commit_id {
                self.commits.remove(index);
                self.current_commit = if self.commits.len() > 0 {
                    self.commits.last().unwrap().id
                }
                else {
                    0
                };
                self.modified = true;
                return Ok(());
            }
            index += 1;
        }
        dprintln!("[ERROR] Cannot remove commit {} from branch {}: Commit not found", commit_id, self.name);
        Err(())
    }

    pub fn get_commit(&self, commit_id: usize) -> Option<&Commit> {
        for commit in self.commits.iter() {
            if commit.id == commit_id {
                return Some(commit);
            }
        }
        None
    }

    pub fn get_commits<'vlt>(&'vlt self) -> Vec<Commit> {
        self.commits.clone()
    }

    pub fn commit_count(&self) -> usize {
        self.commits.len() as usize
    }

    pub fn close(&mut self) {
        if self.closed || !self.modified {
            return;
        }
        let config_str = serde_yaml::to_string(&self).unwrap();
        //dprintln!("[INFO] Writing Branch Config: {}", self.config_path.to_str().unwrap());
        let mut branch_config = match std::fs::File::create(&self.config_path) {
            Ok(file) => file,
            Err(e) => panic!("Cannot create Branch Config {}: {}", self.name, e)
        };

        write!(branch_config, "{}\n", CURRENT_CONFIG_VERSION).unwrap();
        write!(branch_config, "{}", config_str).unwrap();
        //dprintln!("[INFO] Branch Config {} updated!", self.name);

        self.closed = true;
    }
}

impl Drop for BranchConfig {
    fn drop(&mut self) {
        //println!("[INFO] Dropping Branch Config {}", self.name);
        self.close();
    }
}
