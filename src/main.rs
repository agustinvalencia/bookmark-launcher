use anyhow::Result;
use bookmarker::bookmarks::{
    handle_add_command, handle_delete_command, handle_list_command, handle_open_command,
};
use bookmarker::cli::{Cli, Commands};
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { tag } => handle_list_command(tag)?,
        Commands::Open { key } => handle_open_command(&key)?,
        Commands::Delete { key } => handle_delete_command(&key)?,
        Commands::Add {
            key,
            url,
            desc,
            tags,
        } => handle_add_command(key, url, desc, tags)?,
    }

    Ok(())
}
