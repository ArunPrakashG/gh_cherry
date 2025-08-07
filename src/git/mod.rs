use anyhow::{Context, Result};
use git2::{
    CherrypickOptions, Oid, Repository, RepositoryState, Signature,
};
use std::path::Path;

pub struct GitOperations {
    repo: Repository,
}

#[derive(Debug)]
pub struct CherrypickResult {
    pub success: bool,
    pub conflicts: Vec<String>,
    pub commit_sha: Option<String>,
}

#[allow(dead_code)] // Methods for future Git operations functionality
impl GitOperations {
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Result<Self> {
        let repo = Repository::open(repo_path)
            .context("Failed to open Git repository. Are you in a Git repository?")?;
        
        Ok(Self { repo })
    }

    /// Discovers the Git repository from the current directory
    pub fn discover() -> Result<Self> {
        let repo = Repository::discover(".")
            .context("No Git repository found. Please run this command from within a Git repository.")?;
        
        Ok(Self { repo })
    }

    /// Checks if the repository is in a clean state
    pub fn is_clean(&self) -> Result<bool> {
        let statuses = self.repo.statuses(None)
            .context("Failed to check repository status")?;

        Ok(statuses.is_empty())
    }

    /// Gets the current branch name
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()
            .context("Failed to get HEAD reference")?;
        
        let branch_name = head.shorthand()
            .context("Failed to get branch name")?;
        
