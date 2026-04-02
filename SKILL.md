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

### Install if neded

For macOS, Linux, or WSL

```bash
brew install twolivesleft/tap/codea
```

For Windows via PowerShell
```powershell
powershell -c "irm https://github.com/twolivesleft/codea-cli/releases/latest/download/codea-cli-installer.ps1 | iex"
```

### Configuration

Connect to a target:

```bash
codea discover
codea configure --host 192.168.1.42 --port 18513
```

Clear a saved device profile when you want to go back to local-only behavior:

```bash
codea configure --clear
```

Or use environment variables:

```bash
export CODEA_HOST=192.168.1.42
export CODEA_PORT=18513
```

The CLI also performs a cached once-per-day update check by default. Set `CODEA_NO_UPDATE_CHECK=1` to disable it.

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
# Skip dependencies only if you explicitly do not need them
codea pull "My Game" --no-deps

# 2. Read and edit files (use standard file tools)

# 3. Push only the modified files (prefer this over pushing the entire project)
codea push "My Game" Main.lua Player.lua
# Push entire project only if you don't know which files changed
codea push "My Game"

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
- If a saved device is unreachable, `codea new` falls back to local creation after a short probe unless `--wait` is set.
- For local creation, only the `Modern` template is supported.

Examples:

```bash
codea new "My Game"
codea new "My Game" --local
codea new "My Game" --folder
codea new "My Game" --template Modern
codea new "Documents/My Game"
codea new "iCloud/Documents/My Game"
```

After creation:

- On filesystem-backed targets, edit the created directory directly and `run` it by path.
- On collection-backed targets, `pull` it locally, edit, `push`, then `run`.

Use these overrides when needed:

- `codea new --local` forces local creation without probing the configured device.
- `codea --wait new ...` keeps waiting for the configured device instead of falling back.
- `codea configure --clear` forgets the saved device configuration.

## Global Flag: `--wait`

```bash
codea --wait <command>
```

Blocks until Codea's Air Code server responds, then runs the command. Use this whenever the user may have Codea backgrounded on their device (e.g. they are chatting with you from another app). The CLI will poll silently and report how long it waited.

```bash
codea --wait ls
codea --wait run "My Game"
codea --wait run /path/to/MyGame.codea
```

Always prefer `--wait` over asking the user to manually switch to Codea first.

## Commands

### Device
| Command | Description |
|---------|-------------|
| `codea discover` | Scan the local network for Codea devices and save config |
| `codea configure` | Manually set device host/port, or `--clear` the saved profile |
| `codea status` | Show current device config and live state |

### Collections
| Command | Description |
|---------|-------------|
| `codea collections ls` | List all collections |
| `codea collections new <name>` | Create a new local collection |
| `codea collections delete <name>` | Delete a collection (prompts for confirmation) |

### Projects
| Command | Description |
|---------|-------------|
| `codea ls` | List all projects as `Collection/Project` |
| `codea new <name>` | Create a new project; local or remote depending on `projectStorage`, with `--local` to force local creation |
| `codea rename <project> <newname>` | Rename a project |
| `codea move <project> <collection>` | Move a project |
| `codea delete <project>` | Delete a project (prompts for confirmation) |
| `codea runtime <project>` | Get runtime type |
| `codea runtime <project> <type>` | Set runtime type |

### Files
| Command | Description |
|---------|-------------|
| `codea pull <project> [files...]` | Pull project files locally; also pulls dependencies unless `--no-deps` is used |
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
| `codea templates remove <name>` | Remove a custom template (prompts for confirmation) |

Custom templates are part of the collection-backed workflow. They live in the `Templates` collection and can be edited like normal projects:

```bash
codea templates add "Documents/My Game" --name "My Template"
codea pull "Templates/My Template"
# edit files locally
codea push "Templates/My Template"
```

On filesystem-backed targets, local project creation does not use the remote templates collection. For local `codea new`, only the built-in `Modern` project layout is supported.

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

When creating a new project with `codea new "My Game" --template Modern`, the runtime is already set to `modern`.

To change the runtime of an existing collection-backed project, use:

```bash
codea runtime "My Game" modern
codea runtime "My Game" legacy
```

Do not rely on manually editing `Runtime Type` in `Info.plist` for device-backed projects. Use `codea runtime` so the target updates its runtime state correctly.

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

