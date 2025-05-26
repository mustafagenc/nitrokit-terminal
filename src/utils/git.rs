use git2::{Repository, Commit};

pub fn get_repository(path: &str) -> Result<Repository, git2::Error> {
    Repository::open(path)
}

pub fn get_latest_commits(repo: &Repository, count: usize) -> Result<Vec<Commit>, git2::Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    
    let mut commits = Vec::new();
    for (i, oid) in revwalk.enumerate() {
        if i >= count {
            break;
        }
        
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        commits.push(commit);
    }
    
    Ok(commits)
}

pub fn get_commit_message(commit: &Commit) -> Option<String> {
    commit.message().map(|s| s.to_string())
}