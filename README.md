# aipoor

`aipoor` is a Rust CLI that creates clipboard-ready handoff bundles between `codex`, `claude`, and `gemini`.

Instead of trying to replay native sessions across incompatible CLIs, it extracts:

- recent transcript messages
- project memory and instruction files
- relevant config paths
- skill directories

Then it writes a target-specific markdown handoff, saves it locally, and copies it to the clipboard.

## Commands

```bash
aipoor detect
aipoor setup --project /path/to/project
aipoor sync codex claude
aipoor sync gemini codex --project /path/to/project --messages 20
aipoor sync claude gemini --stdout
```

## What `sync` does

`sync <from> <to>`:

1. Detects the latest relevant session from the source CLI.
2. Extracts the last N user/assistant messages.
3. Pulls persistent context from known files like `AGENTS.md`, `CLAUDE.md`, `MEMORY.md`, and `GEMINI.md`.
4. Saves the resulting markdown bundle under `~/.aipoor/bundles/`.
5. Copies the bundle to the clipboard.

## Setup

```bash
cargo run -- setup --project "$(pwd)"
```

This stores the default project in `~/.aipoor/config.toml`.

## Install

Local install:

```bash
cargo install --path .
aipoor setup --project "$(pwd)"
```

GitHub-release install is scaffolded in `install.sh`.
