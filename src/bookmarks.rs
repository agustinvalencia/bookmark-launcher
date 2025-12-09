use anyhow::{Context, Result};
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Bookmark {
    pub url: String,
    pub desc: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub type Bookmarks = HashMap<String, Bookmark>;

fn get_bookmarks_path() -> Result<PathBuf> {
    let home = home_dir().context("Failed to find the home directory")?;
    let config_dir = home.join(".config").join("bookmarker");
    Ok(config_dir.join("bookmarks.yaml"))
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

pub fn add_bookmark(
    bookmarks: &mut Bookmarks,
    key: String,
    url: String,
    desc: String,
    tags: Vec<String>,
) -> Result<()> {
    if bookmarks.contains_key(&key) {
        anyhow::bail!("Bookmark with key '{}' already exists.", key);
    }

    let new_bookmark = Bookmark { url, desc, tags };
    bookmarks.insert(key, new_bookmark);
    Ok(())
}

pub fn update_bookmark(
    bookmarks: &mut Bookmarks,
    key: &str,
    url: String,
    desc: String,
    tags: Vec<String>,
) -> Result<()> {
    if !bookmarks.contains_key(key) {
        anyhow::bail!("Bookmark with key '{}' not found.", key);
    }

    let bookmark = Bookmark { url, desc, tags };
    bookmarks.insert(key.to_string(), bookmark);
    Ok(())
}

pub fn delete_bookmark(bookmarks: &mut Bookmarks, key: &str) -> Result<()> {
    if bookmarks.remove(key).is_none() {
        anyhow::bail!("Bookmark with key '{}' not found.", key);
    }
    Ok(())
}

pub fn open_bookmark(url: &str) -> Result<()> {
    webbrowser::open(url).with_context(|| format!("Failed to open URL: {}", url))?;
    Ok(())
}

pub fn get_all_tags(bookmarks: &Bookmarks) -> Vec<String> {
    let mut tags: Vec<String> = bookmarks
        .values()
        .flat_map(|b| b.tags.iter().cloned())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}