        Ok(branch_name.to_string())
    }

    /// Switches to the specified branch
    pub fn checkout_branch(&self, branch_name: &str) -> Result<()> {
        tracing::info!("Checking out branch: {}", branch_name);

        // Find the branch
        let branch = self.repo.find_branch(branch_name, git2::BranchType::Local)
            .or_else(|_| {
                // Try to find remote branch and create local tracking branch
                self.create_tracking_branch(branch_name)
            })
            .with_context(|| format!("Branch '{}' not found", branch_name))?;

        let commit = branch.get().peel_to_commit()
            .context("Failed to get commit for branch")?;

        // Checkout the branch
        self.repo.checkout_tree(commit.as_object(), None)
            .context("Failed to checkout tree")?;

        // Update HEAD
        self.repo.set_head(&format!("refs/heads/{}", branch_name))
            .context("Failed to update HEAD")?;

        tracing::info!("Successfully checked out branch: {}", branch_name);
        Ok(())
    }

    fn create_tracking_branch(&self, branch_name: &str) -> Result<git2::Branch<'_>, git2::Error> {
        // Try to find remote branch (usually origin/branch_name)
        let remote_branch = self.repo.find_branch(&format!("origin/{}", branch_name), git2::BranchType::Remote)?;
        let remote_commit = remote_branch.get().peel_to_commit()?;

        // Create local tracking branch
        let local_branch = self.repo.branch(branch_name, &remote_commit, false)?;
        
        // Set up tracking
        let mut branch_config = self.repo.config()?;
        branch_config.set_str(&format!("branch.{}.remote", branch_name), "origin")?;
        branch_config.set_str(&format!("branch.{}.merge", branch_name), &format!("refs/heads/{}", branch_name))?;

        Ok(local_branch)
    }

    /// Cherry-picks a commit to the current branch
    pub fn cherry_pick(&self, commit_sha: &str) -> Result<CherrypickResult> {
        tracing::info!("Cherry-picking commit: {}", commit_sha);

        let oid = Oid::from_str(commit_sha)
            .with_context(|| format!("Invalid commit SHA: {}", commit_sha))?;

        let commit = self.repo.find_commit(oid)
            .with_context(|| format!("Commit not found: {}", commit_sha))?;

        // Perform the cherry-pick
        let mut opts = CherrypickOptions::new();
        self.repo.cherrypick(&commit, Some(&mut opts))
            .context("Failed to cherry-pick commit")?;

        // Check repository state after cherry-pick
        match self.repo.state() {
            RepositoryState::Clean => {
                // No conflicts, commit the change
                let signature = self.get_signature()?;
                let tree_id = self.repo.index()?.write_tree()?;
                let tree = self.repo.find_tree(tree_id)?;
                let parent = self.repo.head()?.peel_to_commit()?;

                let commit_id = self.repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    &commit.message().unwrap_or("Cherry-pick"),
                    &tree,
                    &[&parent],
                )?;

                tracing::info!("Cherry-pick successful, created commit: {}", commit_id);
                
                Ok(CherrypickResult {
                    success: true,
                    conflicts: Vec::new(),
                    commit_sha: Some(commit_id.to_string()),
                })
            }
            RepositoryState::CherryPickSequence => {
                // There are conflicts
                let conflicts = self.get_conflicts()?;
                tracing::warn!("Cherry-pick has conflicts: {:?}", conflicts);
                
                Ok(CherrypickResult {
                    success: false,
                    conflicts,
                    commit_sha: None,
                })
            }
            state => {
                anyhow::bail!("Unexpected repository state after cherry-pick: {:?}", state);
            }
        }
    }

    fn get_conflicts(&self) -> Result<Vec<String>> {
        let index = self.repo.index()?;
        let mut conflicts = Vec::new();

        if index.has_conflicts() {
            let conflict_iter = index.conflicts()
                .context("Failed to get conflicts iterator")?;

            for conflict in conflict_iter {
                let conflict = conflict?;
                if let Some(our) = conflict.our {
                    let path = String::from_utf8_lossy(&our.path).to_string();
                    conflicts.push(path);
                }
            }
        }

        Ok(conflicts)
    }

    /// Continues cherry-pick after conflicts are resolved
    pub fn continue_cherry_pick(&self, commit_message: Option<&str>) -> Result<String> {
        tracing::info!("Continuing cherry-pick after conflict resolution");

        // Check if conflicts are resolved
        let index = self.repo.index()?;
        if index.has_conflicts() {
            anyhow::bail!("There are still unresolved conflicts. Please resolve them first.");
        }

        // Stage all changes
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        // Create commit
        let signature = self.get_signature()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        let parent = self.repo.head()?.peel_to_commit()?;

        let message = commit_message.unwrap_or("Cherry-pick (resolved conflicts)");
        let commit_id = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent],
        )?;

        // Clean up cherry-pick state
        self.repo.cleanup_state()?;

        tracing::info!("Cherry-pick continued successfully, created commit: {}", commit_id);
        Ok(commit_id.to_string())
    }

    /// Aborts the current cherry-pick operation
    pub fn abort_cherry_pick(&self) -> Result<()> {
        tracing::info!("Aborting cherry-pick");
        
        self.repo.cleanup_state()
            .context("Failed to cleanup cherry-pick state")?;
        
        // Reset to HEAD
        let head = self.repo.head()?.peel_to_commit()?;
        self.repo.reset(head.as_object(), git2::ResetType::Hard, None)
            .context("Failed to reset to HEAD")?;

        tracing::info!("Cherry-pick aborted successfully");
        Ok(())
    }

    fn get_signature(&self) -> Result<Signature<'_>> {
        // Try to get signature from git config
        let config = self.repo.config()
            .context("Failed to get git config")?;

        let name = config.get_string("user.name")
            .context("Git user.name not configured")?;
        let email = config.get_string("user.email")
            .context("Git user.email not configured")?;

        Signature::now(&name, &email)
            .context("Failed to create git signature")
    }

    /// Fetches latest changes from remote
    pub fn fetch(&self) -> Result<()> {
        tracing::info!("Fetching latest changes from remote");

        let mut remote = self.repo.find_remote("origin")
            .context("Failed to find 'origin' remote")?;

        remote.fetch(&[] as &[&str], None, None)
            .context("Failed to fetch from remote")?;

        tracing::info!("Successfully fetched changes from remote");
        Ok(())
    }

    /// Gets the list of commits between two references
    pub fn get_commits_between(&self, from: &str, to: &str) -> Result<Vec<git2::Commit<'_>>> {
        let from_oid = self.repo.revparse_single(from)?.id();
        let to_oid = self.repo.revparse_single(to)?.id();

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(to_oid)?;
        revwalk.hide(from_oid)?;

        let mut commits = Vec::new();
        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            commits.push(commit);
        }

        Ok(commits)
    }
}
