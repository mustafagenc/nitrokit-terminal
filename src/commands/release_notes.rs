use colored::*;
use crate::utils::{get_repository, log_info, log_error, log_success, write_string_to_file};
use git2::Repository;
use std::collections::HashMap;
use chrono::TimeZone;

pub fn generate_release_notes() {
    log_info("Starting release notes generation...");
    
    match get_repository(".") {
        Ok(repo) => {
            log_info("Repository found, analyzing commits...");
            
            // Get repository information
            let repo_info = get_repository_info(&repo);
            log_info(&format!("Repository: {}", repo_info.url.cyan()));
            
            // Get latest tag or create default
            let (current_tag, previous_tag) = get_tag_range(&repo);
            log_info(&format!("Generating release notes for tag: {}", current_tag.cyan()));
            
            if let Some(ref prev_tag) = previous_tag {
                log_info(&format!("Comparing with previous tag: {}", prev_tag.cyan()));
            } else {
                log_info("No previous tag found, generating initial release notes");
            }
            
            match get_commits_between_tags(&repo, &previous_tag, &current_tag) {
                Ok(commits) => {
                    let release_notes = generate_comprehensive_release_notes(&repo_info, &current_tag, &previous_tag, &commits);
                    
                    // Sadece tag-specific dosya oluştur
                    let filename = format!("RELEASE_NOTES_{}.md", current_tag.replace("/", "_"));
                    match write_string_to_file(&filename, &release_notes) {
                        Ok(_) => {
                            log_success("Release notes generated successfully!");
                            println!("{}", format!("📄 File created: {}", filename).green());
                        }
                        Err(e) => {
                            log_error(&format!("Failed to write release notes: {}", e));
                        }
                    }
                }
                Err(e) => {
                    log_error(&format!("Failed to get commits: {}", e));
                }
            }
        }
        Err(e) => {
            log_error(&format!("Not a git repository or git error: {}", e));
        }
    }
}

fn get_tag_range(repo: &Repository) -> (String, Option<String>) {
    // Try to get latest tag
    if let Ok(mut tags) = get_all_tags(repo) {
        log_info(&format!("Found raw tags: {:?}", tags));
        
        // Filter out non-version tags and sort properly
        tags.retain(|tag| is_version_tag(tag));
        log_info(&format!("Filtered version tags: {:?}", tags));
        
        if !tags.is_empty() {
            // Sort tags by version (semantic versioning aware)
            tags.sort_by(|a, b| compare_version_tags(a, b));
            tags.reverse(); // Most recent first
            
            let latest_tag = tags.first().unwrap().clone();
            let previous_tag = tags.get(1).cloned();
            
            log_info(&format!("Using latest tag: {}", latest_tag));
            if let Some(ref prev) = previous_tag {
                log_info(&format!("Previous tag: {}", prev));
            }
            
            (latest_tag, previous_tag)
        } else {
            // No version tags found, check if we're on a specific commit/branch
            get_current_commit_as_tag(repo)
        }
    } else {
        // Can't read tags, get current commit info
        get_current_commit_as_tag(repo)
    }
}

