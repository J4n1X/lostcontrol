mod lostcontrol;

//use libfct4::fct_archive::FctArchive;
use std::path::PathBuf;
use std::process::exit;
use lostcontrol::{Repo};

/*
fn test_archive() {
    let mut archive = match FctArchive::create_new(&PathBuf::from("test.fct"), 64) {
        Ok(archive) => archive,
        Err(e) => panic!("{}", e),
    };
    archive.add_file(&PathBuf::from("/var/shares/public/programming projects/Cross-Platform Projects/lostcontrol/test.txt")).unwrap();
    for header in archive.get_headers(){
        println!("{}", header.file_path.display());
    }
}
*/

fn usage(prg_loc: &str) {
    println!("Usage: {} [options]", prg_loc);
    println!("Options:");
    println!("  -i, init <repo name>\t\tInitialize a new repository");
    println!("  -l, list\t\t\tList information about the repository");
    println!("  -h, help\t\t\t\tDisplay this help message");
    println!("  -b, branch <option> <branch name>\tManage branches");
    println!("  -c, commit <option> <commit message>\tManage commits");
    println!("  -s, stage <option> <files>\t\tStage files for commit");
}

fn init_repo(args: Vec<String>){
    if args.len() > 2 {
        let repo = match Repo::new(&args[2], &PathBuf::new()) {
            Ok(repo) => repo,
            Err(_) => {
                println!("[ERROR] A repository already exists in this folder! Exiting...");
                exit(1);
            }
        };
        println!("[INFO] Repository {} initialized!", repo.name);
    } else {
        println!("[ERROR] No repository name specified!");
        exit(1);
    }
}

fn list_repo(args: Vec<String>){
    let dir: Option<&String> = if args.len() > 2 {
        Some(&args[2])
    } else {
        None
    };
    let repo = match Repo::from_file(dir) {
        Ok(repo) => repo,
        Err(_) => {
            println!("[ERROR] Failed to load repository metafile! Exiting...");
            exit(1);
        }
    };
    
    println!("Repository {}:", repo.name);
    println!("  Branches:");
    for branch in repo.get_branches().unwrap().iter() {
        println!("    {}{}", 
            branch.name,
            if branch.name == repo.current_branch {
                " (current):"
            } else {
                ":"
            }
        );

        let commits = branch.get_commits();

        let last_update_time = match commits.last() {
            Some(commit) => commit.get_time_formatted(),
            None => "Never".to_string()
        };
        println!("      Commits: {}", commits.len());
        println!("      Last updated: {}", last_update_time);
    }
    if repo.staged_files.len() > 0 {
        println!("Staged Files:");
        for file in repo.staged_files.iter() {
            println!("    {}", file);
        }
    }

    exit(0);
}

fn add_stage_files(repo: &mut Repo, args: Vec<String>){
    let staged_count_prev = repo.staged_files.len();
    let staged_files_args = &args[3..];
    let mut staged_files: Vec<PathBuf> = Vec::new();
    for arg in staged_files_args {
        let arg_path = PathBuf::from(arg);
        if arg_path.exists() {
            staged_files.push(arg_path);
        }
    }
    repo.stage_files(&staged_files);
    println!("[INFO] Staged {} files!", repo.staged_files.len() - staged_count_prev);
}

fn remove_stage_files(repo: &mut Repo, args: Vec<String>){
    let mut files: Vec<PathBuf> = Vec::new();
    for file in &args[3..] {
        files.push(PathBuf::from(file));
    }
    repo.unstage_files(&files);
}

fn commit_add(repo: &mut Repo, args: Vec<String>){
    if args.len() < 4 {
        println!("[ERROR] Not enough arguments specified!");
        exit(1);
    }

    let commit_args = &args[3..];
    let mut commit_message = String::new();
    for arg in commit_args {
        commit_message.push_str(arg);
        commit_message.push_str(" ");
    }
    commit_message.pop();
    match repo.commit(commit_message) {
        Ok(staged_files_count) => {
            println!("[INFO] Committed {} files!", staged_files_count);
        },
        Err(()) => {
            println!("[ERROR] Failed to commit!");
        }
    };

    println!("{}", repo.get_branch(&repo.current_branch).unwrap().get_commits().last().unwrap());
}

fn commit_remove(repo: &mut Repo, args: Vec<String>){
    if args.len() < 4 {
        println!("[ERROR] Not enough arguments specified!");
        exit(1);
    }

    let commit_number = match args[3].parse::<usize>() {
        Ok(commit_number) => commit_number,
        Err(_) => {
            println!("[ERROR] Failed to parse commit number!");
            exit(1);
        }
    };

    match repo.remove_commit(commit_number) {
        Ok(()) => {
            println!("[INFO] Removed commit {}!", commit_number);
        },
        Err(()) => {
            println!("[ERROR] Failed to remove commit {}!", commit_number);
        }
    };
}

