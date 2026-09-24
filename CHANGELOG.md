# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-09-24

### Added

- `codea screen-size [preset]` reads or sets the screen size of the running
  project's viewer, so a project can be checked at another aspect ratio or
  orientation. Presets are `match-display`, `iphone-portrait`,
  `iphone-landscape`, `tv` and `square`. Reading it also reports the viewer's
  current pixel size, taken from its laid-out bounds rather than the preset's
  nominal size. Needs a Codea build that
  provides the `setScreenSize` and `getScreenSize` Air Code tools.

## [0.1.6] - 2026-04-02

### Added

- MCP usage examples in the README.

### Fixed

- `codea status` reports the paused and idle timer states correctly.

### Changed

- Reworked the release mechanism.

## [0.1.5] - 2026-03-30

### Added

- WiX template for the Windows MSI installer.

## [0.1.4] - 2026-03-29

### Added

- Windows installer packaging.

## [0.1.3] - 2026-03-29

### Added

- Update notifications, cached between runs.

## [0.1.2] - 2026-03-29

### Added

- A progress spinner while discovering devices.

## [0.1.1] - 2026-03-29

### Changed

- More reliable fallback when creating a local project.

## [0.1.0] - 2026-03-29

First release. Connects to a Codea or Carbide runtime over Air Code to discover
hosts, save connection profiles, manage projects, collections, templates and
dependencies, run and stop projects, execute Lua, inspect and change a project's
runtime type, query the API docs and autocomplete data, capture screenshots,
stream logs, push and pull project files, and create local projects. Ships
Homebrew, PowerShell and MSI installers.

[Unreleased]: https://github.com/twolivesleft/codea-cli/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/twolivesleft/codea-cli/compare/v0.1.6...v0.2.0
[0.1.6]: https://github.com/twolivesleft/codea-cli/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/twolivesleft/codea-cli/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/twolivesleft/codea-cli/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/twolivesleft/codea-cli/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/twolivesleft/codea-cli/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/twolivesleft/codea-cli/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/twolivesleft/codea-cli/releases/tag/v0.1.0
