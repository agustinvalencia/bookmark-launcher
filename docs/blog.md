# Building a CLI bookmark manager

I spend most of my working between a terminal and a browser. I like the former, I don't enjoy the latter.
Moreover, most of my important tools and their configs I use from the terminal I have backed up in dotfiles in repositories, whereas for the browsers one has to either log-in into, and good luck if you use different browsers in different OSs.

Thus, I decided to write something that I could just launch from the terminal when I need.

This was the excuse I found to build the app described here, plus I've been wanting to play with `rust` for a while.

## Starting the project

```zsh
cargo new bookmarks
cd bookmarks
```

Let's establish the following folders tree.

```
bookmarker/
├── Cargo.toml
└── src/
    ├── main.rs         # Entry point, CLI parsing, and command dispatching
    ├── cli.rs          # Defines the structure of our command-line interface
    ├── bookmarks.rs    # Handles loading, saving, and modifying bookmarks
    └── lib.rs          # Makes our code testable as a library
```

## Defining Our Data

What is a bookmark? It's a URL, a description, and some tags. Let's model that. For this, we need a way to serialize and deserialize our data to and from a file (we'll use YAML). The `serde` and `serde_yaml` crates are for this. We'll also add `anyhow` for simple error handling.

```zsh
cargo add serde --features derive
cargo add serde_yaml
cargo add anyhow
```