fn get_current_commit_as_tag(repo: &Repository) -> (String, Option<String>) {
    log_info("No version tags found, analyzing current commit...");
    
    // Try to get the latest commit to generate a meaningful tag
    if let Ok(head) = repo.head() {
        if let Some(oid) = head.target() {
            if let Ok(commit) = repo.find_commit(oid) {
                let short_hash = oid.to_string()[..7].to_string();
                let timestamp = commit.time().seconds();
                
                // Create a meaningful tag based on commit info
                let date = chrono::Utc.timestamp_opt(timestamp, 0)
                    .single()
                    .map(|dt| dt.format("%Y.%m.%d").to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                
                // Check if we can determine version from branch name or commit message
                let branch_name = get_current_branch(repo);
                let auto_tag = generate_smart_tag(&branch_name, &date, &short_hash, &commit.message().unwrap_or(""));
                
                log_info(&format!("Generated tag from current commit: {}", auto_tag));
                return (auto_tag, None);
            }
        }
    }
    
    // Ultimate fallback
    log_info("Could not access git history, using fallback tag");
    ("v0.1.0-dev".to_string(), None)
}

fn get_current_branch(repo: &Repository) -> String {
    if let Ok(head) = repo.head() {
        if let Some(branch_name) = head.shorthand() {
            return branch_name.to_string();
        }
    }
    "main".to_string()
}

fn generate_smart_tag(branch: &str, date: &str, hash: &str, commit_msg: &str) -> String {
    // Analyze branch name for version hints
    let branch_lower = branch.to_lowercase();
    let commit_lower = commit_msg.to_lowercase();
    
    // Check for version in branch name
    if branch_lower.contains("release") || branch_lower.contains("rel") {
        if let Some(version) = extract_version_from_string(branch) {
            return format!("v{}", version);
        }
    }
    
    // Check for version bumps in commit message
    if commit_lower.contains("major") || commit_lower.contains("breaking") {
        return format!("v1.0.0-dev.{}.{}", date, hash);
    } else if commit_lower.contains("minor") || commit_lower.contains("feature") || commit_lower.contains("feat") {
        return format!("v0.2.0-dev.{}.{}", date, hash);
    } else if commit_lower.contains("patch") || commit_lower.contains("fix") || commit_lower.contains("hotfix") {
        return format!("v0.1.1-dev.{}.{}", date, hash);
    }
    
    // Check branch patterns
    match branch_lower.as_str() {
        "main" | "master" => format!("v0.1.0-main.{}.{}", date, hash),
        "develop" | "dev" => format!("v0.1.0-dev.{}.{}", date, hash),
        branch if branch.starts_with("feature/") => format!("v0.1.0-feature.{}.{}", date, hash),
        branch if branch.starts_with("hotfix/") => format!("v0.1.0-hotfix.{}.{}", date, hash),
        _ => format!("v0.1.0-{}.{}.{}", branch_lower.replace("/", "-"), date, hash)
    }
}

fn extract_version_from_string(text: &str) -> Option<String> {
    // Try to extract version numbers from text like "release/1.2.3" or "rel-v2.0.0"
    let version_regex = regex::Regex::new(r"v?(\d+\.\d+\.\d+(?:-[a-zA-Z0-9]+)?)").ok()?;
    
    if let Some(captures) = version_regex.captures(text) {
        return captures.get(1).map(|m| m.as_str().to_string());
    }
    
    None
}

fn is_version_tag(tag: &str) -> bool {
    // Check if tag looks like a version (v1.0.0, 1.0.0, v2.1.0-beta, etc.)
    let tag_lower = tag.to_lowercase();
    
    // Must contain numbers
    if !tag.chars().any(|c| c.is_numeric()) {
        return false;
    }
    
    // Skip obviously non-version tags
    if tag_lower.contains("backup") || 
       tag_lower.contains("temp") || 
       tag_lower.contains("test") ||
       tag_lower.contains("old") {
        return false;
    }
    
    // Common version patterns
    tag_lower.starts_with("v") || 
    tag.chars().next().unwrap_or('x').is_numeric() ||
    tag_lower.contains("release") ||
    tag_lower.contains("version") ||
    tag_lower.contains("rel-") ||
    // Semantic versioning pattern
    tag.matches('.').count() >= 1
}

fn get_all_tags(repo: &Repository) -> Result<Vec<String>, git2::Error> {
    let mut tags = Vec::new();
    
    repo.tag_foreach(|_oid, name| {
        if let Ok(name_str) = std::str::from_utf8(name) {
            if name_str.starts_with("refs/tags/") {
                let tag_name = name_str.strip_prefix("refs/tags/").unwrap();
                tags.push(tag_name.to_string());
            }
        }
        true
    })?;
    
    // Also try to get lightweight tags if no annotated tags found
    if tags.is_empty() {
        if let Ok(references) = repo.references() {
            for reference in references {
                if let Ok(reference) = reference {
                    if let Some(name) = reference.name() {
                        if name.starts_with("refs/tags/") {
                            let tag_name = name.strip_prefix("refs/tags/").unwrap();
                            tags.push(tag_name.to_string());
                        }
                    }
                }
            }
        }
    }
    
    log_info(&format!("Found {} tags in repository", tags.len()));
    
    Ok(tags)
}

fn compare_version_tags(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    
    // Simple version comparison - you might want to use a proper semver library
    let extract_version = |tag: &str| -> Vec<u32> {
        tag.chars()
            .filter(|c| c.is_numeric() || *c == '.')
            .collect::<String>()
            .split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };
    
    let version_a = extract_version(a);
    let version_b = extract_version(b);
    
    // Compare version numbers
    for (va, vb) in version_a.iter().zip(version_b.iter()) {
        match va.cmp(vb) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    
    // If one version has more parts, it's considered newer
    version_a.len().cmp(&version_b.len())
}

fn get_commits_between_tags(repo: &Repository, previous_tag: &Option<String>, current_tag: &String) -> Result<Vec<CommitInfo>, git2::Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    
    // Set range
    if let Some(prev_tag) = previous_tag {
        if let Ok(prev_oid) = repo.refname_to_id(&format!("refs/tags/{}", prev_tag)) {
            revwalk.hide(prev_oid)?;
        }
    }
    
    // Get HEAD or tag
    let head_oid = if let Ok(tag_oid) = repo.refname_to_id(&format!("refs/tags/{}", current_tag)) {
        tag_oid
    } else {
        repo.head()?.target().unwrap()
    };
    
    revwalk.push(head_oid)?;
    
    let mut commits = Vec::new();
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        
        let commit_info = CommitInfo {
            message: commit.message().unwrap_or("").to_string(),
            author_name: commit.author().name().unwrap_or("").to_string(),
            author_email: commit.author().email().unwrap_or("").to_string(),
            hash: commit.id().to_string(),
            timestamp: commit.time().seconds(),
        };
        
        commits.push(commit_info);
    }
    
    Ok(commits)
}

