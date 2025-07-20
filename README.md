# Bookmarker

[![.github/workflows/ci.yml](https://github.com/agustinvalencia/bookmark-launcher/actions/workflows/ci.yml/badge.svg)](https://github.com/agustinvalencia/bookmark-launcher/actions/workflows/ci.yml)

Acknowledgement: I started this as a small project to get my hands dirty on rust. Any feedback is most welcome!

I spend most of my working between a terminal and a browser. I like the former, I don't enjoy the latter.
Moreover, most of my important tools and their configs I use from the terminal I have backed up in dotfiles in repositories which I later `stow`, whereas for the browsers one has to either log-in into, and good luck if you use different browsers in different OSs.

Thus, I decided to write something that I could just launch from the terminal when I need.

This was the excuse I found to build the app described here, plus I've been wanting to play with `rust` for a while.

# Installation

Clone the repository and run (you'll need cargo)

```bash
cd <repository>
cargo install --path .
```

This should install this crate into yours `~/.cargo/`

(I should prolly improve this into a public crate or so)

# How to use

## Help

```bash
 bookmarker --help

A bookmark manager

Usage: bookmarker <COMMAND>

Commands:
  list
  add
  open
  delete
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Adding a bookmark

```bash
 bookmarker add --help

Usage: bookmarker add [OPTIONS] --desc <DESC> <KEY> <URL>

Arguments:
  <KEY>
  <URL>

Options:
  -d, --desc <DESC>
  -t, --tags <TAGS>
  -h, --help         Print help
```

So as example, let's create a bookmark for github

```bash
 bookmarker add gh https://github.com --desc "repositories and all" --tags dev,code

 > Bookmark 'gh' added.
```

This automatically creates `~/.config/bookmarker/bookmarks.yaml` in your system, so you are free to stow it and even edit it by hand.

## Listing bookmarks

```bash
 bookmarker list
Key        | URL                                      | Description                              | Tags
--------------------------------------------------------------------------------------------------------------
gh         | https://github.com                       | repositories and all                     | dev, code
```

## Launching a bookmark

```bash
 bookmarker open --help
Usage: bookmarker open <KEY>

Arguments:
  <KEY>

Options:
  -h, --help  Print help
```

Let's open our recently added bookmark to github:

```bash
 bookmarker open gh
Opening 'gh'  (https://github.com)
```

(Your default browser has now opened a new tab on github website).

# Roadmap

- [x] Create and delete bookmarks
- [x] Open browser with Bookmarker
- [ ] Make installation smoother
- [ ] Refactor into an interactive TUI
- [ ] Implement fuzzy finding
