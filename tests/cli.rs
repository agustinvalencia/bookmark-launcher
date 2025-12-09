use bookmarker::bookmarks::{
    Bookmark, Bookmarks, add_bookmark, delete_bookmark, get_all_tags, update_bookmark,
};
use std::collections::HashMap;

#[test]
fn test_add_bookmark() {
    let mut bookmarks: Bookmarks = HashMap::new();

    add_bookmark(
        &mut bookmarks,
        "gh".to_string(),
        "https://github.com".to_string(),
        "Code hosting".to_string(),
        vec!["dev".to_string()],
    )
    .unwrap();

    assert!(bookmarks.contains_key("gh"));
    assert_eq!(bookmarks["gh"].url, "https://github.com");
    assert_eq!(bookmarks["gh"].desc, "Code hosting");
    assert_eq!(bookmarks["gh"].tags, vec!["dev"]);
}

#[test]
fn test_add_duplicate_bookmark_fails() {
    let mut bookmarks: Bookmarks = HashMap::new();

    add_bookmark(
        &mut bookmarks,
        "gh".to_string(),
        "https://github.com".to_string(),
        "Code hosting".to_string(),
        vec![],
    )
    .unwrap();

    let result = add_bookmark(
        &mut bookmarks,
        "gh".to_string(),
        "https://different.com".to_string(),
        "Different".to_string(),
        vec![],
    );

    assert!(result.is_err());
}

#[test]
fn test_update_bookmark() {
    let mut bookmarks: Bookmarks = HashMap::new();
    bookmarks.insert(
        "gh".to_string(),
        Bookmark {
            url: "https://github.com".to_string(),
            desc: "Old desc".to_string(),
            tags: vec![],
        },
    );

    update_bookmark(
        &mut bookmarks,
        "gh",
        "https://github.com/new".to_string(),
        "New desc".to_string(),
        vec!["updated".to_string()],
    )
    .unwrap();

    assert_eq!(bookmarks["gh"].url, "https://github.com/new");
    assert_eq!(bookmarks["gh"].desc, "New desc");
    assert_eq!(bookmarks["gh"].tags, vec!["updated"]);
}

#[test]
fn test_update_nonexistent_bookmark_fails() {
    let mut bookmarks: Bookmarks = HashMap::new();

    let result = update_bookmark(
        &mut bookmarks,
        "nonexistent",
        "https://example.com".to_string(),
        "Desc".to_string(),
        vec![],
    );

    assert!(result.is_err());
}

#[test]
fn test_delete_bookmark() {
    let mut bookmarks: Bookmarks = HashMap::new();
    bookmarks.insert(
        "gh".to_string(),
        Bookmark {
            url: "https://github.com".to_string(),
            desc: "Code hosting".to_string(),
            tags: vec![],
        },
    );

    delete_bookmark(&mut bookmarks, "gh").unwrap();

    assert!(!bookmarks.contains_key("gh"));
}

#[test]
fn test_delete_nonexistent_bookmark_fails() {
    let mut bookmarks: Bookmarks = HashMap::new();

    let result = delete_bookmark(&mut bookmarks, "nonexistent");

    assert!(result.is_err());
}

#[test]
fn test_get_all_tags() {
    let mut bookmarks: Bookmarks = HashMap::new();
    bookmarks.insert(
        "gh".to_string(),
        Bookmark {
            url: "https://github.com".to_string(),
            desc: "Code".to_string(),
            tags: vec!["dev".to_string(), "code".to_string()],
        },
    );
    bookmarks.insert(
        "docs".to_string(),
        Bookmark {
            url: "https://docs.rs".to_string(),
            desc: "Docs".to_string(),
            tags: vec!["dev".to_string(), "rust".to_string()],
        },
    );

    let tags = get_all_tags(&bookmarks);

    assert_eq!(tags, vec!["code", "dev", "rust"]);
}

#[test]
fn test_get_all_tags_empty() {
    let bookmarks: Bookmarks = HashMap::new();

    let tags = get_all_tags(&bookmarks);

    assert!(tags.is_empty());
}