#[derive(Debug, Clone)]
struct CommitInfo {
    message: String,
    author_name: String,
    author_email: String,
    hash: String,
    timestamp: i64,
}

impl CommitInfo {
    fn short_hash(&self) -> String {
        if self.hash.len() >= 7 {
            self.hash[..7].to_string()
        } else {
            self.hash.clone()
        }
    }
    
    fn format_date(&self) -> String {
        use chrono::{Utc, TimeZone};
        let dt = Utc.timestamp_opt(self.timestamp, 0).single();
        if let Some(dt) = dt {
            dt.format("%Y-%m-%d").to_string()
        } else {
            "unknown".to_string()
        }
    }
    
    fn format_time(&self) -> String {
        use chrono::{Utc, TimeZone};
        let dt = Utc.timestamp_opt(self.timestamp, 0).single();
        if let Some(dt) = dt {
            dt.format("%H:%M").to_string()
        } else {
            "unknown".to_string()
        }
    }
}

#[derive(Debug, Clone)]
struct RepositoryInfo {
    url: String,
    name: String,
    owner: String,
    is_github: bool,
    is_gitlab: bool,
    is_bitbucket: bool,
}

impl Default for RepositoryInfo {
    fn default() -> Self {
        Self {
            url: "unknown".to_string(),
            name: "unknown".to_string(),
            owner: "unknown".to_string(),
            is_github: false,
            is_gitlab: false,
            is_bitbucket: false,
        }
    }
}

fn get_repository_info(repo: &Repository) -> RepositoryInfo {
    let mut repo_info = RepositoryInfo::default();
    
    // Try to get remote URL
    if let Ok(remotes) = repo.remotes() {
        for remote_name in remotes.iter() {
            if let Some(remote_name) = remote_name {
                if let Ok(remote) = repo.find_remote(remote_name) {
                    if let Some(url) = remote.url() {
                        repo_info.url = url.to_string();
                        
                        // Parse URL to extract owner and repo name
                        parse_git_url(&mut repo_info, url);
                        break;
                    }
                }
            }
        }
    }
    
    repo_info
}

