use git2::Repository;

pub fn get_repository(path: &str) -> Result<Repository, git2::Error> {
    Repository::open(path)
}
