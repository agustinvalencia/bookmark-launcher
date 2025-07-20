use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::PathBuf;
use webbrowser;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Bookmark {
    pub url: String,
    pub desc: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub type Bookmarks = HashMap<String, Bookmark>;

fn get_bookmarks_path() -> Result<PathBuf> {
    Ok(PathBuf::from("bookmarks.yaml"))
}

pub fn load_bookmarks() -> Result<Bookmarks> {
    let path = get_bookmarks_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let file = File::open(&path)
        .with_context(|| format!("Failed to open bookmarks file at '{}'", path.display()))?;

    let bookmarks: Bookmarks = serde_yaml::from_reader(file)
        .with_context(|| format!("Failed to parse YAML from '{}'", path.display()))?;

    Ok(bookmarks)
}

pub fn save_bookmarks(bookmarks: &Bookmarks) -> Result<()> {
    let path = get_bookmarks_path()?;
    let yaml_string = serde_yaml::to_string(bookmarks)?;
    fs::write(&path, yaml_string)
        .with_context(|| format!("Failed to write bookmarks to '{}'", path.display()))?;
    Ok(())
}

pub fn handle_list_command(tag: Option<String>) -> Result<()> {
    let bookmarks = load_bookmarks()?;

    println!(
        "{:<10} | {:<40} | {:<40} | {}",
        "Key", "URL", "Description", "Tags"
    );
    println!("{}", "-".repeat(110));

    for (key, bookmark) in &bookmarks {
        if let Some(filter_tag) = &tag {
            if !bookmark
                .tags
                .iter()
                .any(|t| t.eq_ignore_ascii_case(filter_tag))
            {
                continue;
            }
        }
        println!(
            "{:<10} | {:<40} | {:<40} | {}",
            key,
            bookmark.url,
            bookmark.desc,
            bookmark.tags.join(", ")
        );
    }
    Ok(())
}

pub fn handle_add_command(
    key: String,
    url: String,
    desc: String,
    tags: Option<Vec<String>>,
) -> Result<()> {
    let mut bookmarks = load_bookmarks()?;

    if bookmarks.contains_key(&key) {
        anyhow::bail!("Bookmark with key '{}' already exists.", key);
    }

    let new_bookmark = Bookmark {
        url,
        desc,
        tags: tags.unwrap_or_default(),
    };
    bookmarks.insert(key.clone(), new_bookmark);
    save_bookmarks(&bookmarks)?;

    println!(" > Bookmark '{}' added.", key);
    Ok(())
}

pub fn handle_open_command(key: &str) -> Result<()> {
    let bookmarks = load_bookmarks()?;

    if let Some(bookmark) = bookmarks.get(key) {
        println!("Opening '{}'  ({})", key, bookmark.url);
        webbrowser::open(&bookmark.url)
            .with_context(|| format!("Failed to open URL for key '{}'", key))?;
    } else {
        anyhow::bail!("Bookmark with key '{}' not found.", key);
    }
    Ok(())
}

pub fn handle_delete_command(key: &str) -> Result<()> {
    let mut bookmarks = load_bookmarks()?;

    if bookmarks.get(key).is_some() {
        bookmarks.remove(key);
        save_bookmarks(&bookmarks)?;
        println!("Bookmark '{}' deleted", key);
    } else {
        anyhow::bail!("Bookmark with key '{}' not found.", key);
    }
    Ok(())
}
