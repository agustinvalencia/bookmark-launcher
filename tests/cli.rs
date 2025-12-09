use bmk::bookmarks::{
    Bookmark, Bookmarks, add_bookmark, delete_bookmark, get_all_tags, update_bookmark,
};

#[test]
fn test_add_bookmark() {
    let mut bookmarks: Bookmarks = Vec::new();

    let bookmark = Bookmark {
        name: "GitHub".to_string(),
        url: "https://github.com".to_string(),
        desc: "Code hosting".to_string(),
        tags: vec!["dev".to_string()],
    };

    add_bookmark(&mut bookmarks, bookmark);

    assert_eq!(bookmarks.len(), 1);
    assert_eq!(bookmarks[0].name, "GitHub");
    assert_eq!(bookmarks[0].url, "https://github.com");
    assert_eq!(bookmarks[0].desc, "Code hosting");
    assert_eq!(bookmarks[0].tags, vec!["dev"]);
}

#[test]
fn test_update_bookmark() {
    let mut bookmarks: Bookmarks = vec![Bookmark {
        name: "GitHub".to_string(),
        url: "https://github.com".to_string(),
        desc: "Old desc".to_string(),
        tags: vec![],
    }];

    let updated = Bookmark {
        name: "GitHub Updated".to_string(),
        url: "https://github.com/new".to_string(),
        desc: "New desc".to_string(),
        tags: vec!["updated".to_string()],
    };

    update_bookmark(&mut bookmarks, 0, updated);

    assert_eq!(bookmarks[0].name, "GitHub Updated");
    assert_eq!(bookmarks[0].url, "https://github.com/new");
    assert_eq!(bookmarks[0].desc, "New desc");
    assert_eq!(bookmarks[0].tags, vec!["updated"]);
}

#[test]
fn test_update_bookmark_out_of_bounds() {
    let mut bookmarks: Bookmarks = Vec::new();

    let bookmark = Bookmark {
        name: "Test".to_string(),
        url: "https://test.com".to_string(),
        desc: String::new(),
        tags: vec![],
    };

    // Should not panic, just do nothing
    update_bookmark(&mut bookmarks, 10, bookmark);
    assert!(bookmarks.is_empty());
}

#[test]
fn test_delete_bookmark() {
    let mut bookmarks: Bookmarks = vec![Bookmark {
        name: "GitHub".to_string(),
        url: "https://github.com".to_string(),
        desc: "Code hosting".to_string(),
        tags: vec![],
    }];

    delete_bookmark(&mut bookmarks, 0);

    assert!(bookmarks.is_empty());
}

#[test]
fn test_delete_bookmark_out_of_bounds() {
    let mut bookmarks: Bookmarks = Vec::new();

    // Should not panic, just do nothing
    delete_bookmark(&mut bookmarks, 10);
    assert!(bookmarks.is_empty());
}

#[test]
fn test_get_all_tags() {
    let bookmarks: Bookmarks = vec![
        Bookmark {
            name: "GitHub".to_string(),
            url: "https://github.com".to_string(),
            desc: "Code".to_string(),
            tags: vec!["dev".to_string(), "code".to_string()],
        },
        Bookmark {
            name: "Docs".to_string(),
            url: "https://docs.rs".to_string(),
            desc: "Docs".to_string(),
            tags: vec!["dev".to_string(), "rust".to_string()],
        },
    ];

    let tags = get_all_tags(&bookmarks);

    assert_eq!(tags, vec!["code", "dev", "rust"]);
}

#[test]
fn test_get_all_tags_empty() {
    let bookmarks: Bookmarks = Vec::new();

    let tags = get_all_tags(&bookmarks);

    assert!(tags.is_empty());
}

#[test]
fn test_bookmark_without_optional_fields() {
    let bookmark = Bookmark {
        name: "Test".to_string(),
        url: "https://test.com".to_string(),
        desc: String::new(),
        tags: vec![],
    };

    assert_eq!(bookmark.name, "Test");
    assert!(bookmark.desc.is_empty());
    assert!(bookmark.tags.is_empty());
}
