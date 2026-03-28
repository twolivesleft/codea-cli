# Packaging Strategy

This document describes the recommended release and packaging approach for `codea-cli`.

## Goals

- Users can install `codea` globally and run it from any directory.
- AI agents can install `codea` non-interactively.
- The primary release flow is automated from GitHub tags.
- macOS, Linux, WSL, and Windows all get first-class install paths.

## Recommended Distribution Shape

Use one build pipeline and publish several install surfaces from it:

1. GitHub Releases as the source of truth
   - publish versioned archives for:
     - `aarch64-apple-darwin`
     - `x86_64-apple-darwin`
     - `x86_64-unknown-linux-gnu`
     - `aarch64-unknown-linux-gnu` if you want ARM Linux support
     - `x86_64-pc-windows-msvc`
     - `aarch64-pc-windows-msvc` if you want ARM Windows support
2. Homebrew tap for macOS, Linux, and WSL
   - target command: `brew install twolivesleft/tap/codea`
3. Generated shell installer for macOS and Linux
   - target command: `curl ... | sh`
   - this is useful for AI agents and users who do not want Homebrew
4. Generated PowerShell installer for Windows
   - target command: `irm ... | iex`
   - this is useful for AI agents and automation on Windows
5. Windows MSI installer
   - target UX: download installer, click through, get `codea.exe` on `PATH`
6. Optional macOS `.pkg`
   - target UX: double-click installer, get `codea` on `PATH`

## Recommendation

Use `dist` as the release backbone.

Why:

- it is designed for Rust CLI distribution
- it can generate GitHub release automation
- it supports `homebrew`, `shell`, `powershell`, and `msi` installers from the same release pipeline
- it keeps the global command name aligned with the binary name, which is already `codea`

For `codea-cli`, the clean split is:

- `dist` for archives, Homebrew, shell installer, PowerShell installer, and Windows MSI
- a separate small macOS packaging step for `.pkg`

## What To Ship First

Ship these first:

1. GitHub release archives
2. Homebrew tap
3. shell installer
4. PowerShell installer
5. MSI

Add macOS `.pkg` after that.

Reasoning:

- Homebrew covers macOS, Linux, and WSL well.
- Shell and PowerShell installers make agent installs easy.
- MSI is the best default native installer on Windows.
- macOS `.pkg` is valuable, but it is not the fastest path to broad installability.

## Homebrew Plan

Homebrew is a good default for:

- macOS
- Linux
- WSL 2

Expected install command:

```bash
brew install twolivesleft/tap/codea
```

Implementation shape:

- create a tap repo such as `twolivesleft/homebrew-tap`
- let `dist` publish the formula into that tap
- have the formula install the `codea` binary globally through Homebrew

Current repo setup:

- `dist-workspace.toml` is configured with:
  - `installers = ["homebrew"]`
  - `tap = "twolivesleft/homebrew-tap"`
  - `formula = "codea"`
- `.github/workflows/release.yml` is generated and includes a `publish-homebrew-formula` job

Remaining setup outside this repo:

- create the GitHub repository `twolivesleft/homebrew-tap`
- create a GitHub App with `Contents: Read and write`
- install that app on `twolivesleft/homebrew-tap`
- add a repository variable named `HOMEBREW_APP_ID` in `twolivesleft/codea-cli`
- add a repository secret named `HOMEBREW_APP_PRIVATE_KEY` in `twolivesleft/codea-cli`
- add package license metadata before the first public release

Notes:

- Homebrew works best from its default prefixes on macOS and Linux.
- WSL 2 is the supported WSL path for Homebrew; do not optimize around WSL 1.

## Windows Plan

There are really two Windows tracks:

1. CLI-first install for agents and automation
   - PowerShell installer
2. Native installer for normal users
   - MSI

The installed binary should still be `codea.exe`, accessible as `codea`.

After MSI is stable, submit to WinGet as a separate publishing step so users can also do:

```powershell
winget install TwoLivesLeft.codea
```

WinGet is worth doing, but it should be treated as a distribution channel layered on top of your own release artifacts, not as the primary build system.

## macOS `.pkg` Plan

