# githop

A lightweight TUI for quick git branch operations. No full-screen git client — just branches, fast.

## Install

### Homebrew

```bash
brew install tonegawa07/tap/githop
```

### Cargo

```bash
cargo install githop
```

## Usage

Run `githop` in any git repository:

```sh
githop
```

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `k`, `↑` / `↓` | Navigate branches |
| `Enter` | Switch to branch (and exit) |
| `y` | Copy branch name to clipboard |
| `d` | Delete branch (with confirmation) |
| `n` | Create new branch |
| `r` | Rename branch |
| `/` | Filter branches |
| `q` / `Esc` | Quit |

### Layout

```
┌─ Branches ──────────┬─ Preview ─────────────┐
│ > main            * │ abc1234 fix: bug       │
│   feature/login     │ def5678 feat: login    │
│   feature/signup    │ ghi9012 refactor: auth │
│                     │                        │
├─────────────────────┴────────────────────────┤
│ Copied 'main'                                │
│ [Enter]switch [y]copy [d]delete [n]new [q]quit│
└──────────────────────────────────────────────┘
```

- **Left pane**: Local branches. Current branch marked with `*`.
- **Right pane**: Recent commits for the selected branch.
- **Bottom**: Status messages and keybinding help (always visible).

### Delete safety

- **Merged branches**: Simple `y/n` confirmation, uses `git branch -d`.
- **Unmerged branches**: Warning displayed. Press `d` again to force delete (`git branch -D`), preventing accidental `y` key presses.

### Filter

Press `/` to start typing. Branches are filtered incrementally. `Esc` to clear and show all, `Enter` to keep the filter active.

### Create

Press `n` to create a new branch. Shows which branch you're branching from (e.g., `Create from 'main':`).

## Requirements

- Git
- macOS or Linux
- Clipboard support:
  - macOS: `pbcopy` (pre-installed)
  - Linux (Wayland): `wl-copy`
  - Linux (X11): `xclip`

## License

MIT