fn parse_git_url(repo_info: &mut RepositoryInfo, url: &str) {
    // Remove .git suffix if present
    let clean_url = url.trim_end_matches(".git");
    
    // Check for GitHub
    if clean_url.contains("github.com") {
        repo_info.is_github = true;
        if let Some(path) = extract_repo_path(clean_url, "github.com") {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                repo_info.owner = parts[0].to_string();
                repo_info.name = parts[1].to_string();
            }
        }
    }
    // Check for GitLab
    else if clean_url.contains("gitlab.com") {
        repo_info.is_gitlab = true;
        if let Some(path) = extract_repo_path(clean_url, "gitlab.com") {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                repo_info.owner = parts[0].to_string();
                repo_info.name = parts[1].to_string();
            }
        }
    }
    // Check for Bitbucket
    else if clean_url.contains("bitbucket.org") {
        repo_info.is_bitbucket = true;
        if let Some(path) = extract_repo_path(clean_url, "bitbucket.org") {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                repo_info.owner = parts[0].to_string();
                repo_info.name = parts[1].to_string();
            }
        }
    }
    // Generic git repository
    else {
        // Try to extract from any git URL pattern
        if let Some(repo_name) = clean_url.split('/').last() {
            repo_info.name = repo_name.to_string();
        }
        if let Some(owner) = clean_url.split('/').nth_back(1) {
            repo_info.owner = owner.to_string();
        }
    }
}

fn extract_repo_path(url: &str, domain: &str) -> Option<String> {
    // Handle both HTTPS and SSH URLs
    if url.starts_with("https://") {
        // https://github.com/owner/repo
        url.split(&format!("{}/", domain)).nth(1).map(|s| s.to_string())
    } else if url.starts_with("git@") {
        // git@github.com:owner/repo
        url.split(':').nth(1).map(|s| s.to_string())
    } else {
        None
    }
}

#[derive(Debug)]
struct CategorizedCommits {
    features: Vec<String>,
    fixes: Vec<String>,
    improvements: Vec<String>,
    docs: Vec<String>,
    deps: Vec<String>,
    translations: Vec<String>,
    breaking_changes: Vec<String>,
    security: Vec<String>,
    other: Vec<String>,
}

impl CategorizedCommits {
    fn new() -> Self {
        Self {
            features: Vec::new(),
            fixes: Vec::new(),
            improvements: Vec::new(),
            docs: Vec::new(),
            deps: Vec::new(),
            translations: Vec::new(),
            breaking_changes: Vec::new(),
            security: Vec::new(),
            other: Vec::new(),
        }
    }
}

fn categorize_commits(commits: &[CommitInfo]) -> CategorizedCommits {
    let mut categorized = CategorizedCommits::new();
    
    for commit in commits {
        let first_line = commit.message.lines().next().unwrap_or("").to_lowercase();
        let commit_title = commit.message.lines().next().unwrap_or("").to_string();
        
        // Add commit hash and date info for better tracking
        let enhanced_title = format!("{} ({})", commit_title, commit.short_hash());
        
        if first_line.contains("breaking") || first_line.contains("breaking change") {
            categorized.breaking_changes.push(enhanced_title);
        } else if first_line.contains("security") || first_line.contains("vulnerability") || first_line.contains("cve") {
            categorized.security.push(enhanced_title);
        } else if first_line.contains("feat") || first_line.contains("feature") || first_line.contains("add") {
            categorized.features.push(enhanced_title);
        } else if first_line.contains("fix") || first_line.contains("bug") || first_line.contains("hotfix") {
            categorized.fixes.push(enhanced_title);
        } else if first_line.contains("improve") || first_line.contains("enhance") || first_line.contains("update") || first_line.contains("refactor") {
            categorized.improvements.push(enhanced_title);
        } else if first_line.contains("doc") || first_line.contains("readme") {
            categorized.docs.push(enhanced_title);
        } else if first_line.contains("dep") || first_line.contains("bump") || first_line.contains("upgrade") || first_line.contains("yarn") || first_line.contains("npm") || first_line.contains("cargo") {
            categorized.deps.push(enhanced_title);
        } else if first_line.contains("translation") || first_line.contains("i18n") || first_line.contains("locale") {
            categorized.translations.push(enhanced_title);
        } else {
            categorized.other.push(enhanced_title);
        }
    }
    
    categorized
}

fn is_prerelease(tag: &str) -> bool {
    tag.contains("-alpha") || tag.contains("-beta") || tag.contains("-rc") || tag.contains("-pre")
}

