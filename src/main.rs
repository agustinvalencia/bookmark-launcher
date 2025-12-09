use anyhow::Result;
use bookmarker::bookmarks::open_bookmark;
use bookmarker::tui::run_tui_and_open;

fn main() -> Result<()> {
    if let Some(url) = run_tui_and_open()? {
        open_bookmark(&url)?;
    }
    Ok(())
}
