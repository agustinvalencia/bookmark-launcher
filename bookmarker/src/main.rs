use anyhow::Result;
use bookmarker::bookmarks;
use bookmarker::cli::{Cli, Commands};
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { tag } => {
            bookmarks::handle_list_command(tag)?;
        }
        Commands::Open { key } => {
            bookmarks::handle_open_command(&key)?;
        }
        Commands::Add {
            key,
            url,
            desc,
            tags,
        } => {
            bookmarks::handle_add_command(key, url, desc, tags)?;
        }
        Commands::Delete { key } => {
            println!("Deleting bookmark: {}", key);
        }
    }

    Ok(())
}
