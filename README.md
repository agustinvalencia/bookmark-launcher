# bmk

[![.github/workflows/ci.yml](https://github.com/agustinvalencia/bookmark-launcher/actions/workflows/ci.yml/badge.svg)](https://github.com/agustinvalencia/bookmark-launcher/actions/workflows/ci.yml)

A terminal-based bookmark manager with an interactive TUI. Manage your bookmarks in YAML and launch them in your default browser.

## Why?

I spend most of my time between a terminal and a browser. Most of my important tools and configs I back up in dotfiles repositories which I later `stow`, but browser bookmarks require logging into accounts or syncing across different browsers and operating systems.

This tool lets you manage bookmarks from the terminal with a simple YAML file that you can version control.

## Installation

Clone the repository and run (you'll need cargo):

```bash
cd <repository>
cargo install --path .
```

This installs the `bmk` binary into `~/.cargo/bin/`.

## Usage

### Interactive TUI

Simply run:

```bash
bmk
```

This opens an interactive TUI with your bookmarks.

### Direct Launch

You can also open a bookmark directly without the TUI by providing a search query:

```bash
bmk my-repo
bmk github
bmk "rust docs"
```

This uses the same fuzzy matching as the TUI search and opens the best matching bookmark in your browser. If no match is found, it exits with an error.

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` / `Down` | Move selection down |
| `k` / `Up` | Move selection up |
| `Enter` | Open selected bookmark in browser |
| `a` | Add new bookmark |
| `e` | Edit selected bookmark |
| `d` | Delete selected bookmark |
| `/` | Start searching (fuzzy search) |
| `t` | Filter by tag |
| `Esc` | Cancel current action / Clear filter |
| `q` | Quit |

### Search

Press `/` to activate fuzzy search. Type to filter bookmarks by name, URL, description, or tags. The list updates dynamically as you type. Press `Enter` to confirm search or `Esc` to cancel.

### Tag Filtering

Press `t` to open the tag filter. Select a tag to show only bookmarks with that tag. Press `Esc` to clear the filter.

## Configuration

Bookmarks are stored in `~/.config/bmk/bookmarks.yaml`. The file is created automatically when you add your first bookmark.

### YAML Format

```yaml
- name: GitHub
  url: https://github.com
  desc: Code repositories
  tags:
    - dev
    - code

- name: Rust Docs
  url: https://doc.rust-lang.org
  desc: Official Rust documentation
  tags:
    - dev
    - rust

- name: Example
  url: https://example.com
```

Each bookmark has:
- `name` (required): Display name for the bookmark
- `url` (required): The URL to open
- `desc` (optional): Description
- `tags` (optional): List of tags for filtering

## Roadmap

- [x] Create and delete bookmarks
- [x] Open browser with bmk
- [x] Interactive TUI
- [x] Fuzzy finding
- [x] Tag filtering
- [x] Direct launch from CLI (e.g., `bmk my-repo`)
- [ ] Make installation smoother (publish as crate)
