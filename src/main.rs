use anyhow::Result;
use bmk::bookmarks::{load_bookmarks, open_bookmark};
use bmk::tui::{find_best_match, run_tui_and_open};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // If a query argument is provided, try to open the best matching bookmark directly
    if args.len() > 1 {
        let query = args[1..].join(" ");
        let bookmarks = load_bookmarks()?;

        if let Some(url) = find_best_match(&bookmarks, &query) {
            open_bookmark(&url)?;
        } else {
            eprintln!("No bookmark found matching: {}", query);
            std::process::exit(1);
        }
    } else {
        // No arguments: launch the TUI
        if let Some(url) = run_tui_and_open()? {
            open_bookmark(&url)?;
        }
    }

    Ok(())
}
