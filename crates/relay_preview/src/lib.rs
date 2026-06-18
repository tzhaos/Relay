use regex::Regex;
use relay_core::{PreviewTarget, PreviewTargetId, TaskCommand, TaskId, Timestamp};
use serde::{Deserialize, Serialize};

pub type PreviewResult<T> = Result<T, PreviewError>;

#[derive(Debug, thiserror::Error)]
pub enum PreviewError {
    #[error("preview label cannot be empty")]
    EmptyLabel,
    #[error("preview uri cannot be empty")]
    EmptyUri,
    #[error("unsupported preview uri: {0}")]
    UnsupportedUri(String),
    #[error("failed to open preview target {uri}")]
    OpenFailed {
        uri: String,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewRequest {
    pub label: String,
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviewTargetKind {
    Localhost,
    File,
}

pub trait PreviewProvider {
    fn attach_target(
        &self,
        _task_id: TaskId,
        request: PreviewRequest,
        now: Timestamp,
    ) -> PreviewResult<TaskCommand>;
}

pub trait PreviewOpener {
    fn open_target(&mut self, uri: &str) -> PreviewResult<()>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LocalPreviewProvider;

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemPreviewOpener;

impl PreviewProvider for LocalPreviewProvider {
    fn attach_target(
        &self,
        _task_id: TaskId,
        request: PreviewRequest,
        now: Timestamp,
    ) -> PreviewResult<TaskCommand> {
        let label = request.label.trim();
        let uri = request.uri.trim();
        if label.is_empty() {
            return Err(PreviewError::EmptyLabel);
        }
        if uri.is_empty() {
            return Err(PreviewError::EmptyUri);
        }
        classify_target(uri)?;

        Ok(TaskCommand::AttachPreview {
            target: PreviewTarget {
                id: PreviewTargetId::new(),
                label: label.to_string(),
                uri: uri.to_string(),
            },
            now,
        })
    }
}

impl PreviewOpener for SystemPreviewOpener {
    fn open_target(&mut self, uri: &str) -> PreviewResult<()> {
        classify_target(uri)?;
        open::that(uri).map_err(|source| PreviewError::OpenFailed {
            uri: uri.to_string(),
            source,
        })
    }
}

pub fn localhost_url(port: u16, path: &str) -> String {
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    format!("http://localhost:{port}{path}")
}

pub fn classify_target(uri: &str) -> PreviewResult<PreviewTargetKind> {
    let lower = uri.to_ascii_lowercase();
    if lower.starts_with("file://") {
        return Ok(PreviewTargetKind::File);
    }
    if lower.starts_with("http://localhost:")
        || lower.starts_with("https://localhost:")
        || lower.starts_with("http://127.0.0.1:")
        || lower.starts_with("https://127.0.0.1:")
        || lower.starts_with("http://[::1]:")
        || lower.starts_with("https://[::1]:")
    {
        return Ok(PreviewTargetKind::Localhost);
    }
    Err(PreviewError::UnsupportedUri(uri.to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserAttribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserContextInput {
    pub url: String,
    pub title: Option<String>,
    pub selected_text: Option<String>,
    pub dom_path: Option<String>,
    pub attributes: Vec<BrowserAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserContextPayload {
    pub url: String,
    pub title: Option<String>,
    pub selected_text: Option<String>,
    pub dom_path: Option<String>,
    pub attributes: Vec<BrowserAttribute>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SanitizerBudget {
    pub max_text_chars: usize,
    pub max_attr_value_chars: usize,
    pub max_attributes: usize,
}

impl Default for SanitizerBudget {
    fn default() -> Self {
        Self {
            max_text_chars: 4_000,
            max_attr_value_chars: 240,
            max_attributes: 24,
        }
    }
}

impl BrowserContextPayload {
    pub fn sanitize(input: BrowserContextInput, budget: SanitizerBudget) -> Self {
        let secret_pattern =
            Regex::new("(?i)(password|passwd|token|api[_-]?key|secret|authorization)")
                .expect("static redaction regex should compile");
        let mut truncated = false;
        let selected_text = input.selected_text.map(|text| {
            let redacted = redact(&secret_pattern, &text);
            truncate_chars(&redacted, budget.max_text_chars, &mut truncated)
        });
        let title = input.title.map(|title| {
            let redacted = redact(&secret_pattern, &title);
            truncate_chars(&redacted, 160, &mut truncated)
        });
        let dom_path = input
            .dom_path
            .map(|path| truncate_chars(&path, 320, &mut truncated));
        let attributes = input
            .attributes
            .into_iter()
            .filter(|attribute| !secret_pattern.is_match(&attribute.name))
            .take(budget.max_attributes)
            .map(|attribute| BrowserAttribute {
                name: truncate_chars(&attribute.name, 80, &mut truncated),
                value: truncate_chars(
                    &redact(&secret_pattern, &attribute.value),
                    budget.max_attr_value_chars,
                    &mut truncated,
                ),
            })
            .collect();

        Self {
            url: input.url,
            title,
            selected_text,
            dom_path,
            attributes,
            truncated,
        }
    }

    pub fn to_agent_context(&self) -> String {
        let mut context = format!("Preview URL: {}", self.url);
        if let Some(title) = &self.title {
            context.push_str(&format!("\nTitle: {title}"));
        }
        if let Some(dom_path) = &self.dom_path {
            context.push_str(&format!("\nDOM: {dom_path}"));
        }
        if let Some(selected_text) = &self.selected_text {
            context.push_str(&format!("\nSelection:\n{selected_text}"));
        }
        if !self.attributes.is_empty() {
            context.push_str("\nAttributes:");
            for attribute in &self.attributes {
                context.push_str(&format!("\n- {}={}", attribute.name, attribute.value));
            }
        }
        context
    }
}

fn redact(secret_pattern: &Regex, input: &str) -> String {
    input
        .lines()
        .map(|line| {
            if secret_pattern.is_match(line) {
                "[redacted]".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn truncate_chars(input: &str, max_chars: usize, truncated: &mut bool) -> String {
    let mut chars = input.chars();
    let value = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        *truncated = true;
    }
    value
}

#[cfg(test)]
mod tests {
    use relay_core::{CreateTask, ProjectId, Task, TaskSource};

    use super::*;

    fn now() -> Timestamp {
        Timestamp::UNIX_EPOCH
    }

    #[test]
    fn local_preview_provider_should_attach_localhost_target() {
        let (task, _) = Task::create(CreateTask {
            id: None,
            project_id: ProjectId::new(),
            title: "Preview".to_string(),
            source: TaskSource::Manual,
            now: now(),
        })
        .expect("task should create");
        let provider = LocalPreviewProvider;
        let command = provider
            .attach_target(
                task.id,
                PreviewRequest {
                    label: "App".to_string(),
                    uri: localhost_url(3000, "/"),
                },
                now(),
            )
            .expect("localhost preview should attach");

        assert!(matches!(command, TaskCommand::AttachPreview { .. }));
    }

    #[test]
    fn local_preview_provider_should_reject_remote_urls() {
        let provider = LocalPreviewProvider;
        let error = provider
            .attach_target(
                TaskId::new(),
                PreviewRequest {
                    label: "Remote".to_string(),
                    uri: "https://example.com".to_string(),
                },
                now(),
            )
            .expect_err("remote web URLs are deferred");

        assert!(matches!(error, PreviewError::UnsupportedUri(_)));
    }

    #[test]
    fn system_preview_opener_should_reject_remote_urls() {
        let mut opener = SystemPreviewOpener;
        let error = opener
            .open_target("https://example.com")
            .expect_err("remote web URLs are not preview targets");

        assert!(matches!(error, PreviewError::UnsupportedUri(_)));
    }

    #[test]
    fn browser_context_payload_should_redact_and_truncate() {
        let payload = BrowserContextPayload::sanitize(
            BrowserContextInput {
                url: "http://localhost:3000".to_string(),
                title: Some("Dashboard".to_string()),
                selected_text: Some("token=abc123\nvisible copy".to_string()),
                dom_path: Some("main form".to_string()),
                attributes: vec![
                    BrowserAttribute {
                        name: "data-testid".to_string(),
                        value: "submit-button".to_string(),
                    },
                    BrowserAttribute {
                        name: "password".to_string(),
                        value: "secret".to_string(),
                    },
                ],
            },
            SanitizerBudget {
                max_text_chars: 16,
                max_attr_value_chars: 16,
                max_attributes: 8,
            },
        );

        assert_eq!(payload.selected_text.as_deref(), Some("[redacted]\nvisib"));
        assert_eq!(payload.attributes.len(), 1);
        assert!(payload.truncated);
    }
}
