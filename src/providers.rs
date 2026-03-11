use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde_json::Value;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Codex,
    Claude,
    Gemini,
}

impl Provider {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Codex => "Codex CLI",
            Self::Claude => "Claude Code",
            Self::Gemini => "Gemini CLI",
        }
    }

    pub fn binary_name(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Gemini => "gemini",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "codex" => Some(Self::Codex),
            "claude" => Some(Self::Claude),
            "gemini" => Some(Self::Gemini),
            _ => None,
        }
    }

    pub fn all() -> [Self; 3] {
        [Self::Codex, Self::Claude, Self::Gemini]
    }

    pub fn detect(self) -> ProviderState {
        let binary = find_binary(self.binary_name());
        let home = self.home_dir();
        let installed = binary.is_some() || home.exists();

        ProviderState {
            installed,
            binary,
            home,
            config_paths: self.config_paths(),
            skill_dirs: self.skill_dirs(),
        }
    }

    pub fn home_dir(self) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        match self {
            Self::Codex => home.join(".codex"),
            Self::Claude => home.join(".claude"),
            Self::Gemini => home.join(".gemini"),
        }
    }

    pub fn config_paths(self) -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        match self {
            Self::Codex => vec![home.join(".codex/config.toml")],
            Self::Claude => vec![
                home.join(".claude/settings.json"),
                home.join(".claude.json"),
            ],
            Self::Gemini => vec![home.join(".gemini/settings.json")],
        }
    }

    pub fn skill_dirs(self) -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        match self {
            Self::Codex => vec![home.join(".codex/skills")],
            Self::Claude => vec![home.join(".claude/skills")],
            Self::Gemini => vec![home.join(".gemini/skills")],
        }
    }

    pub fn collect_context(
        self,
        project: Option<&Path>,
        max_messages: usize,
    ) -> Result<CollectedContext> {
        match self {
            Self::Codex => collect_codex_context(project, max_messages),
            Self::Claude => collect_claude_context(project, max_messages),
            Self::Gemini => collect_gemini_context(project, max_messages),
        }
    }
}

#[derive(Debug)]
pub struct ProviderState {
    pub installed: bool,
    pub binary: Option<PathBuf>,
    pub home: PathBuf,
    pub config_paths: Vec<PathBuf>,
    pub skill_dirs: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: Option<DateTime<Local>>,
}

