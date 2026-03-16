# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.1] - 2026-03-16

### Added

- `--version` / `-V` flag

## [0.1.0] - 2026-03-16

### Added

- Branch list with current branch highlight
- Switch branch (`Enter`) with auto-exit
- Copy branch name to clipboard (`y`)
- Delete branch with safety confirmation (`d`)
  - Merged: `y/n` confirmation
  - Unmerged: requires `d` again to force delete
- Create new branch (`n`) with source branch indicator
- Rename branch (`r`) with pre-filled current name
- Incremental filter (`/`)
- Commit preview in right pane
- Persistent keybinding help bar