fn generate_compare_url(repo_info: &RepositoryInfo, from_tag: &str, to_tag: &str) -> String {
    if repo_info.is_github {
        format!("{}/compare/{}...{}", repo_info.url.trim_end_matches(".git"), from_tag, to_tag)
    } else if repo_info.is_gitlab {
        format!("{}/compare/{}...{}", repo_info.url.trim_end_matches(".git"), from_tag, to_tag)
    } else if repo_info.is_bitbucket {
        format!("{}/compare/{}..{}", repo_info.url.trim_end_matches(".git"), from_tag, to_tag)
    } else {
        format!("{}/commits/{}", repo_info.url.trim_end_matches(".git"), to_tag)
    }
}

fn generate_commits_url(repo_info: &RepositoryInfo, tag: &str) -> String {
    if repo_info.is_github {
        format!("{}/commits/{}", repo_info.url.trim_end_matches(".git"), tag)
    } else if repo_info.is_gitlab {
        format!("{}/commits/{}", repo_info.url.trim_end_matches(".git"), tag)
    } else if repo_info.is_bitbucket {
        format!("{}/commits/{}", repo_info.url.trim_end_matches(".git"), tag)
    } else {
        format!("{}/commits/{}", repo_info.url.trim_end_matches(".git"), tag)
    }
}

fn generate_issues_url(repo_info: &RepositoryInfo) -> String {
    if repo_info.is_github {
        format!("{}/issues", repo_info.url.trim_end_matches(".git"))
    } else if repo_info.is_gitlab {
        format!("{}/issues", repo_info.url.trim_end_matches(".git"))
    } else if repo_info.is_bitbucket {
        format!("{}/issues", repo_info.url.trim_end_matches(".git"))
    } else {
        format!("{}/issues", repo_info.url.trim_end_matches(".git"))
    }
}

fn generate_new_issue_url(repo_info: &RepositoryInfo) -> String {
    if repo_info.is_github {
        format!("{}/issues/new", repo_info.url.trim_end_matches(".git"))
    } else if repo_info.is_gitlab {
        format!("{}/issues/new", repo_info.url.trim_end_matches(".git"))
    } else if repo_info.is_bitbucket {
        format!("{}/issues/new", repo_info.url.trim_end_matches(".git"))
    } else {
        format!("{}/issues/new", repo_info.url.trim_end_matches(".git"))
    }
}

fn get_contributors_with_stats(commits: &[CommitInfo]) -> Vec<(String, String, usize)> {
    let mut contributors: HashMap<String, (String, usize)> = HashMap::new();
    
    for commit in commits {
        let entry = contributors.entry(commit.author_email.clone())
            .or_insert((commit.author_name.clone(), 0));
        entry.1 += 1;
    }
    
    let mut result: Vec<(String, String, usize)> = contributors
        .into_iter()
        .map(|(email, (name, count))| (email, name, count))
        .collect();
    
    // Sort by commit count, descending
    result.sort_by(|a, b| b.2.cmp(&a.2));
    
    result
}

fn format_github_username_with_stats(email: &str, name: &str, commit_count: usize, repo_info: &RepositoryInfo) -> String {
    let commits_text = if commit_count == 1 {
        "1 commit".to_string()
    } else {
        format!("{} commits", commit_count)
    };
    
    if email.contains("@users.noreply.github.com") && repo_info.is_github {
        // GitHub no-reply email format - fix temporary value issue
        let temp_email = email.replace("@users.noreply.github.com", "");
        let github_user = temp_email.split('+').last().unwrap_or(email);
        format!("- [@{}](https://github.com/{}) ({}) - {}", github_user, github_user, name, commits_text)
    } else if repo_info.is_gitlab {
        // Try to format for GitLab
        format!("- {} ({}) - {}", name, email, commits_text)
    } else if repo_info.is_bitbucket {
        // Try to format for Bitbucket
        format!("- {} ({}) - {}", name, email, commits_text)
    } else {
        // Generic format
        format!("- {} ({}) - {}", name, email, commits_text)
    }
}

