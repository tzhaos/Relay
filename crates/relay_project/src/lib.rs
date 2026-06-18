use std::path::{Path, PathBuf};

use relay_core::{ChangedFile, ProjectId, WorktreeSnapshot};
use relay_worktree::{
    CreateWorktree, GitCli, WorktreeError, WorktreeProvider, sanitize_branch_component,
};

pub type ProjectResult<T> = Result<T, ProjectError>;

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("worktree error")]
    Worktree(#[from] WorktreeError),
    #[error("project path has no usable display name: {0}")]
    MissingDisplayName(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: ProjectId,
    pub root: PathBuf,
    pub display_name: String,
    pub default_base_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTaskWorktree {
    pub task_title: String,
    pub worktrees_dir: PathBuf,
    pub branch_prefix: String,
}

pub struct ProjectService<P = GitCli> {
    provider: P,
}

impl Default for ProjectService<GitCli> {
    fn default() -> Self {
        Self {
            provider: GitCli::default(),
        }
    }
}

impl<P> ProjectService<P>
where
    P: WorktreeProvider,
{
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub fn open_repo(&self, path: &Path) -> ProjectResult<Project> {
        let repo = self.provider.open_repo(path)?;
        let display_name = repo
            .root
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .ok_or_else(|| ProjectError::MissingDisplayName(repo.root.clone()))?;

        Ok(Project {
            id: ProjectId::new(),
            root: repo.root,
            display_name,
            default_base_ref: repo.default_base_ref,
        })
    }

    pub fn create_task_worktree(
        &self,
        project: &Project,
        request: &CreateTaskWorktree,
    ) -> ProjectResult<WorktreeSnapshot> {
        let slug = sanitize_branch_component(&request.task_title);
        let branch_prefix = request.branch_prefix.trim_matches('/');
        let branch_name = if branch_prefix.is_empty() {
            slug.clone()
        } else {
            format!("{branch_prefix}/{slug}")
        };
        let worktree_path = request.worktrees_dir.join(&slug);

        Ok(self.provider.create_worktree(&CreateWorktree {
            repo_root: project.root.clone(),
            worktree_path,
            branch_name,
            base_ref: project.default_base_ref.clone(),
        })?)
    }

    pub fn changed_files(&self, worktree_path: &Path) -> ProjectResult<Vec<ChangedFile>> {
        Ok(self.provider.changed_files(worktree_path)?)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::Path};

    use relay_worktree::{GitRepositoryInfo, GitWorktree, WorktreeResult};

    use super::*;

    #[derive(Default)]
    struct FakeProvider {
        create_requests: RefCell<Vec<CreateWorktree>>,
    }

    impl WorktreeProvider for FakeProvider {
        fn open_repo(&self, path: &Path) -> WorktreeResult<GitRepositoryInfo> {
            Ok(GitRepositoryInfo {
                root: path.to_path_buf(),
                default_base_ref: "main".to_string(),
            })
        }

        fn list_worktrees(&self, _repo_root: &Path) -> WorktreeResult<Vec<GitWorktree>> {
            Ok(Vec::new())
        }

        fn create_worktree(&self, request: &CreateWorktree) -> WorktreeResult<WorktreeSnapshot> {
            self.create_requests.borrow_mut().push(request.clone());
            Ok(WorktreeSnapshot {
                id: relay_core::WorktreeId::new(),
                path: request.worktree_path.to_string_lossy().to_string(),
                branch: request.branch_name.clone(),
                base_ref: Some(request.base_ref.clone()),
            })
        }

        fn remove_worktree(
            &self,
            _repo_root: &Path,
            _worktree_path: &Path,
            _force: bool,
        ) -> WorktreeResult<()> {
            Ok(())
        }

        fn changed_files(
            &self,
            _worktree_path: &Path,
        ) -> WorktreeResult<Vec<relay_core::ChangedFile>> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn project_service_should_create_branch_and_path_from_task_title() {
        let provider = FakeProvider::default();
        let service = ProjectService::new(provider);
        let project = service
            .open_repo(Path::new("F:/Workspace/Relay"))
            .expect("repo should open");
        let snapshot = service
            .create_task_worktree(
                &project,
                &CreateTaskWorktree {
                    task_title: "Build PTY Runtime!".to_string(),
                    worktrees_dir: PathBuf::from("F:/Workspace/.relay-worktrees"),
                    branch_prefix: "relay".to_string(),
                },
            )
            .expect("worktree should create");

        assert_eq!(snapshot.branch, "relay/build-pty-runtime");
        assert!(snapshot.path.ends_with("build-pty-runtime"));
    }
}
