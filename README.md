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

## Remote Device Workflow

For iPhone and iPad targets, the common loop is:

1. connect to the device
2. create or pull a project locally
3. edit files on disk
4. push changes back
5. run the project
6. inspect logs or capture a screenshot

Example:

```bash
# Find and save a device
codea discover
codea configure --host 192.168.1.42 --port 18513

# Create or pull a project
codea new "My Game"
codea pull "My Game"

# Edit files locally, then push them back
codea push "My Game" Main.lua Player.lua

# Run and inspect
codea run "My Game"
codea logs --tail 20
codea screenshot --output result.png
```

For local macOS Codea or Carbide targets, the workflow is different: edit files directly on disk and run the project by path. See [SKILL.md](/Users/sim/Developer/Open/codea-cli/SKILL.md) for the full target-aware workflow.

## Project Naming

Collection-backed projects can be addressed by bare name when unique, or by full logical path when disambiguation is needed:

```bash
codea pull "Morse"
codea pull "Documents/Morse"
codea pull "iCloud/Documents/Foo"
```

The same naming works with other project commands such as `new`, `run`, `push`, `pull`, `rename`, `move`, and `delete`.

Filesystem-backed targets are addressed by local path:

```bash
codea run /path/to/MyGame.codea
codea run /path/to/MyGame
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

Named profiles are supported:

```bash
codea discover --profile ipad
codea status --profile ipad
codea ls --profile ipad
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

## Pull and Push

`codea pull` mirrors a collection-backed project into a local directory:

```text
My Game/
  Main.lua
  Player.lua
  ...
  Dependencies/
    PhysicsLib/
      Physics.lua
```

Dependencies are pulled automatically unless `--no-deps` is used.

Useful options:

- `codea pull "My Game" --output ~/projects/mygame`
- `codea pull "My Game" --no-deps`
- `codea push "My Game" --input ~/projects/mygame`
- `codea push "My Game" Main.lua Player.lua`

Selective push is useful when a project is already running and you only want to update the changed files.

On filesystem-backed targets, `pull` and `push` are usually unnecessary because the CLI can work directly with the same files on disk.

## Logs

The log commands support full output, slices, and streaming:

```bash
codea logs
codea logs --head 20
codea logs --tail 20
codea logs --follow
```

`--head` is useful for startup errors, `--tail` is useful for the latest output, and `--follow` streams logs in real time while the project runs.

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

## MCP

Codea also exposes an MCP endpoint directly:

```text
http://<device-ip>:18513/mcp
```

Use the CLI when you want:

- shell-friendly workflows
- network discovery with `codea discover`
- local pull/edit/push cycles
- built-in log streaming with `codea logs --follow`

Use MCP directly when you want:

- an MCP-native client integration
- direct file operations on the target without wrapping them in CLI commands
- quick automation in tools that already speak MCP

In practice, the CLI is usually the better interface for terminal and agent workflows, while direct MCP is useful for clients that already have a strong MCP integration model.

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
