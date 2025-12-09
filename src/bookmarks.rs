use anyhow::{Context, Result};
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Bookmark {
    pub name: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub desc: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

pub type Bookmarks = Vec<Bookmark>;

fn get_bookmarks_path() -> Result<PathBuf> {
    let home = home_dir().context("Failed to find the home directory")?;
    let config_dir = home.join(".config").join("bmk");
    Ok(config_dir.join("bookmarks.yaml"))
}

pub fn load_bookmarks() -> Result<Bookmarks> {
    let path = get_bookmarks_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(&path)
        .with_context(|| format!("Failed to open bookmarks file at '{}'", path.display()))?;

    let bookmarks: Bookmarks = serde_yaml::from_reader(file)
        .with_context(|| format!("Failed to parse YAML from '{}'", path.display()))?;

    Ok(bookmarks)
}

pub fn save_bookmarks(bookmarks: &Bookmarks) -> Result<()> {
    let path = get_bookmarks_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create config directory at '{}'",
                parent.display()
            )
        })?;
    }

    let yaml_string = serde_yaml::to_string(bookmarks)?;

    fs::write(&path, yaml_string)
        .with_context(|| format!("Failed to write bookmarks to '{}'", path.display()))?;
    Ok(())
}

pub fn add_bookmark(bookmarks: &mut Bookmarks, bookmark: Bookmark) {
    bookmarks.push(bookmark);
}

pub fn update_bookmark(bookmarks: &mut Bookmarks, index: usize, bookmark: Bookmark) {
    if index < bookmarks.len() {
        bookmarks[index] = bookmark;
    }
}

pub fn delete_bookmark(bookmarks: &mut Bookmarks, index: usize) {
    if index < bookmarks.len() {
        bookmarks.remove(index);
    }
}

pub fn open_bookmark(url: &str) -> Result<()> {
    webbrowser::open(url).with_context(|| format!("Failed to open URL: {}", url))?;
    Ok(())
}

pub fn get_all_tags(bookmarks: &Bookmarks) -> Vec<String> {
    let mut tags: Vec<String> = bookmarks
        .iter()
        .flat_map(|b| b.tags.iter().cloned())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}