If you want a double-click installer, build a signed `.pkg` that installs:

- the binary payload under a stable location such as `/usr/local/lib/codea-cli`
- a symlink or wrapper at `/usr/local/bin/codea`
- optionally `Carbide.app` into `/Applications`

Keep the `.pkg` separate from Homebrew:

- Homebrew users should continue to install with `brew`
- `.pkg` users should get a self-contained install without Homebrew

The `.pkg` should be signed with a Developer ID Installer certificate before public release.

## Carbide On macOS

Including `Carbide.app` on macOS is feasible, but it changes the best packaging shape.

### `.pkg`

This is the easiest place to bundle both products together.

A macOS installer package can install:

- `codea` onto `PATH`
- `Carbide.app` into `/Applications`

This gives you the most polished "one download" macOS experience.

### Homebrew

For Homebrew, do not try to force the app into the same cross-platform formula as the CLI.

Recommended split:

- keep `codea` as a formula for macOS, Linux, and WSL
- publish `Carbide.app` as a macOS-only cask
- optionally add a meta cask for macOS that depends on both

Why:

- formulas are the right shape for CLI tools
- casks are the right shape for `.app` bundles and can also install linked binaries
- keeping the CLI formula separate preserves Linux and WSL support cleanly

Possible macOS Homebrew commands:

```bash
brew install twolivesleft/tap/codea
brew install --cask twolivesleft/tap/carbide
```

Optional bundled macOS command via a meta cask:

```bash
brew install --cask twolivesleft/tap/codea-suite
```

That cask would install `Carbide.app` and declare a dependency on the `codea` formula.

### Practical Recommendation

If you want the simplest long-term packaging model:

1. Keep `codea` as the primary cross-platform package.
2. Add `Carbide.app` to the macOS `.pkg`.
3. Publish a separate `carbide` Homebrew cask later.
4. Only add a combined Homebrew meta cask if users strongly want a single macOS `brew install` entrypoint.

## Release Automation Plan

Use GitHub tags as the release trigger.

Target release flow:

1. bump version in `Cargo.toml`
2. create and push tag like `v0.2.0`
3. GitHub Actions builds all target binaries
4. release artifacts are uploaded to GitHub Releases
5. Homebrew formula is updated in the tap
6. install scripts and MSI are attached to the release
7. optional follow-up jobs publish to WinGet and build/sign the macOS `.pkg`

## Required Metadata Before Packaging

The crate is still missing some metadata that package managers and installers expect.

Add these to `Cargo.toml` before wiring up release automation:

- `description`
- `repository`
- `homepage`
- `license`
- `authors`

Why they matter:

- Homebrew wants clear homepage and license information
- MSI needs a manufacturer, which `dist` derives from `authors`
- good metadata also improves release pages and generated installers

## Suggested First Implementation Pass

1. Add package metadata to `Cargo.toml`.
2. Create the tap repo: `twolivesleft/homebrew-tap`.
3. Install `dist`.
4. Run `dist init`.
5. Enable these installers:
   - `homebrew`
   - `shell`
   - `powershell`
   - `msi`
6. Point the Homebrew tap setting at `twolivesleft/homebrew-tap`.
7. Commit the generated `dist` config and GitHub workflow.
8. Test a tag-based release on a prerelease version.

## Suggested User-Facing Install Commands

macOS, Linux, WSL:

```bash
brew install twolivesleft/tap/codea
```

Agent-friendly macOS and Linux fallback:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/twolivesleft/codea-cli/releases/latest/download/codea-installer.sh | sh
```

Windows PowerShell:

```powershell
powershell -c "irm https://github.com/twolivesleft/codea-cli/releases/latest/download/codea-installer.ps1 | iex"
```

Windows native installer:

- download the latest `codea-<version>-x86_64-pc-windows-msvc.msi`

## Open Decisions

- whether to support ARM Linux in the first release wave
- whether to publish WinGet immediately or after MSI has baked for a release or two
- where the macOS `.pkg` should install the binary and whether it should create a symlink or a wrapper script
- whether to notarize the macOS `.pkg` immediately or after unsigned internal testing
