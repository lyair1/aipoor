use std::fmt::Write as _;
use std::path::PathBuf;

use chrono::{DateTime, Local};

use crate::providers::{ContextSnippet, Message, Provider};

pub struct HandoffBundle {
    pub source: Provider,
    pub target: Provider,
    pub project: Option<PathBuf>,
    pub generated_at: DateTime<Local>,
    pub session_path: Option<PathBuf>,
    pub recent_messages: Vec<Message>,
    pub snippets: Vec<ContextSnippet>,
    pub config_paths: Vec<PathBuf>,
    pub skill_dirs: Vec<PathBuf>,
}

impl HandoffBundle {
    pub fn render_markdown(&self) -> String {
        let mut out = String::new();
        let project = self
            .project
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        writeln!(
            out,
            "# Continue This Session In {}",
            self.target.display_name()
        )
        .ok();
        writeln!(out).ok();
        writeln!(out, "- Source agent: {}", self.source.display_name()).ok();
        writeln!(out, "- Target agent: {}", self.target.display_name()).ok();
        writeln!(out, "- Project: {project}").ok();
        writeln!(
            out,
            "- Generated: {}",
            self.generated_at.format("%Y-%m-%d %H:%M:%S %Z")
        )
        .ok();

        if let Some(session_path) = &self.session_path {
            writeln!(out, "- Source session file: {}", session_path.display()).ok();
        }

        if !self.config_paths.is_empty() {
            writeln!(out, "- Relevant config paths:").ok();
            for path in &self.config_paths {
                writeln!(out, "  - {}", path.display()).ok();
            }
        }

        if !self.skill_dirs.is_empty() {
            writeln!(out, "- Relevant skill dirs:").ok();
            for path in &self.skill_dirs {
                writeln!(out, "  - {}", path.display()).ok();
            }
        }

        writeln!(out).ok();
        writeln!(out, "## Instructions").ok();
        writeln!(
            out,
            "Use the following context as the continuation point. Treat filesystem, git state, and tool availability as potentially stale and verify before taking actions."
        )
        .ok();

        if !self.snippets.is_empty() {
            writeln!(out).ok();
            writeln!(out, "## Persistent Context").ok();
            for snippet in &self.snippets {
                writeln!(out).ok();
                writeln!(out, "### {} ({})", snippet.label, snippet.path.display()).ok();
                writeln!(out, "```text").ok();
                writeln!(out, "{}", snippet.content.trim()).ok();
                writeln!(out, "```").ok();
            }
        }

        if !self.recent_messages.is_empty() {
            writeln!(out).ok();
            writeln!(out, "## Recent Transcript").ok();
            for message in &self.recent_messages {
                writeln!(out).ok();
                writeln!(
                    out,
                    "### {}{}",
                    message.role_label(),
                    message
                        .timestamp
                        .map(|ts| format!(" ({})", ts.format("%Y-%m-%d %H:%M:%S %Z")))
                        .unwrap_or_default()
                )
                .ok();
                writeln!(out, "```text").ok();
                writeln!(out, "{}", message.content.trim()).ok();
                writeln!(out, "```").ok();
            }
        }

        writeln!(out).ok();
        writeln!(out, "## Resume Prompt").ok();
        writeln!(
            out,
            "Continue from this state. Keep the same task continuity, constraints, and project assumptions where they still hold."
        )
        .ok();

        out
    }
}