fn commit_restore(repo: &mut Repo, args: Vec<String>){
    if args.len() < 3 {
        println!("[ERROR] Not enough arguments specified!");
        exit(1);
    }

    let commit_number = match args.get(3) {
        Some(arg) => {
            match arg.parse::<usize>() {
                Ok(commit_number) => commit_number,
                Err(_) => {
                    println!("[ERROR] Failed to parse commit number!");
                 exit(1);
                }
            }
        },
        None => repo.get_branch(&repo.current_branch).unwrap().current_commit
    };

    match repo.restore_commit(commit_number) {
        Ok(()) => {
            println!("[INFO] Restored commit {}!", commit_number);
        },
        Err(()) => {
            println!("[ERROR] Failed to restore commit {}!", commit_number);
        }
    };
}

fn commit_list(repo: &Repo, args: Vec<String>){
    if args.len() < 3 {
        println!("[ERROR] Not enough arguments specified!");
        exit(1);
    }

    match args.get(3) {
        Some(selector) => {
            let commit_number = match selector.parse::<usize>() {
                Ok(commit_number) => commit_number,
                Err(_) => {
                    println!("[ERROR] Failed to parse commit number!");
                    exit(1);
                }
            };
            match repo.get_branch(&repo.current_branch).unwrap().get_commit(commit_number) {
                Some(commit) => println!("{}", commit),
                None => {
                    println!("[ERROR] Failed to get commit {}!", commit_number);
                    exit(1);
                }
            };
            
        },
        None => {
            let commits = repo.get_branch(&repo.current_branch).unwrap().get_commits();        

            if commits.len() > 0 {
                for commit in commits.iter() {
                    print!("{}", commit);
                    println!("----------------------------------------");
                }
            }
            else {
                println!("[INFO] Branch {} contains no commits", repo.current_branch);
            }
        }
    }

}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "-i" | "init" => {
                init_repo(args);
            },
            "-l" | "list" => {
                list_repo(args);
            },
            "-s" | "stage" => {
                if args.len() < 4 {
                    println!("[ERROR] Not enough arguments specified!");
                    exit(1);
                }

                let mut repo = match Repo::from_file(None) {
                    Ok(repo) => repo,
                    Err(_) => {
                        println!("[ERROR] Failed to load repository metafile! Exiting...");
                        exit(1);
                    }
                };
                match args[2].as_str() {
                    "add" => {
                        add_stage_files(&mut repo, args);
                    },
                    "remove" => {
                        remove_stage_files(&mut repo, args);
                    },
                    "clear" => {
                        repo.unstage_all();
                        println!("[INFO] Cleared all staged files!");
                    },
                    _ => {
                        println!("[ERROR] Invalid stage option!");
                        repo.close();
                        exit(1);
                    }
                }
                repo.close();
            },
            "-c" | "commit" => {
                
                let mut repo = match Repo::from_file(None) {
                    Ok(repo) => repo,
                    Err(_) => {
                        println!("[ERROR] Failed to load repository metafile! Exiting...");
                        exit(1);
                    }
                };
                
                match args[2].as_str() {
                    "add" => {
                        commit_add(&mut repo, args);
                    },
                    "remove" => {
                        commit_remove(&mut repo, args);
                    }
                    "restore" => {
                        commit_restore(&mut repo, args);
                    }
                    "list" => {
                        commit_list(&repo, args);
                    }
                    _ => {
                        println!("[ERROR] Invalid commit option!");
                        repo.close();
                        exit(1);
                    }
                }
                repo.close();
                
            },
            "-b" | "branch" => {
                if args.len() > 2 {
                    match args[2].as_str() {
                        _ => {
                            todo!();
                        }
                    }
                }
            },
            "-h" | "help" => {
                usage(args[0].as_str());
                exit(0);
            },
            _ => {
                usage(args[0].as_str());
                println!("[ERROR] Invalid option: {}", args[1]);
                exit(1);
            }
        } // match
    } // if args.len() > 1
    else {
        usage(args[0].as_str());
        println!("[ERROR] No options provided");
        exit(1);
    }
}

/* fn main1() {
    let mut test_branch = BranchConfig::new(String::from("test"));
    test_branch.commits.push(Commit::new(0, String::from("test commit")));
    test_branch.close();
    test_branch = BranchConfig::from_file(&PathBuf::from("test.yml")).unwrap();
    println!("{:#?}", test_branch);
} */
