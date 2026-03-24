# codea-cli

`codea-cli` is a command line tool for working with [Codea](https://codea.io/) runtimes over MCP.

## What It Does

The `codea` binary can:

- discover running Codea / Carbide hosts on the local network
- save and switch connection profiles
- inspect runtime status
- run, stop, and restart projects
- execute Lua in the running runtime
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
- if the connected host reports `projectStorage == "filesystem"`, it creates a local filesystem project. These hosts are typically iPad or iPhones running Codea
- if the connected host reports `projectStorage == "collections"`, it creates the project remotely via MCP. These hosts are the macOS Carbide.app or Codea.app

For local project creation, only the `Modern` template is supported.

## Current State

This is an early rewrite. The core command surface is implemented and the crate builds successfully, but it still needs:

- fuller end-to-end testing against live Codea / Carbide hosts
- parity verification against the Python tool
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