impl Message {
    pub fn role_label(&self) -> &'static str {
        match self.role {
            MessageRole::User => "User",
            MessageRole::Assistant => "Assistant",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct ContextSnippet {
    pub label: String,
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug)]
pub struct CollectedContext {
    pub session_path: Option<PathBuf>,
    pub messages: Vec<Message>,
    pub snippets: Vec<ContextSnippet>,
    pub config_paths: Vec<PathBuf>,
    pub skill_dirs: Vec<PathBuf>,
}

fn collect_codex_context(project: Option<&Path>, max_messages: usize) -> Result<CollectedContext> {
    let home = Provider::Codex.home_dir();
    let session_path = pick_latest_file(
        &home.join("sessions"),
        project,
        Some(is_jsonl_file),
        |path| codex_session_matches_project(path, project),
    )?;
    let messages = match &session_path {
        Some(path) => parse_codex_session(path, max_messages)?,
        None => Vec::new(),
    };

    let mut snippets = Vec::new();
    if let Some(project_root) = project {
        let agents_path = project_root.join("AGENTS.md");
        if agents_path.exists() {
            snippets.push(read_snippet("AGENTS.md", &agents_path, 8_000)?);
        }
    }

    let memory_root = home.join("memories");
    if memory_root.exists() {
        for path in list_files(&memory_root, 3)? {
            snippets.push(read_snippet("Codex memory", &path, 4_000)?);
        }
    }

    Ok(CollectedContext {
        session_path,
        messages,
        snippets,
        config_paths: Provider::Codex.config_paths(),
        skill_dirs: Provider::Codex.skill_dirs(),
    })
}

fn collect_claude_context(project: Option<&Path>, max_messages: usize) -> Result<CollectedContext> {
    let home = Provider::Claude.home_dir();
    let projects_root = home.join("projects");
    let project_dir = project
        .map(claude_project_dir_name)
        .map(|name| projects_root.join(name));
    let session_path = if let Some(project_dir) = project_dir.as_ref().filter(|path| path.exists())
    {
        pick_latest_file(project_dir, None, Some(is_jsonl_file), |path| {
            !is_subagent_path(path)
        })?
    } else {
        pick_latest_file(&projects_root, None, Some(is_jsonl_file), |path| {
            !is_subagent_path(path)
        })?
    };

    let messages = match &session_path {
        Some(path) => parse_claude_session(path, max_messages)?,
        None => Vec::new(),
    };

    let mut snippets = Vec::new();
    if let Some(project_root) = project {
        for candidate in [
            project_root.join("CLAUDE.md"),
            project_root.join(".claude/CLAUDE.md"),
        ] {
            if candidate.exists() {
                snippets.push(read_snippet(
                    "Claude project instructions",
                    &candidate,
                    8_000,
                )?);
            }
        }
    }

    let global_claude_md = home.join("CLAUDE.md");
    if global_claude_md.exists() {
        snippets.push(read_snippet(
            "Claude global instructions",
            &global_claude_md,
            8_000,
        )?);
    }

    if let Some(project_dir) = project_dir {
        let memory_path = project_dir.join("memory/MEMORY.md");
        if memory_path.exists() {
            snippets.push(read_snippet("Claude project memory", &memory_path, 8_000)?);
        }
    }

    Ok(CollectedContext {
        session_path,
        messages,
        snippets,
        config_paths: Provider::Claude.config_paths(),
        skill_dirs: Provider::Claude.skill_dirs(),
    })
}

fn collect_gemini_context(project: Option<&Path>, max_messages: usize) -> Result<CollectedContext> {
    let home = Provider::Gemini.home_dir();
    let tmp_root = home.join("tmp");
    let project_name = project
        .and_then(|path| path.file_name())
        .and_then(OsStr::to_str);
    let session_path = if let Some(name) = project_name {
        let candidate_root = tmp_root.join(name);
        if candidate_root.exists() {
            pick_latest_file(
                &candidate_root,
                None,
                Some(|path: &Path| {
                    is_json_file(path)
                        && path.ancestors().any(|ancestor| ancestor.ends_with("chats"))
                }),
                |_| true,
            )?
        } else {
            pick_latest_file(
                &tmp_root,
                None,
                Some(|path: &Path| {
                    is_json_file(path)
                        && path.ancestors().any(|ancestor| ancestor.ends_with("chats"))
                }),
                |_| true,
            )?
        }
    } else {
        pick_latest_file(
            &tmp_root,
            None,
            Some(|path: &Path| {
                is_json_file(path) && path.ancestors().any(|ancestor| ancestor.ends_with("chats"))
            }),
            |_| true,
        )?
    };

    let messages = match &session_path {
        Some(path) => parse_gemini_session(path, max_messages)?,
        None => Vec::new(),
    };

    let mut snippets = Vec::new();
    let global_memory = home.join("GEMINI.md");
    if global_memory.exists() {
        snippets.push(read_snippet("Gemini global memory", &global_memory, 8_000)?);
    }

    if let Some(project_root) = project {
        let project_memory = project_root.join("GEMINI.md");
        if project_memory.exists() {
            snippets.push(read_snippet(
                "Gemini project memory",
                &project_memory,
                8_000,
            )?);
        }
    }

    Ok(CollectedContext {
        session_path,
        messages,
        snippets,
        config_paths: Provider::Gemini.config_paths(),
        skill_dirs: Provider::Gemini.skill_dirs(),
    })
}

fn parse_codex_session(path: &Path, max_messages: usize) -> Result<Vec<Message>> {
    let mut messages = Vec::new();
    for line in fs::read_to_string(path)
        .with_context(|| format!("failed reading {}", path.display()))?
        .lines()
    {
        let value: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if value.get("type").and_then(Value::as_str) != Some("response_item") {
            continue;
        }

        let payload = &value["payload"];
        if payload.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }

        let role = match payload.get("role").and_then(Value::as_str) {
            Some("user") => MessageRole::User,
            Some("assistant") => MessageRole::Assistant,
            _ => continue,
        };

        let content = payload
            .get("content")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("text").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();

        if !content.trim().is_empty() {
            messages.push(Message {
                role,
                content,
                timestamp: parse_timestamp(value.get("timestamp").and_then(Value::as_str)),
            });
        }
    }

    Ok(take_last_messages(messages, max_messages))
}

fn parse_claude_session(path: &Path, max_messages: usize) -> Result<Vec<Message>> {
    let mut messages = Vec::new();
    for line in fs::read_to_string(path)
        .with_context(|| format!("failed reading {}", path.display()))?
        .lines()
    {
        let value: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };

        let entry_type = value.get("type").and_then(Value::as_str);
        if !matches!(entry_type, Some("user") | Some("assistant")) {
            continue;
        }

        if value.get("isSidechain").and_then(Value::as_bool) == Some(true) {
            continue;
        }

        let message = value.get("message").unwrap_or(&Value::Null);
        let role = match message.get("role").and_then(Value::as_str) {
            Some("user") => MessageRole::User,
            Some("assistant") => MessageRole::Assistant,
            _ => continue,
        };

        let content = if let Some(text) = message.get("content").and_then(Value::as_str) {
            text.to_string()
        } else if let Some(items) = message.get("content").and_then(Value::as_array) {
            items
                .iter()
                .filter(|item| item.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|item| item.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            String::new()
        };

        if !content.trim().is_empty() {
            messages.push(Message {
                role,
                content,
                timestamp: parse_timestamp(value.get("timestamp").and_then(Value::as_str)),
            });
        }
    }

    Ok(take_last_messages(messages, max_messages))
}

