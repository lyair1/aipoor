mod bundle;
mod clipboard;
mod config;
mod providers;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use bundle::HandoffBundle;
use chrono::Local;
use clap::{Parser, Subcommand};
use config::AppConfig;
use providers::Provider;

#[derive(Parser)]
#[command(name = "aipoor")]
#[command(about = "Create clipboard handoff bundles between Codex, Claude, and Gemini")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Detect,
    Setup {
        #[arg(long)]
        project: Option<PathBuf>,
    },
    Sync {
        from: String,
        to: String,
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long, default_value_t = 12)]
        messages: usize,
        #[arg(long)]
        stdout: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Detect => cmd_detect(),
        Commands::Setup { project } => cmd_setup(project),
        Commands::Sync {
            from,
            to,
            project,
            messages,
            stdout,
        } => cmd_sync(&from, &to, project.as_deref(), messages, stdout),
    }
}

fn cmd_detect() -> Result<()> {
    for provider in Provider::all() {
        let state = provider.detect();
        println!(
            "{}: {}",
            provider.display_name(),
            if state.installed {
                "detected"
            } else {
                "not detected"
            }
        );
        println!("  home: {}", state.home.display());
        println!(
            "  binary: {}",
            state
                .binary
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "not found".to_string())
        );
        if !state.config_paths.is_empty() {
            println!("  configs:");
            for path in state.config_paths {
                println!("    {}", path.display());
            }
        }
        if !state.skill_dirs.is_empty() {
            println!("  skills:");
            for path in state.skill_dirs {
                println!("    {}", path.display());
            }
        }
    }

    Ok(())
}

fn cmd_setup(project: Option<PathBuf>) -> Result<()> {
    let project = project.map(Ok).unwrap_or_else(|| {
        std::env::current_dir().context("failed to resolve current directory")
    })?;

    let config = AppConfig {
        default_project: Some(project.clone()),
    };
    let path = config.save()?;

    println!("Saved config to {}", path.display());
    println!("Default project: {}", project.display());
    println!();
    cmd_detect()
}

fn cmd_sync(
    from: &str,
    to: &str,
    project_arg: Option<&Path>,
    messages: usize,
    stdout: bool,
) -> Result<()> {
    let source =
        Provider::parse(from).with_context(|| format!("unsupported source provider: {from}"))?;
    let target =
        Provider::parse(to).with_context(|| format!("unsupported target provider: {to}"))?;

    if source == target {
        bail!("source and target providers must be different");
    }

    let config = AppConfig::load()?;
    let project = config.resolve_project(project_arg)?;
    let context = source.collect_context(Some(project.as_path()), messages)?;
    let bundle = HandoffBundle {
        source,
        target,
        project: Some(project.clone()),
        generated_at: Local::now(),
        session_path: context.session_path,
        recent_messages: context.messages,
        snippets: context.snippets,
        config_paths: context.config_paths,
        skill_dirs: context.skill_dirs,
    };

    let rendered = bundle.render_markdown();
    let bundle_path = save_bundle(source, target, &rendered)?;
    clipboard::copy_to_clipboard(&rendered)?;

    if stdout {
        println!("{rendered}");
    } else {
        println!("Copied handoff bundle to clipboard.");
        println!("Saved bundle to {}", bundle_path.display());
        println!("Project: {}", project.display());
        println!("Use it by pasting into {}.", target.display_name());
    }

    Ok(())
}

fn save_bundle(source: Provider, target: Provider, contents: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().context("failed to resolve home directory")?;
    let bundle_dir = home.join(".aipoor").join("bundles");
    fs::create_dir_all(&bundle_dir)
        .with_context(|| format!("failed creating {}", bundle_dir.display()))?;

    let file_name = format!(
        "{}-{}-to-{}.md",
        Local::now().format("%Y%m%d-%H%M%S"),
        source.binary_name(),
        target.binary_name()
    );
    let path = bundle_dir.join(file_name);
    fs::write(&path, contents).with_context(|| format!("failed writing {}", path.display()))?;
    Ok(path)
}