```bash
codea doc background --all                  # show all available docs (legacy + modern)
codea doc background --project "My Game"    # show only what's relevant to the project
codea doc background --modern               # force modern (Carbide) docs
codea doc background --legacy               # force legacy docs
codea search-doc storage                    # find all storage-related functions
codea search-doc "draw sprite"              # keyword search across help text
codea search-doc physics --modern           # modern-only results
codea search-doc physics --project "My Game"  # auto-select runtime from project
```

For broader reference, the online docs are organized by topic:

- **Legacy runtime index**: https://codea.io/reference/index.html
  - Animation: https://codea.io/reference/Animation.html
  - Craft (3D): https://codea.io/reference/Craft.html
  - Display & keyboard: https://codea.io/reference/Display.html
  - Graphics & assets: https://codea.io/reference/Graphics.html
  - Lua: https://codea.io/reference/Lua.html
  - Motion & location: https://codea.io/reference/Accelerometer.html
  - Network: https://codea.io/reference/Network.html
  - Parameters: https://codea.io/reference/Parameters.html
  - Physics: https://codea.io/reference/Physics.html
  - Shaders & Mesh: https://codea.io/reference/Shaders.html
  - Sounds: https://codea.io/reference/Sounds.html
  - Storage: https://codea.io/reference/Storage.html
  - Touch & input: https://codea.io/reference/Touch.html
  - Vector: https://codea.io/reference/Vector.html
- **Modern runtime (Carbide)**: https://twolivesleft.github.io/Codea4-Docs/

## Best Practices & Gotchas

### Asset Strings
Asset strings using the `Project:Asset` format (e.g., `readImage("Blobbo:empty")`) are **deprecated**. You should use static assets instead:
- **Correct**: `asset.empty` or `asset.wall`
- **Deprecated**: `readImage("Project:empty")`

### Missing `roundRect`
The `roundRect` function is not built-in to the Codea runtime. If your project requires rounded rectangles, you must implement the function yourself (e.g., using `mesh` or drawing multiple `rect` and `ellipse` calls).

---

## Modern Runtime (Carbide) Gotchas

### Text and Emojis
Emojis render as empty boxes with the default text renderer. To display emojis correctly, use `TEXT_NATIVE`:
```lua
style.push().textStyle(TEXT_NATIVE)
text("🎉 🚀 ❤️", x, y)
style.pop()
```
Note: `TEXT_NATIVE` uses the system font renderer; other `textStyle` flags (bold, italic, rich text, etc.) are ignored while it is active.

### 3D Coordinate System
Codea 4.x uses a **left-handed, +Z-forward** coordinate system (Metal convention). Objects must be at **positive Z** to be visible:
```lua
matrix.push()
    matrix.perspective(60)      -- camera at origin, looking toward +Z
    matrix.translate(0, 0, 4)   -- place object in front (positive Z)
    myMesh:draw()
matrix.pop()
-- Always wrap 3D in push/pop to restore 2D state afterwards
```

### Mesh: `vertices` vs `positions`
For custom mesh geometry, use `mesh.vertices = {...}` (not `mesh.positions`). `vertices` auto-sets the index buffer so the mesh draws immediately. `positions` leaves the index buffer untouched — useful for deforming an existing mesh, but on a fresh empty mesh nothing will be drawn.

### Mesh Lighting and Materials
Generated meshes (`mesh.sphere()`, `mesh.box()`, etc.) render **black** by default — they need a material or a light:
- **Unlit with color** (no light needed): `m.material = material.unlit(); m.material.color = color(r,g,b)`
- **Lit with shading**: `m.material = material.lit(); m.material.color = color(r,g,b)` + a directional light

Only `light.directional()` is currently supported in immediate mode — `light.point()` and `light.spot()` are defined but not yet implemented by the renderer.

Use `light.push(lt)` / `light.pop()` as **static functions**, not methods:
```lua
local lt = light.directional(vec3(1, -1, 1))
light.push(lt)
myMesh:draw()
light.pop()
```

## Notes for Agents

- Always `pull` before editing to get the latest files from device
- Use `sleep 2` or similar between `run` and `screenshot` to let the project render a frame
- `exec` requires a project to already be running
- Screenshot returns a PNG — save it and use vision to inspect results; do not open it in an external app unless the user explicitly asks
- `codea logs` accumulates all output since last `clear-logs`; use `--head 20` when Codea is spamming a repeated error to find the original cause
- File paths on device use `codea://` URIs internally; you don't need to deal with these directly
