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

## Install

Install via Homebrew

```bash
brew install twolivesleft/tap/codea
```

Windows users can either run the PowerShell installer or install the MSI from the latest GitHub release

```powershell
powershell -c "irm https://github.com/twolivesleft/codea-cli/releases/latest/download/codea-cli-installer.ps1 | iex"
```

Or you can build locally with Cargo

```bash
cargo build --release
./target/release/codea --help
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

Force local creation without probing the configured device:

```bash
codea new MyGame --local
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

The same file also stores cached update-check state.

Environment variables override stored config:

- `CODEA_HOST`
- `CODEA_PORT`
- `CODEA_NO_UPDATE_CHECK=1` disables the once-per-day release check

Clear a saved profile:

```bash
codea configure --clear
```

## Local vs Remote Project Creation

`codea new` is target-aware:

- if no host is configured, it creates a local filesystem project
- if the connected host reports `projectStorage == "filesystem"`, it creates a local filesystem project. These hosts are typically the macOS Carbide.app or Codea.app
- if the connected host reports `projectStorage == "collections"`, it creates the project remotely via MCP. These hosts are typically iPhone or iPad devices running Codea
- if a configured device is unreachable, `codea new` now falls back to local creation after a short probe and prints a warning

Use these overrides when you want explicit behavior:

- `codea new --local` forces local filesystem creation without probing a device
- `codea --wait new ...` keeps waiting for the configured device instead of falling back
- `codea configure --clear` forgets the saved device profile

For local project creation, only the `Modern` template is supported.

## Command Surface

- `discover`, `configure`, `status`
- `ls`, `new`, `rename`, `move`, `delete`
- `pull`, `push`
- `run`, `stop`, `restart`, `pause`, `resume`, `paused`, `exec`
- `screenshot`, `idle-timer`, `logs`, `clear-logs`
- `collections ls|new|delete`
- `templates ls|add|remove`
- `deps ls|available|add|remove`
- `autocomplete`, `runtime`, `doc`, `search-doc`

## Release

Homebrew publishing is automated through GitHub Actions and a separate tap repo:

- release repo: `twolivesleft/codea-cli`
- tap repo: `twolivesleft/homebrew-tap`

Current release setup expects:

- a GitHub App installed on `twolivesleft/homebrew-tap`
- `HOMEBREW_APP_ID` configured as a repository variable in `twolivesleft/codea-cli`
- `HOMEBREW_APP_PRIVATE_KEY` configured as a repository secret in `twolivesleft/codea-cli`

To publish a release:

```bash
# Update Cargo.toml version first, then:
git tag v0.1.5
git push origin v0.1.5
```

The generated [release workflow](/Users/sim/Developer/Open/codea-cli/.github/workflows/release.yml) will:

- build release archives for the configured targets
- create the GitHub release
- generate the Homebrew formula
- push `Formula/codea.rb` to `twolivesleft/homebrew-tap`

## Development

Format the crate:

```bash
cargo fmt
```

Build again after changes:

```bash
cargo build
```
