---
name: codea
description: Control Codea on a connected iOS, iPadOS, or macOS device. Use this skill when working on Codea projects — pulling code, editing files, pushing changes, running projects, capturing screenshots, and inspecting state via Lua.
---

# Codea Skill

This directory contains the `codea` CLI tool for working with Codea projects on a connected iOS, iPadOS, or macOS device.

The most important distinction is the target type:

- `Project storage = filesystem`
  Use the local filesystem workflow. This is the macOS Codea / Carbide case. Edit files directly on disk and `run` the project by path. Do not use `pull` / `push` unless you specifically need them for some other reason.
- `Project storage = collections`
  Use the repository workflow. This is the iOS / iPadOS Codea case. Pull projects from the device, edit locally, then push changes back.

## Setup

Check whether the CLI is already installed:

```bash
codea --help
```

Build it locally if needed:

```bash
cargo build
./target/debug/codea --help
```

Connect to a target:

```bash
codea discover
codea configure --host 192.168.1.42 --port 18513
```

Or use environment variables:

```bash
export CODEA_HOST=192.168.1.42
export CODEA_PORT=18513
```

If `codea` is not on `PATH`, use `./target/debug/codea ...`.

## Determine The Target Type

Before choosing a workflow, query the current target state and check `Project storage`.

```bash
codea status
```

- If `Project storage` is `filesystem`, the target is a local macOS app and projects live directly on disk.
- If `Project storage` is `collections`, the target is using Codea's project repository model and projects should be accessed through `pull` / `push`.

Also pay attention to `Project path` when present. This is the canonical running project identifier:

- `Examples/Flappy` or `iCloud/Documents/Foo` for collection-backed targets
- `/path/to/MyGame.codea` for filesystem-backed targets

## Project Naming

Collection-backed projects are identified as `Collection/Project` or just `Project` if the name is unique:

```bash
codea pull "Morse"
codea pull "Documents/Morse"
codea pull "iCloud/Documents/Foo"
```

Filesystem-backed targets are usually addressed by local path:

```bash
codea run /path/to/MyGame.codea
codea run /path/to/MyGame
```

## Typical Agent Workflow

Always disable the idle timer at the start of a device session so the target stays awake:

```bash
codea idle-timer off
```

### Filesystem-backed workflow (`projectStorage = filesystem`)

Use this for local macOS Codea / Carbide targets.

```bash
# 1. Work directly in the project directory
cd /path/to/MyGame.codea

# 2. Read and edit files with normal filesystem tools

# 3. Run the project by path
codea clear-logs
codea logs --follow >> /tmp/codea.log &
codea run /path/to/MyGame.codea
sleep 2
codea screenshot --output result.png

# 4. Inspect runtime state
codea exec "print(WIDTH, HEIGHT)"
cat /tmp/codea.log

# 5. Iterate by editing files on disk, then restart or run again
codea restart
```

For filesystem-backed targets, `push` and `pull` are normally unnecessary because the agent can already access the same files directly.

### Collection-backed workflow (`projectStorage = collections`)

Use this for iPhone / iPad Codea targets.

```bash
# 1. Pull the project and its dependencies
codea pull "My Game"

# 2. Read and edit files locally

# 3. Push only the changed files when possible
codea push "My Game" Main.lua Player.lua

# 4. Start logs, run, and inspect
codea clear-logs
codea logs --follow >> /tmp/codea.log &
codea run "My Game"
sleep 3
codea screenshot --output result.png

# 5. Execute Lua to inspect state
codea exec "print(health)"

# 6. Check logs
cat /tmp/codea.log

# 7. Iterate
codea push "My Game" Main.lua
codea restart
sleep 2
cat /tmp/codea.log
```

### Creating a new project

`codea new` is target-aware:

- On filesystem-backed targets it creates a local project on disk.
- On collection-backed targets it creates a project in the target repository.

Examples:

```bash
codea new "My Game"
codea new "My Game" --folder
codea new "My Game" --template Modern
codea new "Documents/My Game"
codea new "iCloud/Documents/My Game"
```

After creation:

- On filesystem-backed targets, edit the created directory directly and `run` it by path.
- On collection-backed targets, `pull` it locally, edit, `push`, then `run`.

## Global Flag: `--wait`

```bash
codea --wait <command>
```

This waits for the Air Code server to respond before running the command. Prefer this over asking the user to manually foreground Codea.

```bash
codea --wait ls
codea --wait run "My Game"
codea --wait run /path/to/MyGame.codea
```

## Commands

### Device
| Command | Description |
|---------|-------------|
| `codea discover` | Scan the local network for Codea devices and save config |
| `codea configure` | Manually set device host/port |
| `codea status` | Show current device config and live state |

