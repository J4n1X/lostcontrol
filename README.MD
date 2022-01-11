# Lostcontrol Version Control System
This is a simple version control system because I grew a bit tired of git

## Commands

### help
Displays the help. What did you expect?

### init
Initializes a repository.

Usage: ```init <repo name>```

### list
Lists general information about the repository

Usage: ```list```

### branch
Not yet implemented, but used to manipulate branches

### stage
Stage files for commit.

Usage: ```stage <subcommand>```

Subcommands:
  - ```add <files/directory>```: Add files for staging
  - ```remove <files/directory>```: Remove files from staging
  - ```clear```: Clear all staged files

### commit
Manipulate commits on the currently selected branch

Usage: ```commit <subcommand>```

Subcommands:
  - ```add <commit message>``` Create a new commit on the current branch
  - ```remove <commit id>``` Remove the commit with the passed ID
  - ```restore (commit id)``` Restore the commit with the passed ID or the current commit, if none is passed
  - ```list (commit id)``` List information about the commit with the passed ID or all commits, if none is passed
