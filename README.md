# aipoor

`aipoor` is for people with premium-agent ambitions and very normal-agent budgets.

If you cannot justify a `$200/month` mega-subscription, but you *can* justify juggling a few `$20/month` CLI subscriptions, this tool is for you.

You start in one agent, hit the token wall, switch to another one, and suddenly you are doing manual memory reconstruction like a digital archaeologist.

That gets old fast.

`aipoor` helps you jump between `codex`, `claude`, and `gemini` without losing the thread.

It reads the local state those CLIs already store, extracts the useful context, builds a handoff bundle, saves it, and copies it to your clipboard so you can paste it into the next agent and keep going.

Not glamorous. Extremely useful.

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
aipoor sync claude gemini --project /Users/you/src/my-project
```

`aipoor` will:

1. Find the latest relevant local session for the source CLI.
2. Extract recent transcript messages.
3. Pull in persistent memory or instruction files when available.
4. Include useful config and skill directory references.
5. Generate a markdown handoff bundle for the target CLI.
6. Save that bundle under `~/.aipoor/bundles/`.
7. Copy the same bundle to your clipboard.

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
curl -fsSL https://raw.githubusercontent.com/yairlevi/aipoor/main/install.sh | sh
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
aipoor sync <from> <to> --project /absolute/path/to/project
```

Examples:

```bash
aipoor sync codex claude --project /Users/yairlevi/src/aipoor
aipoor sync claude gemini --project /Users/yairlevi/src/betterClaw
aipoor sync gemini codex --project /Users/yairlevi/src/escape-market
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
aipoor sync codex claude --project /Users/you/src/project
```

4. Open `claude`
5. Paste the handoff
6. Continue working
7. Hit another wall
8. Run:

```bash
aipoor sync claude gemini --project /Users/you/src/project
```

9. Open `gemini`
10. Paste
11. Continue pretending this was always an intentional multi-agent architecture decision

## Useful Commands

Set your default project:

```bash
aipoor setup --project "$(pwd)"
```

Use the default project later:

```bash
aipoor sync codex claude
```

Include more recent transcript messages:

```bash
aipoor sync claude gemini --project /Users/you/src/project --messages 20
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
