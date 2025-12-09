use anyhow::Result;
use bmk::bookmarks::open_bookmark;
use bmk::tui::run_tui_and_open;

fn main() -> Result<()> {
    if let Some(url) = run_tui_and_open()? {
        open_bookmark(&url)?;
    }
    Ok(())
}
