<img width="966" height="174" alt="Screenshot 2026-03-11 at 16 47 47" src="https://github.com/user-attachments/assets/6416cf6c-ae89-43bc-af77-f67eb90da82b" />

# aipoor

`aipoor` is for people with premium-agent ambitions and very normal-agent budgets.

If you cannot justify a `$200/month` mega-subscription, but you *can* justify juggling a few `$20/month` CLI subscriptions, this tool is for you.

You start in one agent, hit the token wall, switch to another one, and suddenly you are doing manual memory reconstruction like a digital archaeologist.

That gets old fast.

`aipoor` helps you jump between `codex`, `claude`, and `gemini` without losing the thread.

It reads the local state those CLIs already store, extracts the useful context, builds a handoff bundle, saves it, and copies it to your clipboard so you can paste it into the next agent and keep going.

Not glamorous. Extremely useful.

## Quick Start

Install it:

```bash
curl -fsSL https://raw.githubusercontent.com/lyair1/aipoor/main/install.sh | sh
```

Then, inside the project you are working on, create a handoff:

```bash
aipoor sync codex claude
```

What happens next:

1. `aipoor` reads the current project context.
2. It builds a handoff bundle for the target agent.
3. It saves that bundle under `~/.aipoor/bundles/`.
4. It copies the handoff text to your clipboard.
5. You open the next agent, paste, and continue.

Example switch:

```bash
cd /Users/you/src/project
aipoor sync claude gemini
# now open Gemini CLI and paste
```

## Why This CLI Exists

This tool exists because the real-world workflow looks like this:

1. You use one AI CLI.
2. It runs out of tokens, rate limit, patience, or all three.
3. You switch to another CLI.
4. You waste ten minutes rebuilding context by hand.
5. You become annoyed enough to build a tool.

So here we are.

`aipoor` is basically a continuity tool for people running a multi-CLI workflow on a non-luxury budget.

## What It Does

When you run a command like this:

```bash
aipoor sync claude gemini
```

`aipoor` will:

1. Use the folder you are currently in as the project.
2. Find the latest relevant local session for the source CLI.
3. Extract recent transcript messages.
4. Pull in persistent memory or instruction files when available.
5. Include useful config and skill directory references.
6. Generate a markdown handoff bundle for the target CLI.
7. Save that bundle under `~/.aipoor/bundles/`.
8. Copy the same bundle to your clipboard.

Then you open the next agent, paste, and continue.

## What It Supports

Right now it supports:

- `Codex CLI`
- `Claude Code`
- `Gemini CLI`

## Install

### Recommended install with `curl`

Once the repo and releases are published, install it with:

```bash
curl -fsSL https://raw.githubusercontent.com/lyair1/aipoor/main/install.sh | sh
```

What this does:

1. Downloads the install script.
2. Fetches the correct binary for your machine from GitHub Releases.
3. Installs `aipoor` into your local bin directory.
4. Runs the setup command for you.

### Install from source

If you already have the repo locally, install it with Cargo:

```bash
cargo install --path .
```

After that, the binary is available as:

```bash
aipoor
```

## Setup

After install, run this from the project you usually want to work on:

```bash
aipoor setup --project "$(pwd)"
```

This stores your default project in:

```bash
~/.aipoor/config.toml
```

It also detects your local AI CLIs so you can confirm everything is wired correctly.

## How To Use It

### Step 1: Detect your installed CLIs

Run:

```bash
aipoor detect
```

This shows:

- whether `codex`, `claude`, and `gemini` were found
- their home directories
- detected config files
- detected skill directories

If one of them is missing, you will know immediately.

### Step 2: Pick your source and target agent

The main command shape is:

```bash
aipoor sync <from> <to>
```

By default, `aipoor` uses the folder you are running the command from.

Examples:

```bash
cd /Users/yairlevi/src/aipoor && aipoor sync codex claude
cd /Users/yairlevi/src/betterClaw && aipoor sync claude gemini
cd /Users/yairlevi/src/escape-market && aipoor sync gemini codex
```

If you want to override that, you can still do:

```bash
aipoor sync claude gemini --project /Users/you/src/project
```

### Step 3: Let `aipoor` build the handoff

When `sync` runs, it:

1. Reads the latest local session from the source CLI.
2. Pulls the most recent messages.
3. Adds any durable context it can find.
4. Saves the bundle to `~/.aipoor/bundles/...`.
5. Copies the handoff text to your clipboard.

### Step 4: Paste into the next CLI

After the command finishes:

1. Open the target CLI
2. Paste the clipboard contents
3. Continue the task

That is the whole point of this tool.

## Step-By-Step Example

Here is the intended workflow:

1. Start working in `codex`
2. Hit the wall
3. Run:

```bash
cd /Users/you/src/project
aipoor sync codex claude
```

4. Open `claude`
5. Paste the handoff
6. Continue working
7. Hit another wall
8. Run:

```bash
cd /Users/you/src/project
aipoor sync claude gemini
```

9. Open `gemini`
10. Paste
11. Continue pretending this was always an intentional multi-agent architecture decision

## Useful Commands

Set your default project:

```bash
aipoor setup --project "$(pwd)"
```

Use the current folder as the project:

```bash
aipoor sync codex claude
```

Include more recent transcript messages:

```bash
aipoor sync claude gemini --messages 20
```

Override the folder explicitly:

```bash
aipoor sync claude gemini --project /Users/you/src/project
```

Print the generated handoff to stdout too:

```bash
aipoor sync gemini codex --stdout
```

## Where `aipoor` Stores Its Own Files

Everything `aipoor` writes lives under:

```bash
~/.aipoor/
```

Main locations:

- `~/.aipoor/config.toml`
- `~/.aipoor/bundles/`

## Important Limitation

`aipoor` does **not** try to directly merge native session databases across different AI CLIs.

That would be fragile, painful, and a great way to create new and exciting problems.

Instead, it creates a reliable handoff bundle you can paste into the next tool.

This is not a universal session teleporter.

It is a practical “keep moving without rewriting the entire context from scratch” tool.

## In One Sentence

`aipoor` is what you use when you are too cheap for the fancy plan, too busy to rewrite context manually, and too pragmatic to care whether the solution is elegant.