fn generate_comprehensive_release_notes(repo_info: &RepositoryInfo, current_tag: &str, previous_tag: &Option<String>, commits: &[CommitInfo]) -> String {
    let mut output = String::new();
    
    let comparison_text = if let Some(ref prev_tag) = previous_tag {
        format!("Changes since {}", prev_tag)
    } else {
        "Initial release".to_string()
    };
    
    // Header
    output.push_str(&format!("# Release {}\n\n", current_tag));
    output.push_str(&format!("## 📋 {}\n\n", comparison_text));
    
    // Release date and stats
    let release_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    output.push_str(&format!("- **Release Date:** {}\n", release_date));
    output.push_str(&format!("- **Repository:** {}\n", repo_info.url.trim_end_matches(".git")));
    output.push_str(&format!("- **Total Commits:** {}\n", commits.len()));
    
    // Add commit date range
    if !commits.is_empty() {
        let oldest_commit = commits.last().unwrap(); // commits are sorted by time, newest first
        let newest_commit = commits.first().unwrap();
        output.push_str(&format!("- **Commit Range:** {} to {}\n", 
            oldest_commit.format_date(), 
            newest_commit.format_date()
        ));
    }
    output.push_str("\n");
    
    // Pre-release warning
    if is_prerelease(current_tag) {
        output.push_str("🚨 **This is a pre-release version** - Use with caution in production environments.\n\n");
    }
    
    // Categorize commits
    let categorized = categorize_commits(commits);
    
    // Breaking changes (highest priority)
    if !categorized.breaking_changes.is_empty() {
        output.push_str("## ⚠️ Breaking Changes\n\n");
        output.push_str("🚨 **Important:** This release contains breaking changes. Please review the migration guide before upgrading.\n\n");
        for change in &categorized.breaking_changes {
            output.push_str(&format!("- {}\n", change));
        }
        output.push_str("\n");
    }
    
    // Security updates
    if !categorized.security.is_empty() {
        output.push_str("## 🔒 Security Updates\n\n");
        output.push_str("🛡️ **Security patches included in this release:**\n\n");
        for security in &categorized.security {
            output.push_str(&format!("- {}\n", security));
        }
        output.push_str("\n");
    }
    
    // Features
    if !categorized.features.is_empty() {
        output.push_str("## ✨ New Features\n\n");
        for feature in &categorized.features {
            output.push_str(&format!("- {}\n", feature));
        }
        output.push_str("\n");
    }
    
    // Bug fixes
    if !categorized.fixes.is_empty() {
        output.push_str("## 🐛 Bug Fixes\n\n");
        for fix in &categorized.fixes {
            output.push_str(&format!("- {}\n", fix));
        }
        output.push_str("\n");
    }
    
    // Improvements
    if !categorized.improvements.is_empty() {
        output.push_str("## 🔧 Improvements\n\n");
        for improvement in &categorized.improvements {
            output.push_str(&format!("- {}\n", improvement));
        }
        output.push_str("\n");
    }
    
    // Translations
    if !categorized.translations.is_empty() {
        output.push_str("## 🌍 Translation Updates\n\n");
        for translation in &categorized.translations {
            output.push_str(&format!("- {}\n", translation));
        }
        output.push_str("\n");
    }
    
    // Documentation
    if !categorized.docs.is_empty() {
        output.push_str("## 📚 Documentation\n\n");
        for doc in &categorized.docs {
            output.push_str(&format!("- {}\n", doc));
        }
        output.push_str("\n");
    }
    
    // Dependencies
    if !categorized.deps.is_empty() {
        output.push_str("## 📦 Dependencies\n\n");
        for dep in &categorized.deps {
            output.push_str(&format!("- {}\n", dep));
        }
        output.push_str("\n");
    }
    
    // Other changes (if any significant ones exist)
    if !categorized.other.is_empty() && categorized.other.len() <= 10 {
        output.push_str("## 🔄 Other Changes\n\n");
        for other in &categorized.other {
            output.push_str(&format!("- {}\n", other));
        }
        output.push_str("\n");
    }
    
    // Contributors with commit stats
    let contributors = get_contributors_with_stats(commits);
    if !contributors.is_empty() {
        output.push_str("## 👥 Contributors\n\n");
        output.push_str("Thanks to all the contributors who made this release possible:\n\n");
        for (email, name, commit_count) in contributors {
            let formatted_contributor = format_github_username_with_stats(&email, &name, commit_count, repo_info);
            output.push_str(&format!("{}\n", formatted_contributor));
        }
        output.push_str("\n");
    }
    
    // Installation instructions
    output.push_str("## 🚀 Installation & Upgrade\n\n");
    output.push_str("### For new projects:\n");
    output.push_str("```bash\n");
    output.push_str(&format!("git clone {}\n", repo_info.url));
    output.push_str(&format!("cd {}\n", repo_info.name));
    output.push_str(&format!("git checkout {}\n", current_tag));
    
    // Smart build instructions based on project type
    if repo_info.name.to_lowercase().contains("rust") || repo_info.url.contains("rust") {
        output.push_str("cargo build --release\n");
    } else if repo_info.name.to_lowercase().contains("node") || repo_info.url.contains("node") || repo_info.name.to_lowercase().contains("js") {
        output.push_str("npm install\n");
        output.push_str("npm run build\n");
    } else if repo_info.name.to_lowercase().contains("python") || repo_info.url.contains("python") {
        output.push_str("pip install -r requirements.txt\n");
    } else {
        output.push_str("# Follow project-specific build instructions\n");
    }
    output.push_str("```\n\n");
    
    output.push_str("### For existing projects:\n");
    output.push_str("```bash\n");
    output.push_str("git pull origin main\n");
    output.push_str(&format!("git checkout {}\n", current_tag));
    
    if repo_info.name.to_lowercase().contains("rust") || repo_info.url.contains("rust") {
        output.push_str("cargo update\n");
        output.push_str("cargo build --release\n");
    } else if repo_info.name.to_lowercase().contains("node") || repo_info.url.contains("node") || repo_info.name.to_lowercase().contains("js") {
        output.push_str("npm update\n");
        output.push_str("npm run build\n");
    } else if repo_info.name.to_lowercase().contains("python") || repo_info.url.contains("python") {
        output.push_str("pip install --upgrade -r requirements.txt\n");
    } else {
        output.push_str("# Follow project-specific update instructions\n");
    }
    output.push_str("```\n\n");
    
    // Detailed commit timeline (for smaller releases)
    if commits.len() <= 20 {
        output.push_str("## 📊 Detailed Timeline\n\n");
        output.push_str("| Date | Time | Commit | Author | Message |\n");
        output.push_str("|------|------|--------|--------|---------|\n");
        for commit in commits.iter().take(20) {
            let short_message = commit.message.lines().next().unwrap_or("")
                .chars().take(50).collect::<String>();
            let short_message = if commit.message.lines().next().unwrap_or("").len() > 50 {
                format!("{}...", short_message)
            } else {
                short_message
            };
            
            output.push_str(&format!("| {} | {} | `{}` | {} | {} |\n",
                commit.format_date(),
                commit.format_time(),
                commit.short_hash(),
                commit.author_name,
                short_message
            ));
        }
        output.push_str("\n");
    }
    
    // Full changelog
    output.push_str("## 📝 Full Changelog\n\n");
    if let Some(ref prev_tag) = previous_tag {
        output.push_str(&format!("**Full Changelog**: {}\n", generate_compare_url(repo_info, prev_tag, current_tag)));
    } else {
        output.push_str(&format!("**Full Changelog**: {}\n", generate_commits_url(repo_info, current_tag)));
    }
    output.push_str("\n");
    
    // Additional information
    output.push_str("---\n\n");
    output.push_str("### 🔗 Useful Links\n\n");
    output.push_str(&format!("- 📖 **Documentation**: [README.md]({}#readme)\n", repo_info.url.trim_end_matches(".git")));
    
    if repo_info.is_github {
        output.push_str(&format!("- 💬 **Discussions**: [GitHub Discussions]({}/discussions)\n", repo_info.url.trim_end_matches(".git")));
    }
    
    output.push_str(&format!("- 🐛 **Report Issues**: [Issues]({})\n", generate_issues_url(repo_info)));
    output.push_str("\n");
    
    output.push_str("### 🆘 Getting Help\n\n");
    output.push_str("If you encounter any issues with this release:\n\n");
    output.push_str("1. Check the project documentation and README\n");
    output.push_str(&format!("2. Search [existing issues]({})\n", generate_issues_url(repo_info)));
    output.push_str(&format!("3. Create a [new issue]({}) with detailed information\n", generate_new_issue_url(repo_info)));
    output.push_str("\n");
    
    output.push_str("---\n\n");
    output.push_str(&format!("**Enjoy building with {}! 🚀**\n", repo_info.name));
    
    output
}