### Collections
| Command | Description |
|---------|-------------|
| `codea collections ls` | List all collections |
| `codea collections new <name>` | Create a new local collection |
| `codea collections delete <name>` | Delete a collection |

### Projects
| Command | Description |
|---------|-------------|
| `codea ls` | List all projects as `Collection/Project` |
| `codea new <name>` | Create a new project; local or remote depending on `projectStorage` |
| `codea rename <project> <newname>` | Rename a project |
| `codea move <project> <collection>` | Move a project |
| `codea delete <project>` | Delete a project |
| `codea runtime <project>` | Get runtime type |
| `codea runtime <project> <type>` | Set runtime type |

### Files
| Command | Description |
|---------|-------------|
| `codea pull <project> [files...]` | Pull project files locally |
| `codea push <project> [files...]` | Push files back to the target |

### Runtime
| Command | Description |
|---------|-------------|
| `codea run <project-or-path>` | Start a project by repository name or filesystem path |
| `codea stop` | Stop the running project |
| `codea restart` | Restart the running project |
| `codea exec "<lua>"` | Execute Lua in the running project |
| `codea exec --file <path>` | Execute a Lua file |
| `codea pause` | Pause the running project |
| `codea resume` | Resume the running project |
| `codea paused [on\|off]` | Get or set paused state |
| `codea screenshot [--output <file>]` | Capture a screenshot |
| `codea idle-timer <on\|off>` | Get or set idle timer |
| `codea logs` | Get log output |
| `codea logs --head N` | Get first N lines |
| `codea logs --tail N` | Get last N lines |
| `codea logs --follow` | Stream logs in real time |
| `codea clear-logs` | Clear the log buffer |

### Templates
| Command | Description |
|---------|-------------|
| `codea templates ls` | List all templates |
| `codea templates add <project>` | Add a custom template |
| `codea templates remove <name>` | Remove a custom template |

### Dependencies
| Command | Description |
|---------|-------------|
| `codea deps ls <project>` | List project dependencies |
| `codea deps available <project>` | List addable dependencies |
| `codea deps add <project> <dependency>` | Add a dependency |
| `codea deps remove <project> <dependency>` | Remove a dependency |

### Documentation
| Command | Description |
|---------|-------------|
| `codea autocomplete <project> <code>` | Get completions for a Lua prefix |
| `codea doc <function>` | Show API docs for the current runtime context; defaults to the running project's runtime, otherwise `modern` |
| `codea doc <function> --all` | Show both modern and legacy docs |
| `codea doc <function> --modern` | Show modern docs only |
| `codea doc <function> --legacy` | Show legacy docs only |
| `codea doc <function> --project <name-or-path>` | Filter docs by that project's runtime |
| `codea doc <function> --project` | Filter docs by the currently running project's runtime |
| `codea search-doc <query>` | Search docs for the current runtime context; defaults to the running project's runtime, otherwise `modern` |
| `codea search-doc <query> --all` | Search both modern and legacy docs |
| `codea search-doc <query> --modern` | Search modern docs only |
| `codea search-doc <query> --legacy` | Search legacy docs only |
| `codea search-doc <query> --project <name-or-path>` | Search docs using that project's runtime |
| `codea search-doc <query> --project` | Search docs using the currently running project's runtime |

When `--all` is given, `doc` and `search-doc` return both modern and legacy entries.

When no runtime flags are given, `doc` and `search-doc` resolve runtime in this order:

1. `--project <name-or-path>`
2. bare `--project` using the currently running project
3. the currently running project's runtime automatically
4. `modern` if no project is running

On filesystem-backed macOS targets, runtime should be treated as `modern`.

## Pull / Push Details

`codea pull "My Game"` creates:

```text
My Game/
  Main.lua
  Player.lua
  ...
  Dependencies/
    PhysicsLib/
      Physics.lua
```

`codea push "My Game"` pushes all files in `./My Game/` back, routing `Dependencies/<name>/` files to the correct project on the target.

Use `--output <dir>` with pull and `--input <dir>` with push to specify custom directories.

## File Loading Order (`Info.plist`)

Each Codea project contains an `Info.plist` file. The `Buffer Order` array defines the file load order. When adding new `.lua` files, update `Info.plist` and either push it back to the collection-backed target or keep it correct in the local project directory on filesystem-backed targets.

## Log Monitoring with `--follow`

The recommended pattern is:

```bash
codea clear-logs
codea logs --follow >> /tmp/codea.log &
codea run "My Game"

cat /tmp/codea.log
tail -n 20 /tmp/codea.log
```

For filesystem-backed targets, replace `"My Game"` with a local path as needed.

Kill the background stream when done:

```bash
kill %1
```