fn parse_gemini_session(path: &Path, max_messages: usize) -> Result<Vec<Message>> {
    let value: Value = serde_json::from_str(
        &fs::read_to_string(path).with_context(|| format!("failed reading {}", path.display()))?,
    )
    .with_context(|| format!("failed parsing {}", path.display()))?;
    let messages = value
        .get("messages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let role = match item.get("type").and_then(Value::as_str) {
                Some("user") => MessageRole::User,
                Some("gemini") => MessageRole::Assistant,
                _ => return None,
            };

            let content = match item.get("content") {
                Some(Value::String(text)) => text.clone(),
                Some(Value::Array(items)) => items
                    .iter()
                    .filter_map(|entry| entry.get("text").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("\n"),
                _ => String::new(),
            };

            if content.trim().is_empty() {
                return None;
            }

            Some(Message {
                role,
                content,
                timestamp: parse_timestamp(item.get("timestamp").and_then(Value::as_str)),
            })
        })
        .collect::<Vec<_>>();

    Ok(take_last_messages(messages, max_messages))
}

fn find_binary(name: &str) -> Option<PathBuf> {
    let output = Command::new("which").arg(name).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!path.is_empty()).then(|| PathBuf::from(path))
}

fn pick_latest_file<F, G>(
    root: &Path,
    _project: Option<&Path>,
    filter: Option<F>,
    matches_project: G,
) -> Result<Option<PathBuf>>
where
    F: Fn(&Path) -> bool,
    G: Fn(&Path) -> bool,
{
    if !root.exists() {
        return Ok(None);
    }

    let predicate = |path: &Path| filter.as_ref().map(|func| func(path)).unwrap_or(true);
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !entry.file_type().is_file() || !predicate(path) || !matches_project(path) {
            continue;
        }

        let modified = fs::metadata(path)
            .and_then(|meta| meta.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

        match &best {
            Some((best_time, _)) if modified <= *best_time => {}
            _ => best = Some((modified, path.to_path_buf())),
        }
    }

    Ok(best.map(|(_, path)| path))
}

fn codex_session_matches_project(path: &Path, project: Option<&Path>) -> bool {
    let Some(project) = project else {
        return true;
    };

    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    let Some(first_line) = content.lines().next() else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<Value>(first_line) else {
        return false;
    };
    let cwd = value
        .get("payload")
        .and_then(|payload| payload.get("cwd"))
        .and_then(Value::as_str)
        .or_else(|| {
            value
                .get("payload")
                .and_then(|payload| payload.get("session_meta"))
                .and_then(|payload| payload.get("cwd"))
                .and_then(Value::as_str)
        });

    cwd.map(|cwd| cwd == project.to_string_lossy())
        .unwrap_or(false)
}

fn read_snippet(label: &str, path: &Path, max_len: usize) -> Result<ContextSnippet> {
    let mut content = fs::read_to_string(path)
        .with_context(|| format!("failed reading snippet from {}", path.display()))?;
    if content.len() > max_len {
        content.truncate(max_len);
        content.push_str("\n...[truncated]");
    }

    Ok(ContextSnippet {
        label: label.to_string(),
        path: path.to_path_buf(),
        content,
    })
}

fn list_files(root: &Path, limit: usize) -> Result<Vec<PathBuf>> {
    let mut files = WalkDir::new(root)
        .min_depth(1)
        .max_depth(2)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<_>>();
    files.sort();
    files.truncate(limit);
    Ok(files)
}

fn parse_timestamp(value: Option<&str>) -> Option<DateTime<Local>> {
    let value = value?;
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|ts| ts.with_timezone(&Local))
}

fn take_last_messages(mut messages: Vec<Message>, max_messages: usize) -> Vec<Message> {
    if messages.len() > max_messages {
        messages.drain(0..messages.len() - max_messages);
    }
    messages
}

fn claude_project_dir_name(project: &Path) -> String {
    format!(
        "-{}",
        project
            .to_string_lossy()
            .trim_start_matches('/')
            .replace('/', "-")
    )
}

fn is_jsonl_file(path: &Path) -> bool {
    path.extension().and_then(OsStr::to_str) == Some("jsonl")
}

fn is_json_file(path: &Path) -> bool {
    path.extension().and_then(OsStr::to_str) == Some("json")
}

fn is_subagent_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == OsStr::new("subagents"))
}
