use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process::Command,
};

use relay_core::{ChangeStatus, ChangedFile, WorktreeId, WorktreeSnapshot};

pub mod remote;
pub use remote::{
    ExecutionHostKind, LocalExecutionHost, RemoteCommand, RemoteCommandOutput, RemoteExecutionHost,
};

pub type WorktreeResult<T> = Result<T, WorktreeError>;

#[derive(Debug, thiserror::Error)]
pub enum WorktreeError {
    #[error("git command failed: {command}\n{stderr}")]
    Git { command: String, stderr: String },
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("path is not inside a git repository: {0}")]
    NotRepository(PathBuf),
    #[error("worktree has uncommitted changes: {0}")]
    Dirty(PathBuf),
    #[error("git output was not valid utf-8")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("remote command failed: {command}\n{stderr}")]
    RemoteCommand { command: String, stderr: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRepositoryInfo {
    pub root: PathBuf,
    pub default_base_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitWorktree {
    pub path: PathBuf,
    pub head: Option<String>,
    pub branch: Option<String>,
    pub is_bare: bool,
    pub is_detached: bool,
    pub is_main: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateWorktree {
    pub repo_root: PathBuf,
    pub worktree_path: PathBuf,
    pub branch_name: String,
    pub base_ref: String,
}

pub trait WorktreeProvider {
    fn open_repo(&self, path: &Path) -> WorktreeResult<GitRepositoryInfo>;
    fn list_worktrees(&self, repo_root: &Path) -> WorktreeResult<Vec<GitWorktree>>;
    fn create_worktree(&self, request: &CreateWorktree) -> WorktreeResult<WorktreeSnapshot>;
    fn remove_worktree(
        &self,
        repo_root: &Path,
        worktree_path: &Path,
        force: bool,
    ) -> WorktreeResult<()>;
    fn changed_files(&self, worktree_path: &Path) -> WorktreeResult<Vec<ChangedFile>>;
}

#[derive(Debug, Clone)]
pub struct GitCli {
    program: OsString,
}

impl Default for GitCli {
    fn default() -> Self {
        Self {
            program: OsString::from("git"),
        }
    }
}

impl GitCli {
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
        }
    }

    fn git<I, S>(&self, cwd: Option<&Path>, args: I) -> WorktreeResult<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args: Vec<OsString> = args
            .into_iter()
            .map(|arg| arg.as_ref().to_os_string())
            .collect();
        let mut command = Command::new(&self.program);
        if let Some(cwd) = cwd {
            command.current_dir(cwd);
        }
        command.args(&args);
        let output = command.output()?;
        if output.status.success() {
            return Ok(String::from_utf8(output.stdout)?);
        }

        Err(WorktreeError::Git {
            command: format_command(&self.program, &args),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }

    fn ref_exists(&self, repo_root: &Path, qualified_ref: &str) -> bool {
        self.git(
            Some(repo_root),
            ["show-ref", "--verify", "--quiet", qualified_ref],
        )
        .is_ok()
    }

    fn resolve_worktree_add_base_ref(&self, repo_root: &Path, base_ref: &str) -> String {
        if base_ref.starts_with("refs/") {
            return base_ref.to_string();
        }

        let candidates = if base_ref.contains('/') {
            vec![
                format!("refs/remotes/{base_ref}"),
                format!("refs/heads/{base_ref}"),
            ]
        } else {
            vec![format!("refs/heads/{base_ref}")]
        };

        candidates
            .into_iter()
            .find(|candidate| self.ref_exists(repo_root, candidate))
            .unwrap_or_else(|| base_ref.to_string())
    }

    fn default_base_ref(&self, repo_root: &Path) -> String {
        self.git(
            Some(repo_root),
            [
                "symbolic-ref",
                "--quiet",
                "--short",
                "refs/remotes/origin/HEAD",
            ],
        )
        .map(|value| value.trim().to_string())
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            self.git(
                Some(repo_root),
                ["symbolic-ref", "--quiet", "--short", "HEAD"],
            )
            .map(|value| value.trim().to_string())
            .ok()
            .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "HEAD".to_string())
    }
}

impl WorktreeProvider for GitCli {
    fn open_repo(&self, path: &Path) -> WorktreeResult<GitRepositoryInfo> {
        let root = self
            .git(Some(path), ["rev-parse", "--show-toplevel"])
            .map_err(|_| WorktreeError::NotRepository(path.to_path_buf()))?;
        let root = PathBuf::from(root.trim());
        let default_base_ref = self.default_base_ref(&root);

        Ok(GitRepositoryInfo {
            root,
            default_base_ref,
        })
    }

    fn list_worktrees(&self, repo_root: &Path) -> WorktreeResult<Vec<GitWorktree>> {
        let output = self.git(Some(repo_root), ["worktree", "list", "--porcelain"])?;
        Ok(parse_worktree_list(&output))
    }

    fn create_worktree(&self, request: &CreateWorktree) -> WorktreeResult<WorktreeSnapshot> {
        if let Some(parent) = request.worktree_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let base_ref = self.resolve_worktree_add_base_ref(&request.repo_root, &request.base_ref);
        self.git(
            Some(&request.repo_root),
            [
                OsString::from("worktree"),
                OsString::from("add"),
                OsString::from("-b"),
                OsString::from(&request.branch_name),
                OsString::from("--"),
                request.worktree_path.as_os_str().to_os_string(),
                OsString::from(&base_ref),
            ],
        )?;

        Ok(WorktreeSnapshot {
            id: WorktreeId::new(),
            path: request.worktree_path.to_string_lossy().to_string(),
            branch: request.branch_name.clone(),
            base_ref: Some(base_ref),
        })
    }

    fn remove_worktree(
        &self,
        repo_root: &Path,
        worktree_path: &Path,
        force: bool,
    ) -> WorktreeResult<()> {
        if !force && !self.changed_files(worktree_path)?.is_empty() {
            return Err(WorktreeError::Dirty(worktree_path.to_path_buf()));
        }

        let mut args = vec![OsString::from("worktree"), OsString::from("remove")];
        if force {
            args.push(OsString::from("--force"));
        }
        args.push(OsString::from("--"));
        args.push(worktree_path.as_os_str().to_os_string());
        self.git(Some(repo_root), args)?;
        Ok(())
    }

    fn changed_files(&self, worktree_path: &Path) -> WorktreeResult<Vec<ChangedFile>> {
        let output = self.git(Some(worktree_path), ["status", "--porcelain=v1"])?;
        Ok(parse_changed_files(&output))
    }
}

pub fn sanitize_branch_component(input: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in input.chars().flat_map(char::to_lowercase) {
        let is_word = ch.is_ascii_alphanumeric();
        if is_word {
            slug.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "task".to_string()
    } else {
        slug.to_string()
    }
}

fn parse_worktree_list(output: &str) -> Vec<GitWorktree> {
    output
        .split("\n\n")
        .filter_map(|block| {
            let mut path = None;
            let mut head = None;
            let mut branch = None;
            let mut is_bare = false;
            let mut is_detached = false;

            for line in block.lines() {
                if let Some(value) = line.strip_prefix("worktree ") {
                    path = Some(PathBuf::from(value));
                } else if let Some(value) = line.strip_prefix("HEAD ") {
                    head = Some(value.to_string());
                } else if let Some(value) = line.strip_prefix("branch ") {
                    branch = Some(value.trim_start_matches("refs/heads/").to_string());
                } else if line == "bare" {
                    is_bare = true;
                } else if line == "detached" {
                    is_detached = true;
                }
            }

            path.map(|path| GitWorktree {
                path,
                head,
                branch,
                is_bare,
                is_detached,
                is_main: false,
            })
        })
        .enumerate()
        .map(|(index, mut worktree)| {
            worktree.is_main = index == 0;
            worktree
        })
        .collect()
}

fn parse_changed_files(output: &str) -> Vec<ChangedFile> {
    output
        .lines()
        .filter_map(|line| {
            if line.len() < 4 {
                return None;
            }

            let status = match &line[0..2] {
                "??" => ChangeStatus::Untracked,
                value if value.contains('A') => ChangeStatus::Added,
                value if value.contains('D') => ChangeStatus::Deleted,
                value if value.contains('R') => ChangeStatus::Renamed,
                _ => ChangeStatus::Modified,
            };
            let raw_path = &line[3..];
            let path = raw_path
                .rsplit_once(" -> ")
                .map(|(_, new_path)| new_path)
                .unwrap_or(raw_path)
                .to_string();

            Some(ChangedFile { path, status })
        })
        .collect()
}

fn format_command(program: &OsStr, args: &[OsString]) -> String {
    std::iter::once(program.to_string_lossy().to_string())
        .chain(args.iter().map(|arg| arg.to_string_lossy().to_string()))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path, process::Command};

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn sanitize_branch_component_should_produce_git_safe_slug() {
        assert_eq!(
            sanitize_branch_component("Fix: GPUI shell / PTY!"),
            "fix-gpui-shell-pty"
        );
        assert_eq!(sanitize_branch_component("!!!"), "task");
    }

    #[test]
    fn git_cli_should_create_detect_status_and_remove_worktree() {
        let temp = tempdir().expect("tempdir should exist");
        let repo = temp.path().join("repo");
        let worktrees = temp.path().join("worktrees");
        fs::create_dir_all(&repo).expect("repo dir should exist");
        git(&repo, ["init", "-b", "main"]);
        git(&repo, ["config", "user.email", "relay@example.com"]);
        git(&repo, ["config", "user.name", "Relay Test"]);
        fs::write(repo.join("README.md"), "hello\n").expect("file should write");
        git(&repo, ["add", "."]);
        git(&repo, ["commit", "-m", "initial"]);

        let provider = GitCli::default();
        let info = provider.open_repo(&repo).expect("repo should open");
        let worktree_path = worktrees.join("task-one");
        let snapshot = provider
            .create_worktree(&CreateWorktree {
                repo_root: info.root.clone(),
                worktree_path: worktree_path.clone(),
                branch_name: "relay/task-one".to_string(),
                base_ref: info.default_base_ref,
            })
            .expect("worktree should create");

        assert_eq!(snapshot.branch, "relay/task-one");
        assert!(worktree_path.exists());

        fs::write(worktree_path.join("new.txt"), "changed\n").expect("file should write");
        let changed = provider
            .changed_files(&worktree_path)
            .expect("changed files should load");
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].status, ChangeStatus::Untracked);

        let dirty_error = provider
            .remove_worktree(&info.root, &worktree_path, false)
            .expect_err("dirty worktree should be blocked");
        assert!(matches!(dirty_error, WorktreeError::Dirty(_)));

        provider
            .remove_worktree(&info.root, &worktree_path, true)
            .expect("force remove should succeed");
        assert!(!worktree_path.exists());
    }

    fn git<const N: usize>(cwd: &Path, args: [&str; N]) {
        let output = Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .expect("git command should run");
        assert!(
            output.status.success(),
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
