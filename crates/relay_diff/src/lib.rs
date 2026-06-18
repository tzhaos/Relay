use std::{
    collections::{BTreeMap, HashSet},
    ffi::{OsStr, OsString},
    path::Path,
    process::Command,
};

use relay_core::{
    ChangeStatus, ChangedFile, DiffReview, DiffSide, LineIdentity, ReviewComment, ReviewCommentId,
    SelectedRange, Task, TaskCommand, TaskId, Timestamp,
};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

pub type DiffResult<T> = Result<T, DiffError>;

#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    #[error("git diff command failed: {command}\n{stderr}")]
    Git { command: String, stderr: String },
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("diff output was not valid utf-8")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("review comment body cannot be empty")]
    EmptyComment,
    #[error("review comment path cannot be empty")]
    EmptyPath,
    #[error("task has no active agent session")]
    MissingAgent,
    #[error("there are no pending review comments")]
    NoPendingComments,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffFile {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub status: ChangeStatus,
    pub hunks: Vec<DiffHunk>,
    pub is_binary: bool,
}

impl DiffFile {
    pub fn display_path(&self) -> &str {
        self.new_path
            .as_deref()
            .or(self.old_path.as_deref())
            .unwrap_or("<unknown>")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffLineKind {
    Context,
    Added,
    Deleted,
    NoNewline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub content: String,
    pub identity: LineIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffSnapshot {
    pub files: Vec<DiffFile>,
    pub tree: DiffTree,
    pub stats: DiffStats,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffStats {
    pub files_changed: usize,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone)]
pub struct DiffEngine {
    git_program: OsString,
}

impl Default for DiffEngine {
    fn default() -> Self {
        Self {
            git_program: OsString::from("git"),
        }
    }
}

impl DiffEngine {
    pub fn new(git_program: impl Into<OsString>) -> Self {
        Self {
            git_program: git_program.into(),
        }
    }

    pub fn load(&self, worktree_path: &Path, base_ref: Option<&str>) -> DiffResult<DiffSnapshot> {
        let base = base_ref.unwrap_or("HEAD");
        let diff_text = self.git(
            worktree_path,
            [
                OsString::from("diff"),
                OsString::from("--no-ext-diff"),
                OsString::from("--no-color"),
                OsString::from("--find-renames"),
                OsString::from("--unified=3"),
                OsString::from(base),
                OsString::from("--"),
            ],
        )?;
        let mut files = parse_unified_diff(&diff_text);
        let untracked = self.untracked_files(worktree_path)?;
        files.extend(self.diff_untracked_files(worktree_path, &untracked)?);

        Ok(snapshot_from_files(files))
    }

    pub fn from_texts(path: impl Into<String>, old: &str, new: &str) -> DiffSnapshot {
        let path = path.into();
        let text_diff = TextDiff::from_lines(old, new);
        let mut old_line = 1_u32;
        let mut new_line = 1_u32;
        let mut lines = Vec::new();

        for change in text_diff.iter_all_changes() {
            let (kind, current_old, current_new, side) = match change.tag() {
                ChangeTag::Delete => {
                    let current = old_line;
                    old_line += 1;
                    (DiffLineKind::Deleted, Some(current), None, DiffSide::Old)
                }
                ChangeTag::Insert => {
                    let current = new_line;
                    new_line += 1;
                    (DiffLineKind::Added, None, Some(current), DiffSide::New)
                }
                ChangeTag::Equal => {
                    let current_old = old_line;
                    let current_new = new_line;
                    old_line += 1;
                    new_line += 1;
                    (
                        DiffLineKind::Context,
                        Some(current_old),
                        Some(current_new),
                        DiffSide::New,
                    )
                }
            };
            lines.push(DiffLine {
                kind,
                old_line: current_old,
                new_line: current_new,
                content: change.to_string(),
                identity: LineIdentity {
                    path: path.clone(),
                    side,
                    old_line: current_old,
                    new_line: current_new,
                    hunk_header: "@@ synthetic @@".to_string(),
                },
            });
        }

        snapshot_from_files(vec![DiffFile {
            old_path: Some(path.clone()),
            new_path: Some(path),
            status: ChangeStatus::Modified,
            hunks: vec![DiffHunk {
                old_start: 1,
                old_lines: old.lines().count() as u32,
                new_start: 1,
                new_lines: new.lines().count() as u32,
                header: "@@ synthetic @@".to_string(),
                lines,
            }],
            is_binary: false,
        }])
    }

    fn untracked_files(&self, worktree_path: &Path) -> DiffResult<Vec<String>> {
        let output = self.git(
            worktree_path,
            [
                OsString::from("ls-files"),
                OsString::from("--others"),
                OsString::from("--exclude-standard"),
            ],
        )?;
        Ok(output
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect())
    }

    fn diff_untracked_files(
        &self,
        worktree_path: &Path,
        paths: &[String],
    ) -> DiffResult<Vec<DiffFile>> {
        paths
            .iter()
            .map(|path| {
                let full_path = worktree_path.join(path);
                let content = std::fs::read_to_string(&full_path)?;
                let snapshot = Self::from_texts(path.clone(), "", &content);
                let mut file = snapshot
                    .files
                    .into_iter()
                    .next()
                    .ok_or(DiffError::NoPendingComments)?;
                file.old_path = None;
                file.status = ChangeStatus::Untracked;
                Ok(file)
            })
            .collect()
    }

    fn git<I, S>(&self, cwd: &Path, args: I) -> DiffResult<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args: Vec<OsString> = args
            .into_iter()
            .map(|arg| arg.as_ref().to_os_string())
            .collect();
        let output = Command::new(&self.git_program)
            .current_dir(cwd)
            .args(&args)
            .output()?;
        if output.status.success() {
            return Ok(String::from_utf8(output.stdout)?);
        }

        Err(DiffError::Git {
            command: format_command(&self.git_program, &args),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewDelivery {
    pub task_id: TaskId,
    pub comment_ids: Vec<ReviewCommentId>,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitMessageDraft {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ReviewService;

impl ReviewService {
    pub fn add_comment(
        task_id: TaskId,
        path: impl Into<String>,
        body: impl Into<String>,
        selected_range: Option<SelectedRange>,
        now: Timestamp,
    ) -> DiffResult<ReviewComment> {
        let path = path.into();
        let body = body.into();
        if path.trim().is_empty() {
            return Err(DiffError::EmptyPath);
        }
        if body.trim().is_empty() {
            return Err(DiffError::EmptyComment);
        }
        let line = selected_range
            .as_ref()
            .map(|range| Box::new(range.start.clone()));

        Ok(ReviewComment {
            id: ReviewCommentId::new(),
            task_id,
            path,
            line,
            selected_range: selected_range.map(Box::new),
            body: body.trim().to_string(),
            created_at: now,
        })
    }

    pub fn pending_comments(review: &DiffReview) -> Vec<&ReviewComment> {
        let delivered: HashSet<_> = review
            .delivered_comments
            .iter()
            .map(|entry| entry.id)
            .collect();
        review
            .comments
            .iter()
            .filter(|comment| !delivered.contains(&comment.id))
            .collect()
    }

    pub fn deliver_comments_to_agent(task: &Task) -> DiffResult<ReviewDelivery> {
        if task.agent_session_id.is_none() {
            return Err(DiffError::MissingAgent);
        }
        let pending = Self::pending_comments(&task.diff_review);
        if pending.is_empty() {
            return Err(DiffError::NoPendingComments);
        }

        let mut prompt = format!(
            "Relay review notes for task \"{}\".\nPlease address these diff comments, update the worktree, then summarize what changed.\n",
            task.title
        );
        for (index, comment) in pending.iter().enumerate() {
            prompt.push('\n');
            prompt.push_str(&format!(
                "{}. {}\n",
                index + 1,
                format_comment_location(comment)
            ));
            prompt.push_str(&format!("   {}\n", comment.body));
            if let Some(range) = &comment.selected_range
                && let Some(selected_text) = &range.selected_text
            {
                prompt.push_str(&format!("   selected: {}\n", selected_text.trim()));
            }
        }

        Ok(ReviewDelivery {
            task_id: task.id,
            comment_ids: pending.into_iter().map(|comment| comment.id).collect(),
            prompt,
        })
    }

    pub fn mark_delivered(delivery: &ReviewDelivery, now: Timestamp) -> TaskCommand {
        TaskCommand::MarkReviewDelivered {
            comment_ids: delivery.comment_ids.clone(),
            now,
        }
    }

    pub fn draft_commit_message(task: &Task, snapshot: &DiffSnapshot) -> CommitMessageDraft {
        let first_file = snapshot.files.first().map(DiffFile::display_path);
        let title = match (snapshot.files.len(), first_file) {
            (0, _) => format!("Update {}", task.title),
            (1, Some(path)) => format!("Update {path}"),
            (count, _) => format!("Update {} files for {}", count, task.title),
        };
        let pending_count = Self::pending_comments(&task.diff_review).len();
        let body = format!(
            "Files changed: {}\nAdditions: {}\nDeletions: {}\nReview notes addressed: {}",
            snapshot.stats.files_changed,
            snapshot.stats.additions,
            snapshot.stats.deletions,
            task.diff_review
                .comments
                .len()
                .saturating_sub(pending_count)
        );

        CommitMessageDraft { title, body }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffTree {
    pub rows: Vec<DiffTreeRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffTreeRow {
    pub path: String,
    pub label: String,
    pub depth: usize,
    pub kind: DiffTreeRowKind,
    pub status: Option<ChangeStatus>,
    pub file_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffTreeRowKind {
    Directory,
    File,
}

#[derive(Default)]
struct TreeNode {
    name: String,
    path: String,
    children: BTreeMap<String, TreeNode>,
    files: Vec<ChangedFile>,
}

impl DiffTree {
    pub fn from_changed_files(files: &[ChangedFile]) -> Self {
        let mut root = TreeNode::default();
        for file in files {
            let parts: Vec<_> = file
                .path
                .split('/')
                .filter(|part| !part.is_empty())
                .collect();
            if parts.is_empty() {
                continue;
            }
            let mut node = &mut root;
            let mut path = String::new();
            for (index, part) in parts.iter().enumerate() {
                if index == parts.len() - 1 {
                    node.files.push(file.clone());
                } else {
                    if !path.is_empty() {
                        path.push('/');
                    }
                    path.push_str(part);
                    node = node
                        .children
                        .entry((*part).to_string())
                        .or_insert_with(|| TreeNode {
                            name: (*part).to_string(),
                            path: path.clone(),
                            ..TreeNode::default()
                        });
                }
            }
        }

        let mut rows = Vec::new();
        flatten_tree(&root, 0, &mut rows);
        Self { rows }
    }
}

pub fn parse_unified_diff(input: &str) -> Vec<DiffFile> {
    let mut files = Vec::new();
    let mut current_file: Option<DiffFile> = None;
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_line = 0_u32;
    let mut new_line = 0_u32;

    for line in input.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            push_hunk(&mut current_file, &mut current_hunk);
            if let Some(file) = current_file.take() {
                files.push(file);
            }
            let (old_path, new_path) = parse_diff_git_paths(rest);
            current_file = Some(DiffFile {
                old_path,
                new_path,
                status: ChangeStatus::Modified,
                hunks: Vec::new(),
                is_binary: false,
            });
            continue;
        }

        let Some(file) = current_file.as_mut() else {
            continue;
        };

        if line.starts_with("Binary files ") {
            file.is_binary = true;
            continue;
        }
        if line == "new file mode" || line.starts_with("new file mode ") {
            file.status = ChangeStatus::Added;
            continue;
        }
        if line == "deleted file mode" || line.starts_with("deleted file mode ") {
            file.status = ChangeStatus::Deleted;
            continue;
        }
        if let Some(path) = line.strip_prefix("rename from ") {
            file.old_path = Some(path.to_string());
            file.status = ChangeStatus::Renamed;
            continue;
        }
        if let Some(path) = line.strip_prefix("rename to ") {
            file.new_path = Some(path.to_string());
            file.status = ChangeStatus::Renamed;
            continue;
        }
        if let Some(path) = line.strip_prefix("--- ") {
            file.old_path = normalize_diff_path(path);
            continue;
        }
        if let Some(path) = line.strip_prefix("+++ ") {
            file.new_path = normalize_diff_path(path);
            continue;
        }
        if line.starts_with("@@ ") {
            push_hunk(&mut current_file, &mut current_hunk);
            if let Some((old_start, old_lines, new_start, new_lines)) = parse_hunk_header(line) {
                old_line = old_start;
                new_line = new_start;
                current_hunk = Some(DiffHunk {
                    old_start,
                    old_lines,
                    new_start,
                    new_lines,
                    header: line.to_string(),
                    lines: Vec::new(),
                });
            }
            continue;
        }

        let Some(hunk) = current_hunk.as_mut() else {
            continue;
        };
        let path = file.display_path().to_string();
        match line.as_bytes().first().copied() {
            Some(b' ') => {
                let content = line[1..].to_string();
                hunk.lines.push(DiffLine {
                    kind: DiffLineKind::Context,
                    old_line: Some(old_line),
                    new_line: Some(new_line),
                    content,
                    identity: LineIdentity {
                        path,
                        side: DiffSide::New,
                        old_line: Some(old_line),
                        new_line: Some(new_line),
                        hunk_header: hunk.header.clone(),
                    },
                });
                old_line += 1;
                new_line += 1;
            }
            Some(b'+') => {
                let content = line[1..].to_string();
                hunk.lines.push(DiffLine {
                    kind: DiffLineKind::Added,
                    old_line: None,
                    new_line: Some(new_line),
                    content,
                    identity: LineIdentity {
                        path,
                        side: DiffSide::New,
                        old_line: None,
                        new_line: Some(new_line),
                        hunk_header: hunk.header.clone(),
                    },
                });
                new_line += 1;
            }
            Some(b'-') => {
                let content = line[1..].to_string();
                hunk.lines.push(DiffLine {
                    kind: DiffLineKind::Deleted,
                    old_line: Some(old_line),
                    new_line: None,
                    content,
                    identity: LineIdentity {
                        path,
                        side: DiffSide::Old,
                        old_line: Some(old_line),
                        new_line: None,
                        hunk_header: hunk.header.clone(),
                    },
                });
                old_line += 1;
            }
            Some(b'\\') => hunk.lines.push(DiffLine {
                kind: DiffLineKind::NoNewline,
                old_line: None,
                new_line: None,
                content: line.to_string(),
                identity: LineIdentity {
                    path,
                    side: DiffSide::New,
                    old_line: None,
                    new_line: None,
                    hunk_header: hunk.header.clone(),
                },
            }),
            _ => {}
        }
    }

    push_hunk(&mut current_file, &mut current_hunk);
    if let Some(file) = current_file {
        files.push(file);
    }
    files
}

fn snapshot_from_files(files: Vec<DiffFile>) -> DiffSnapshot {
    let changed_files = files
        .iter()
        .map(|file| ChangedFile {
            path: file.display_path().to_string(),
            status: file.status,
        })
        .collect::<Vec<_>>();
    let additions = files
        .iter()
        .flat_map(|file| &file.hunks)
        .flat_map(|hunk| &hunk.lines)
        .filter(|line| line.kind == DiffLineKind::Added)
        .count();
    let deletions = files
        .iter()
        .flat_map(|file| &file.hunks)
        .flat_map(|hunk| &hunk.lines)
        .filter(|line| line.kind == DiffLineKind::Deleted)
        .count();

    DiffSnapshot {
        stats: DiffStats {
            files_changed: files.len(),
            additions,
            deletions,
        },
        tree: DiffTree::from_changed_files(&changed_files),
        files,
    }
}

fn push_hunk(file: &mut Option<DiffFile>, hunk: &mut Option<DiffHunk>) {
    if let (Some(file), Some(hunk)) = (file.as_mut(), hunk.take()) {
        file.hunks.push(hunk);
    }
}

fn parse_diff_git_paths(rest: &str) -> (Option<String>, Option<String>) {
    let mut parts = rest.split_whitespace();
    let old_path = parts.next().and_then(normalize_diff_path);
    let new_path = parts.next().and_then(normalize_diff_path);
    (old_path, new_path)
}

fn normalize_diff_path(path: &str) -> Option<String> {
    let trimmed = path.trim();
    if trimmed == "/dev/null" {
        return None;
    }
    trimmed
        .strip_prefix("a/")
        .or_else(|| trimmed.strip_prefix("b/"))
        .or(Some(trimmed))
        .map(ToOwned::to_owned)
}

fn parse_hunk_header(line: &str) -> Option<(u32, u32, u32, u32)> {
    let mut parts = line.split_whitespace();
    let old = parts.nth(1)?;
    let new = parts.next()?;
    let (old_start, old_lines) = parse_hunk_span(old.strip_prefix('-')?)?;
    let (new_start, new_lines) = parse_hunk_span(new.strip_prefix('+')?)?;
    Some((old_start, old_lines, new_start, new_lines))
}

fn parse_hunk_span(span: &str) -> Option<(u32, u32)> {
    let (start, count) = span.split_once(',').unwrap_or((span, "1"));
    Some((start.parse().ok()?, count.parse().ok()?))
}

fn flatten_tree(node: &TreeNode, depth: usize, rows: &mut Vec<DiffTreeRow>) -> usize {
    let mut file_count = 0;
    for child in node.children.values() {
        let (terminal, label) = compact_directory_chain(child);
        let row_index = rows.len();
        rows.push(DiffTreeRow {
            path: terminal.path.clone(),
            label,
            depth,
            kind: DiffTreeRowKind::Directory,
            status: None,
            file_count: 0,
        });
        let child_count = flatten_tree(terminal, depth + 1, rows);
        rows[row_index].file_count = child_count;
        file_count += child_count;
    }

    for file in &node.files {
        rows.push(DiffTreeRow {
            path: file.path.clone(),
            label: file
                .path
                .rsplit('/')
                .next()
                .unwrap_or(file.path.as_str())
                .to_string(),
            depth,
            kind: DiffTreeRowKind::File,
            status: Some(file.status),
            file_count: 1,
        });
        file_count += 1;
    }

    file_count
}

fn compact_directory_chain(mut node: &TreeNode) -> (&TreeNode, String) {
    let mut parts = vec![node.name.clone()];
    while node.files.is_empty() && node.children.len() == 1 {
        let Some(child) = node.children.values().next() else {
            break;
        };
        parts.push(child.name.clone());
        node = child;
    }
    (node, parts.join("/"))
}

fn format_comment_location(comment: &ReviewComment) -> String {
    let Some(line) = &comment.line else {
        return comment.path.clone();
    };
    let line_number = match line.side {
        DiffSide::Old => line.old_line,
        DiffSide::New => line.new_line,
    };
    line_number
        .map(|number| format!("{}:{} ({:?})", comment.path, number, line.side))
        .unwrap_or_else(|| comment.path.clone())
}

fn format_command(program: &OsStr, args: &[OsString]) -> String {
    std::iter::once(program.to_string_lossy().to_string())
        .chain(args.iter().map(|arg| arg.to_string_lossy().to_string()))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use relay_core::{
        AgentKind, AgentSessionId, ChangeStatus, CreateTask, ProjectId, TaskSource,
        TerminalSessionId, WorktreeId, WorktreeSnapshot,
    };

    use super::*;

    fn now() -> Timestamp {
        Timestamp::UNIX_EPOCH
    }

    #[test]
    fn parse_unified_diff_should_keep_hunk_line_identity() {
        let diff = "\
diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,3 +1,4 @@
 fn old() {
-    one();
+    two();
+    three();
 }
";

        let files = parse_unified_diff(diff);

        assert_eq!(files[0].hunks[0].lines[1].identity.old_line, Some(2));
        assert_eq!(files[0].hunks[0].lines[2].identity.new_line, Some(2));
    }

    #[test]
    fn diff_tree_should_compact_single_child_directories() {
        let tree = DiffTree::from_changed_files(&[
            ChangedFile {
                path: "crates/relay_diff/src/lib.rs".to_string(),
                status: ChangeStatus::Added,
            },
            ChangedFile {
                path: "crates/relay_core/src/task.rs".to_string(),
                status: ChangeStatus::Modified,
            },
        ]);

        assert_eq!(tree.rows[0].label, "crates");
    }

    #[test]
    fn review_service_should_format_pending_delivery_and_mark_delivered() {
        let mut task = demo_task();
        let selected_range = SelectedRange {
            start: LineIdentity {
                path: "src/lib.rs".to_string(),
                side: DiffSide::New,
                old_line: None,
                new_line: Some(8),
                hunk_header: "@@ -7,1 +8,1 @@".to_string(),
            },
            end: LineIdentity {
                path: "src/lib.rs".to_string(),
                side: DiffSide::New,
                old_line: None,
                new_line: Some(8),
                hunk_header: "@@ -7,1 +8,1 @@".to_string(),
            },
            selected_text: Some("new line".to_string()),
        };
        let comment = ReviewService::add_comment(
            task.id,
            "src/lib.rs",
            "Tighten the error handling.",
            Some(selected_range),
            now(),
        )
        .expect("comment should be valid");
        apply(&mut task, TaskCommand::AddReviewComment(comment));

        let delivery =
            ReviewService::deliver_comments_to_agent(&task).expect("delivery should build");
        let command = ReviewService::mark_delivered(&delivery, now());

        assert!(matches!(command, TaskCommand::MarkReviewDelivered { .. }));
    }

    #[test]
    fn commit_message_draft_should_summarize_diff_stats() {
        let task = demo_task();
        let snapshot = DiffEngine::from_texts("src/lib.rs", "old\n", "new\nextra\n");
        let draft = ReviewService::draft_commit_message(&task, &snapshot);

        assert!(draft.body.contains("Additions: 2"));
    }

    fn demo_task() -> Task {
        let (mut task, _) = Task::create(CreateTask {
            id: None,
            project_id: ProjectId::new(),
            title: "Review diff".to_string(),
            source: TaskSource::Manual,
            now: now(),
        })
        .expect("task should create");
        apply(
            &mut task,
            TaskCommand::AttachWorktree {
                snapshot: WorktreeSnapshot {
                    id: WorktreeId::new(),
                    path: "/repo/task".to_string(),
                    branch: "task/review".to_string(),
                    base_ref: Some("main".to_string()),
                },
                now: now(),
            },
        );
        apply(
            &mut task,
            TaskCommand::AttachTerminal {
                id: TerminalSessionId::new(),
                now: now(),
            },
        );
        apply(
            &mut task,
            TaskCommand::AttachAgent {
                id: AgentSessionId::new(),
                kind: AgentKind::Codex,
                started_at: now(),
            },
        );
        task
    }

    fn apply(task: &mut Task, command: TaskCommand) {
        for event in task.handle(command).expect("command should be valid") {
            task.apply(&event).expect("event should apply");
        }
    }
}
