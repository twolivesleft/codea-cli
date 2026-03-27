# codea-cli

`codea-cli` is a command line tool for working with [Codea](https://codea.io/) runtimes over MCP.

Agent-facing workflow guidance lives in [SKILL.md](/Users/sim/Developer/Open/codea-cli/SKILL.md).

## What It Does

The `codea` binary can:

- discover running Codea / Carbide hosts on the local network
- save and switch connection profiles
- inspect runtime status
- manage projects, collections, templates, and dependencies
- run, stop, and restart projects
- execute Lua in the running runtime
- inspect and change project runtime type
- query API docs and autocomplete data
- capture screenshots
- stream logs
- pull and push project files
- create new local projects when the target uses filesystem-backed storage

The CLI works with both:

- local macOS `Carbide.app` runners
- remote iOS / iPadOS Codea runtimes that expose the same MCP surface

## Build

```bash
cargo build
```

Run the debug binary directly:

```bash
./target/debug/codea --help
```

For an optimized build:

```bash
cargo build --release

# The binary will be in target/release

codea --help
```

## Quick Start

Discover a running Codea host:

```bash
codea discover
```

Show current connection and runtime state:

```bash
codea status
```

Create a new local project:

```bash
codea new MyGame
```

Create a plain folder project instead of a `.codea` bundle:

```bash
codea new MyGame --folder
```

Run a project:

```bash
codea run /path/to/MyGame.codea
```

Capture a screenshot:

```bash
codea screenshot
```

Show the full command surface:

```bash
codea --help
```

## Connection Model

The CLI stores profiles in:

```text
~/.codea/config.json
```

Environment variables override stored config:

- `CODEA_HOST`
- `CODEA_PORT`

## Local vs Remote Project Creation

`codea new` is target-aware:

- if no host is configured, it creates a local filesystem project
- if the connected host reports `projectStorage == "filesystem"`, it creates a local filesystem project. These hosts are typically the macOS Carbide.app or Codea.app
- if the connected host reports `projectStorage == "collections"`, it creates the project remotely via MCP. These hosts are typically iPhone or iPad devices running Codea

For local project creation, only the `Modern` template is supported.

## Command Surface

The Rust rewrite now includes the same top-level CLI commands as the Python tool:

- `discover`, `configure`, `status`
- `ls`, `new`, `rename`, `move`, `delete`
- `pull`, `push`
- `run`, `stop`, `restart`, `pause`, `resume`, `paused`, `exec`
- `screenshot`, `idle-timer`, `logs`, `clear-logs`
- `collections ls|new|delete`
- `templates ls|add|remove`
- `deps ls|available|add|remove`
- `autocomplete`, `runtime`, `doc`, `search-doc`

## Current State

The rewrite has reached command-surface parity with the Python CLI. Remaining work is primarily:

- end-to-end integration coverage against live Codea / Carbide hosts
- packaging and release automation

## Development

Format the crate:

```bash
cargo fmt
```

Build again after changes:

```bash
cargo build
```
