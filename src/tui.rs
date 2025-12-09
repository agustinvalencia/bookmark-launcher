use crate::bookmarks::{
    Bookmarks, add_bookmark, delete_bookmark, get_all_tags, load_bookmarks, save_bookmarks,
    update_bookmark,
};
use cursive::Cursive;
use cursive::event::Key;
use cursive::theme::{BorderStyle, PaletteColor, Theme};
use cursive::traits::*;
use cursive::views::{Dialog, EditView, LinearLayout, OnEventView, Panel, SelectView, TextView};
use std::cell::RefCell;
use std::rc::Rc;

const BOOKMARK_LIST_NAME: &str = "bookmark_list";
const SEARCH_INPUT_NAME: &str = "search_input";

// Catppuccin Mocha palette
mod catppuccin {
    use cursive::theme::Color;

    pub const BASE: Color = Color::Rgb(30, 30, 46);
    pub const CRUST: Color = Color::Rgb(17, 17, 27);
    pub const TEXT: Color = Color::Rgb(205, 214, 244);
    pub const SUBTEXT0: Color = Color::Rgb(166, 173, 200);
    pub const SURFACE0: Color = Color::Rgb(49, 50, 68);
    pub const SURFACE1: Color = Color::Rgb(69, 71, 90);
    pub const OVERLAY0: Color = Color::Rgb(108, 112, 134);
    pub const LAVENDER: Color = Color::Rgb(180, 190, 254);
    pub const MAUVE: Color = Color::Rgb(203, 166, 247);
    pub const PINK: Color = Color::Rgb(245, 194, 231);
}

fn catppuccin_theme() -> Theme {
    let mut theme = Theme {
        shadow: false,
        borders: BorderStyle::Simple,
        ..Default::default()
    };

    theme.palette[PaletteColor::Background] = catppuccin::BASE;
    theme.palette[PaletteColor::View] = catppuccin::BASE;
    theme.palette[PaletteColor::Primary] = catppuccin::TEXT;
    theme.palette[PaletteColor::Secondary] = catppuccin::SUBTEXT0;
    theme.palette[PaletteColor::Tertiary] = catppuccin::OVERLAY0;
    theme.palette[PaletteColor::TitlePrimary] = catppuccin::MAUVE;
    theme.palette[PaletteColor::TitleSecondary] = catppuccin::PINK;
    theme.palette[PaletteColor::Highlight] = catppuccin::SURFACE1;
    theme.palette[PaletteColor::HighlightInactive] = catppuccin::SURFACE0;
    theme.palette[PaletteColor::HighlightText] = catppuccin::LAVENDER;
    theme.palette[PaletteColor::Shadow] = catppuccin::CRUST;

    theme
}

pub fn run_tui() -> anyhow::Result<()> {
    let bookmarks = load_bookmarks()?;
    let bookmarks = Rc::new(RefCell::new(bookmarks));
    let filter = Rc::new(RefCell::new(String::new()));
    let tag_filter = Rc::new(RefCell::new(Option::<String>::None));
    let search_active = Rc::new(RefCell::new(false));

    let mut siv = cursive::default();

    siv.set_theme(catppuccin_theme());

    siv.set_user_data(AppState {
        bookmarks: Rc::clone(&bookmarks),
        filter: Rc::clone(&filter),
        tag_filter: Rc::clone(&tag_filter),
        search_active: Rc::clone(&search_active),
    });

    build_main_view(&mut siv);

    siv.add_global_callback('q', |s| {
        let state = s.user_data::<AppState>().unwrap();
        if !*state.search_active.borrow() {
            s.quit();
        }
    });

    siv.add_global_callback(Key::Esc, |s| {
        let state = s.user_data::<AppState>().unwrap();
        if *state.search_active.borrow() {
            *state.search_active.borrow_mut() = false;
            *state.filter.borrow_mut() = String::new();
            build_main_view(s);
        } else {
            s.quit();
        }
    });

    siv.run();
    Ok(())
}

struct AppState {
    bookmarks: Rc<RefCell<Bookmarks>>,
    filter: Rc<RefCell<String>>,
    tag_filter: Rc<RefCell<Option<String>>>,
    search_active: Rc<RefCell<bool>>,
}

fn build_main_view(siv: &mut Cursive) {
    siv.pop_layer();

    let state = siv.user_data::<AppState>().unwrap();
    let bookmarks = state.bookmarks.borrow();
    let filter = state.filter.borrow().clone();
    let tag_filter = state.tag_filter.borrow().clone();
    let search_active = *state.search_active.borrow();

    let mut select = SelectView::<String>::new().on_submit(on_select_bookmark);

    let mut items: Vec<(String, String, i64)> = bookmarks
        .iter()
        .filter_map(|(key, bm)| {
            let matches_tag = tag_filter
                .as_ref()
                .is_none_or(|t| bm.tags.iter().any(|tag| tag.eq_ignore_ascii_case(t)));

            if !matches_tag {
                return None;
            }

            let score = if filter.is_empty() {
                0
            } else {
                fuzzy_score(&filter, key, &bm.url, &bm.desc, &bm.tags)
            };

            if !filter.is_empty() && score < 0 {
                return None;
            }

            let tags_str = if bm.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", bm.tags.join(", "))
            };
            let label = format!(
                "{:<12} {:<50} {}{}",
                key,
                truncate(&bm.url, 50),
                truncate(&bm.desc, 30),
                tags_str
            );
            Some((label, key.clone(), score))
        })
        .collect();

    // Sort by score (descending) then by key (ascending)
    items.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(&b.1)));

    for (label, key, _) in items {
        select.add_item(label, key);
    }

    if select.is_empty() {
        if filter.is_empty() {
            select.add_item("(no bookmarks - press 'a' to add one)", String::new());
        } else {
            select.add_item("(no matches)", String::new());
        }
    }

    let select = select.with_name(BOOKMARK_LIST_NAME);

    let tag_display = if let Some(t) = tag_filter.as_ref() {
        format!(" [tag: {}]", t)
    } else {
        String::new()
    };

    let title = format!("Bookmarks{}", tag_display);

    drop(bookmarks);

    let select = OnEventView::new(select)
        .on_event('a', |s| {
            let state = s.user_data::<AppState>().unwrap();
            if !*state.search_active.borrow() {
                show_add_dialog(s);
            }
        })
        .on_event('e', |s| {
            let state = s.user_data::<AppState>().unwrap();
            if !*state.search_active.borrow() {
                show_edit_dialog(s);
            }
        })
        .on_event('d', |s| {
            let state = s.user_data::<AppState>().unwrap();
            if !*state.search_active.borrow() {
                show_delete_dialog(s);
            }
        })
        .on_event('/', |s| {
            let state = s.user_data::<AppState>().unwrap();
            if !*state.search_active.borrow() {
                *state.search_active.borrow_mut() = true;
                build_main_view(s);
                s.focus_name(SEARCH_INPUT_NAME).ok();
            }
        })
        .on_event('t', |s| {
            let state = s.user_data::<AppState>().unwrap();
            if !*state.search_active.borrow() {
                show_tag_filter_dialog(s);
            }
        });

    let help_text = if search_active {
        "Type to filter | Enter: Select | Esc: Cancel search"
    } else {
        "Enter: Open | a: Add | e: Edit | d: Delete | /: Search | t: Tags | q: Quit"
    };

    let mut layout =
        LinearLayout::vertical().child(Panel::new(select.scrollable().full_screen()).title(title));

    if search_active {
        let search_input = EditView::new()
            .content(&filter)
            .on_edit(|s, text, _| {
                let state = s.user_data::<AppState>().unwrap();
                *state.filter.borrow_mut() = text.to_string();
                update_bookmark_list(s);
            })
            .on_submit(|s, _| {
                // When Enter is pressed, try to open the selected bookmark
                if let Some(key) = get_selected_key(s)
                    && !key.is_empty()
                {
                    on_select_bookmark(s, &key);
                }
            })
            .with_name(SEARCH_INPUT_NAME)
            .full_width();

        layout.add_child(
            LinearLayout::horizontal()
                .child(TextView::new("> "))
                .child(search_input),
        );
    }

    layout.add_child(TextView::new(help_text));

    siv.add_fullscreen_layer(layout);

    if search_active {
        siv.focus_name(SEARCH_INPUT_NAME).ok();
    }
}

fn update_bookmark_list(siv: &mut Cursive) {
    let (items, filter_empty) = {
        let state = siv.user_data::<AppState>().unwrap();
        let bookmarks = state.bookmarks.borrow();
        let filter = state.filter.borrow().clone();
        let tag_filter = state.tag_filter.borrow().clone();

        let mut items: Vec<(String, String, i64)> = bookmarks
            .iter()
            .filter_map(|(key, bm)| {
                let matches_tag = tag_filter
                    .as_ref()
                    .is_none_or(|t| bm.tags.iter().any(|tag| tag.eq_ignore_ascii_case(t)));

                if !matches_tag {
                    return None;
                }

                let score = if filter.is_empty() {
                    0
                } else {
                    fuzzy_score(&filter, key, &bm.url, &bm.desc, &bm.tags)
                };

                if !filter.is_empty() && score < 0 {
                    return None;
                }

                let tags_str = if bm.tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", bm.tags.join(", "))
                };
                let label = format!(
                    "{:<12} {:<50} {}{}",
                    key,
                    truncate(&bm.url, 50),
                    truncate(&bm.desc, 30),
                    tags_str
                );
                Some((label, key.clone(), score))
            })
            .collect();

        items.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(&b.1)));

        (items, filter.is_empty())
    };

    siv.call_on_name(BOOKMARK_LIST_NAME, |view: &mut SelectView<String>| {
        view.clear();

        for (label, key, _) in items {
            view.add_item(label, key);
        }

        if view.is_empty() {
            if filter_empty {
                view.add_item("(no bookmarks - press 'a' to add one)", String::new());
            } else {
                view.add_item("(no matches)", String::new());
            }
        }
    });
}

/// Fuzzy matching score - returns negative if no match, higher scores are better matches
fn fuzzy_score(pattern: &str, key: &str, url: &str, desc: &str, tags: &[String]) -> i64 {
    let pattern_lower = pattern.to_lowercase();
    let pattern_chars: Vec<char> = pattern_lower.chars().collect();

    // Check each field and return the best score
    let key_score = fuzzy_match_score(&pattern_chars, &key.to_lowercase());
    let url_score = fuzzy_match_score(&pattern_chars, &url.to_lowercase());
    let desc_score = fuzzy_match_score(&pattern_chars, &desc.to_lowercase());
    let tags_score = tags
        .iter()
        .map(|t| fuzzy_match_score(&pattern_chars, &t.to_lowercase()))
        .max()
        .unwrap_or(-1);

    // Prioritize key matches, then url, then desc, then tags
    if key_score >= 0 {
        key_score + 1000
    } else if url_score >= 0 {
        url_score + 500
    } else if desc_score >= 0 {
        desc_score + 100
    } else if tags_score >= 0 {
        tags_score
    } else {
        -1
    }
}

/// Score a fuzzy match - returns -1 if no match, otherwise a score based on match quality
fn fuzzy_match_score(pattern: &[char], text: &str) -> i64 {
    if pattern.is_empty() {
        return 0;
    }

    let text_chars: Vec<char> = text.chars().collect();
    let mut pattern_idx = 0;
    let mut score: i64 = 0;
    let mut last_match_idx: Option<usize> = None;
    let mut consecutive_bonus = 0;

    for (i, &c) in text_chars.iter().enumerate() {
        if pattern_idx < pattern.len() && c == pattern[pattern_idx] {
            // Bonus for consecutive matches
            if let Some(last) = last_match_idx {
                if i == last + 1 {
                    consecutive_bonus += 10;
                } else {
                    consecutive_bonus = 0;
                }
            }

            // Bonus for matching at word boundaries
            let word_boundary_bonus = if i == 0
                || text_chars
                    .get(i - 1)
                    .is_some_and(|&c| c == '/' || c == '.' || c == '-' || c == '_' || c == ' ')
            {
                20
            } else {
                0
            };

            // Bonus for early matches
            let position_bonus = 10 - (i.min(10) as i64);

            score += 10 + consecutive_bonus + word_boundary_bonus + position_bonus;
            last_match_idx = Some(i);
            pattern_idx += 1;
        }
    }

    if pattern_idx == pattern.len() {
        score
    } else {
        -1
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}

fn on_select_bookmark(siv: &mut Cursive, key: &String) {
    if key.is_empty() {
        return;
    }

    let state = siv.user_data::<AppState>().unwrap();
    let bookmarks = state.bookmarks.borrow();

    if let Some(bm) = bookmarks.get(key) {
        let url = bm.url.clone();
        drop(bookmarks);

        // Quit first, then open the browser
        siv.quit();

        // Schedule the browser open after quit
        siv.set_user_data(Some(url));
    }
}

pub fn run_tui_and_open() -> anyhow::Result<Option<String>> {
    let bookmarks = load_bookmarks()?;
    let bookmarks = Rc::new(RefCell::new(bookmarks));
    let filter = Rc::new(RefCell::new(String::new()));
    let tag_filter = Rc::new(RefCell::new(Option::<String>::None));
    let search_active = Rc::new(RefCell::new(false));
    let url_to_open: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    let mut siv = cursive::default();

    siv.set_theme(catppuccin_theme());

    let url_to_open_clone = Rc::clone(&url_to_open);

    siv.set_user_data(AppStateWithUrl {
        bookmarks: Rc::clone(&bookmarks),
        filter: Rc::clone(&filter),
        tag_filter: Rc::clone(&tag_filter),
        search_active: Rc::clone(&search_active),
        url_to_open: url_to_open_clone,
    });

    build_main_view_with_url(&mut siv);

    siv.add_global_callback('q', |s| {
        let state = s.user_data::<AppStateWithUrl>().unwrap();
        if !*state.search_active.borrow() {
            s.quit();
        }
    });

    siv.add_global_callback(Key::Esc, |s| {
        let state = s.user_data::<AppStateWithUrl>().unwrap();
        if *state.search_active.borrow() {
            *state.search_active.borrow_mut() = false;
            *state.filter.borrow_mut() = String::new();
            build_main_view_with_url(s);
        } else {
            s.quit();
        }
    });

    siv.run();

    Ok(url_to_open.borrow().clone())
}

struct AppStateWithUrl {
    bookmarks: Rc<RefCell<Bookmarks>>,
    filter: Rc<RefCell<String>>,
    tag_filter: Rc<RefCell<Option<String>>>,
    search_active: Rc<RefCell<bool>>,
    url_to_open: Rc<RefCell<Option<String>>>,
}

fn build_main_view_with_url(siv: &mut Cursive) {
    siv.pop_layer();

    let state = siv.user_data::<AppStateWithUrl>().unwrap();
    let bookmarks = state.bookmarks.borrow();
    let filter = state.filter.borrow().clone();
    let tag_filter = state.tag_filter.borrow().clone();
    let search_active = *state.search_active.borrow();

    let mut select = SelectView::<String>::new().on_submit(on_select_bookmark_with_url);

    let mut items: Vec<(String, String, i64)> = bookmarks
        .iter()
        .filter_map(|(key, bm)| {
            let matches_tag = tag_filter
                .as_ref()
                .is_none_or(|t| bm.tags.iter().any(|tag| tag.eq_ignore_ascii_case(t)));

            if !matches_tag {
                return None;
            }

            let score = if filter.is_empty() {
                0
            } else {
                fuzzy_score(&filter, key, &bm.url, &bm.desc, &bm.tags)
            };

            if !filter.is_empty() && score < 0 {
                return None;
            }

            let tags_str = if bm.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", bm.tags.join(", "))
            };
            let label = format!(
                "{:<12} {:<50} {}{}",
                key,
                truncate(&bm.url, 50),
                truncate(&bm.desc, 30),
                tags_str
            );
            Some((label, key.clone(), score))
        })
        .collect();

    items.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(&b.1)));

    for (label, key, _) in items {
        select.add_item(label, key);
    }

    if select.is_empty() {
        if filter.is_empty() {
            select.add_item("(no bookmarks - press 'a' to add one)", String::new());
        } else {
            select.add_item("(no matches)", String::new());
        }
    }

    let select = select.with_name(BOOKMARK_LIST_NAME);

    let tag_display = if let Some(t) = tag_filter.as_ref() {
        format!(" [tag: {}]", t)
    } else {
        String::new()
    };

    let title = format!("Bookmarks{}", tag_display);

    drop(bookmarks);

    let select = OnEventView::new(select)
        .on_event('a', |s| {
            let state = s.user_data::<AppStateWithUrl>().unwrap();
            if !*state.search_active.borrow() {
                show_add_dialog_with_url(s);
            }
        })
        .on_event('e', |s| {
            let state = s.user_data::<AppStateWithUrl>().unwrap();
            if !*state.search_active.borrow() {
                show_edit_dialog_with_url(s);
            }
        })
        .on_event('d', |s| {
            let state = s.user_data::<AppStateWithUrl>().unwrap();
            if !*state.search_active.borrow() {
                show_delete_dialog_with_url(s);
            }
        })
        .on_event('/', |s| {
            let state = s.user_data::<AppStateWithUrl>().unwrap();
            if !*state.search_active.borrow() {
                *state.search_active.borrow_mut() = true;
                build_main_view_with_url(s);
                s.focus_name(SEARCH_INPUT_NAME).ok();
            }
        })
        .on_event('t', |s| {
            let state = s.user_data::<AppStateWithUrl>().unwrap();
            if !*state.search_active.borrow() {
                show_tag_filter_dialog_with_url(s);
            }
        });

    let help_text = if search_active {
        "Type to filter | Enter: Select | Esc: Cancel search"
    } else {
        "Enter: Open | a: Add | e: Edit | d: Delete | /: Search | t: Tags | q: Quit"
    };

    let mut layout =
        LinearLayout::vertical().child(Panel::new(select.scrollable().full_screen()).title(title));

    if search_active {
        let search_input = EditView::new()
            .content(&filter)
            .on_edit(|s, text, _| {
                let state = s.user_data::<AppStateWithUrl>().unwrap();
                *state.filter.borrow_mut() = text.to_string();
                update_bookmark_list_with_url(s);
            })
            .on_submit(|s, _| {
                if let Some(key) = get_selected_key(s)
                    && !key.is_empty()
                {
                    on_select_bookmark_with_url(s, &key);
                }
            })
            .with_name(SEARCH_INPUT_NAME)
            .full_width();

        layout.add_child(
            LinearLayout::horizontal()
                .child(TextView::new("> "))
                .child(search_input),
        );
    }

    layout.add_child(TextView::new(help_text));

    siv.add_fullscreen_layer(layout);

    if search_active {
        siv.focus_name(SEARCH_INPUT_NAME).ok();
    }
}

fn update_bookmark_list_with_url(siv: &mut Cursive) {
    let (items, filter_empty) = {
        let state = siv.user_data::<AppStateWithUrl>().unwrap();
        let bookmarks = state.bookmarks.borrow();
        let filter = state.filter.borrow().clone();
        let tag_filter = state.tag_filter.borrow().clone();

        let mut items: Vec<(String, String, i64)> = bookmarks
            .iter()
            .filter_map(|(key, bm)| {
                let matches_tag = tag_filter
                    .as_ref()
                    .is_none_or(|t| bm.tags.iter().any(|tag| tag.eq_ignore_ascii_case(t)));

                if !matches_tag {
                    return None;
                }

                let score = if filter.is_empty() {
                    0
                } else {
                    fuzzy_score(&filter, key, &bm.url, &bm.desc, &bm.tags)
                };

                if !filter.is_empty() && score < 0 {
                    return None;
                }

                let tags_str = if bm.tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", bm.tags.join(", "))
                };
                let label = format!(
                    "{:<12} {:<50} {}{}",
                    key,
                    truncate(&bm.url, 50),
                    truncate(&bm.desc, 30),
                    tags_str
                );
                Some((label, key.clone(), score))
            })
            .collect();

        items.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(&b.1)));

        (items, filter.is_empty())
    };

    siv.call_on_name(BOOKMARK_LIST_NAME, |view: &mut SelectView<String>| {
        view.clear();

        for (label, key, _) in items {
            view.add_item(label, key);
        }

        if view.is_empty() {
            if filter_empty {
                view.add_item("(no bookmarks - press 'a' to add one)", String::new());
            } else {
                view.add_item("(no matches)", String::new());
            }
        }
    });
}

fn on_select_bookmark_with_url(siv: &mut Cursive, key: &String) {
    if key.is_empty() {
        return;
    }

    let state = siv.user_data::<AppStateWithUrl>().unwrap();
    let bookmarks = state.bookmarks.borrow();

    if let Some(bm) = bookmarks.get(key) {
        let url = bm.url.clone();
        *state.url_to_open.borrow_mut() = Some(url);
        drop(bookmarks);
        siv.quit();
    }
}

fn show_add_dialog(siv: &mut Cursive) {
    let dialog = Dialog::new()
        .title("Add Bookmark")
        .content(
            LinearLayout::vertical()
                .child(TextView::new("Key:"))
                .child(EditView::new().with_name("key").fixed_width(40))
                .child(TextView::new("URL:"))
                .child(EditView::new().with_name("url").fixed_width(40))
                .child(TextView::new("Description:"))
                .child(EditView::new().with_name("desc").fixed_width(40))
                .child(TextView::new("Tags (comma-separated):"))
                .child(EditView::new().with_name("tags").fixed_width(40)),
        )
        .button("Add", |s| {
            let key = s
                .call_on_name("key", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();
            let url = s
                .call_on_name("url", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();
            let desc = s
                .call_on_name("desc", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();
            let tags_str = s
                .call_on_name("tags", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();

            if key.is_empty() || url.is_empty() {
                s.add_layer(Dialog::info("Key and URL are required"));
                return;
            }

            let tags: Vec<String> = tags_str
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            let state = s.user_data::<AppState>().unwrap();
            let mut bookmarks = state.bookmarks.borrow_mut();

            match add_bookmark(&mut bookmarks, key, url, desc, tags) {
                Ok(()) => {
                    if let Err(e) = save_bookmarks(&bookmarks) {
                        drop(bookmarks);
                        s.add_layer(Dialog::info(format!("Failed to save: {}", e)));
                        return;
                    }
                    drop(bookmarks);
                    s.pop_layer();
                    build_main_view(s);
                }
                Err(e) => {
                    drop(bookmarks);
                    s.add_layer(Dialog::info(format!("Error: {}", e)));
                }
            }
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn show_add_dialog_with_url(siv: &mut Cursive) {
    let dialog = Dialog::new()
        .title("Add Bookmark")
        .content(
            LinearLayout::vertical()
                .child(TextView::new("Key:"))
                .child(EditView::new().with_name("key").fixed_width(40))
                .child(TextView::new("URL:"))
                .child(EditView::new().with_name("url").fixed_width(40))
                .child(TextView::new("Description:"))
                .child(EditView::new().with_name("desc").fixed_width(40))
                .child(TextView::new("Tags (comma-separated):"))
                .child(EditView::new().with_name("tags").fixed_width(40)),
        )
        .button("Add", |s| {
            let key = s
                .call_on_name("key", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();
            let url = s
                .call_on_name("url", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();
            let desc = s
                .call_on_name("desc", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();
            let tags_str = s
                .call_on_name("tags", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();

            if key.is_empty() || url.is_empty() {
                s.add_layer(Dialog::info("Key and URL are required"));
                return;
            }

            let tags: Vec<String> = tags_str
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            let state = s.user_data::<AppStateWithUrl>().unwrap();
            let mut bookmarks = state.bookmarks.borrow_mut();

            match add_bookmark(&mut bookmarks, key, url, desc, tags) {
                Ok(()) => {
                    if let Err(e) = save_bookmarks(&bookmarks) {
                        drop(bookmarks);
                        s.add_layer(Dialog::info(format!("Failed to save: {}", e)));
                        return;
                    }
                    drop(bookmarks);
                    s.pop_layer();
                    build_main_view_with_url(s);
                }
                Err(e) => {
                    drop(bookmarks);
                    s.add_layer(Dialog::info(format!("Error: {}", e)));
                }
            }
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn show_edit_dialog(siv: &mut Cursive) {
    let selected_key = get_selected_key(siv);
    let selected_key = match selected_key {
        Some(k) if !k.is_empty() => k,
        _ => return,
    };

    let state = siv.user_data::<AppState>().unwrap();
    let bookmarks = state.bookmarks.borrow();

    let bm = match bookmarks.get(&selected_key) {
        Some(b) => b.clone(),
        None => return,
    };

    drop(bookmarks);

    let key_for_closure = selected_key.clone();

    let dialog = Dialog::new()
        .title(format!("Edit: {}", selected_key))
        .content(
            LinearLayout::vertical()
                .child(TextView::new("URL:"))
                .child(
                    EditView::new()
                        .content(&bm.url)
                        .with_name("url")
                        .fixed_width(40),
                )
                .child(TextView::new("Description:"))
                .child(
                    EditView::new()
                        .content(&bm.desc)
                        .with_name("desc")
                        .fixed_width(40),
                )
                .child(TextView::new("Tags (comma-separated):"))
                .child(
                    EditView::new()
                        .content(bm.tags.join(", "))
                        .with_name("tags")
                        .fixed_width(40),
                ),
        )
        .button("Save", move |s| {
            let url = s
                .call_on_name("url", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();
            let desc = s
                .call_on_name("desc", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();
            let tags_str = s
                .call_on_name("tags", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();

            if url.is_empty() {
                s.add_layer(Dialog::info("URL is required"));
                return;
            }

            let tags: Vec<String> = tags_str
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            let state = s.user_data::<AppState>().unwrap();
            let mut bookmarks = state.bookmarks.borrow_mut();

            if let Err(e) = update_bookmark(&mut bookmarks, &key_for_closure, url, desc, tags) {
                drop(bookmarks);
                s.add_layer(Dialog::info(format!("Error: {}", e)));
                return;
            }

            if let Err(e) = save_bookmarks(&bookmarks) {
                drop(bookmarks);
                s.add_layer(Dialog::info(format!("Failed to save: {}", e)));
                return;
            }

            drop(bookmarks);
            s.pop_layer();
            build_main_view(s);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn show_edit_dialog_with_url(siv: &mut Cursive) {
    let selected_key = get_selected_key(siv);
    let selected_key = match selected_key {
        Some(k) if !k.is_empty() => k,
        _ => return,
    };

    let state = siv.user_data::<AppStateWithUrl>().unwrap();
    let bookmarks = state.bookmarks.borrow();

    let bm = match bookmarks.get(&selected_key) {
        Some(b) => b.clone(),
        None => return,
    };

    drop(bookmarks);

    let key_for_closure = selected_key.clone();

    let dialog = Dialog::new()
        .title(format!("Edit: {}", selected_key))
        .content(
            LinearLayout::vertical()
                .child(TextView::new("URL:"))
                .child(
                    EditView::new()
                        .content(&bm.url)
                        .with_name("url")
                        .fixed_width(40),
                )
                .child(TextView::new("Description:"))
                .child(
                    EditView::new()
                        .content(&bm.desc)
                        .with_name("desc")
                        .fixed_width(40),
                )
                .child(TextView::new("Tags (comma-separated):"))
                .child(
                    EditView::new()
                        .content(bm.tags.join(", "))
                        .with_name("tags")
                        .fixed_width(40),
                ),
        )
        .button("Save", move |s| {
            let url = s
                .call_on_name("url", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();
            let desc = s
                .call_on_name("desc", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();
            let tags_str = s
                .call_on_name("tags", |v: &mut EditView| v.get_content())
                .unwrap()
                .to_string();

            if url.is_empty() {
                s.add_layer(Dialog::info("URL is required"));
                return;
            }

            let tags: Vec<String> = tags_str
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            let state = s.user_data::<AppStateWithUrl>().unwrap();
            let mut bookmarks = state.bookmarks.borrow_mut();

            if let Err(e) = update_bookmark(&mut bookmarks, &key_for_closure, url, desc, tags) {
                drop(bookmarks);
                s.add_layer(Dialog::info(format!("Error: {}", e)));
                return;
            }

            if let Err(e) = save_bookmarks(&bookmarks) {
                drop(bookmarks);
                s.add_layer(Dialog::info(format!("Failed to save: {}", e)));
                return;
            }

            drop(bookmarks);
            s.pop_layer();
            build_main_view_with_url(s);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn show_delete_dialog(siv: &mut Cursive) {
    let selected_key = get_selected_key(siv);
    let selected_key = match selected_key {
        Some(k) if !k.is_empty() => k,
        _ => return,
    };

    let key_for_closure = selected_key.clone();

    let dialog = Dialog::new()
        .title("Delete Bookmark")
        .content(TextView::new(format!(
            "Delete bookmark '{}'?",
            selected_key
        )))
        .button("Delete", move |s| {
            let state = s.user_data::<AppState>().unwrap();
            let mut bookmarks = state.bookmarks.borrow_mut();

            if let Err(e) = delete_bookmark(&mut bookmarks, &key_for_closure) {
                drop(bookmarks);
                s.add_layer(Dialog::info(format!("Error: {}", e)));
                return;
            }

            if let Err(e) = save_bookmarks(&bookmarks) {
                drop(bookmarks);
                s.add_layer(Dialog::info(format!("Failed to save: {}", e)));
                return;
            }

            drop(bookmarks);
            s.pop_layer();
            build_main_view(s);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn show_delete_dialog_with_url(siv: &mut Cursive) {
    let selected_key = get_selected_key(siv);
    let selected_key = match selected_key {
        Some(k) if !k.is_empty() => k,
        _ => return,
    };

    let key_for_closure = selected_key.clone();

    let dialog = Dialog::new()
        .title("Delete Bookmark")
        .content(TextView::new(format!(
            "Delete bookmark '{}'?",
            selected_key
        )))
        .button("Delete", move |s| {
            let state = s.user_data::<AppStateWithUrl>().unwrap();
            let mut bookmarks = state.bookmarks.borrow_mut();

            if let Err(e) = delete_bookmark(&mut bookmarks, &key_for_closure) {
                drop(bookmarks);
                s.add_layer(Dialog::info(format!("Error: {}", e)));
                return;
            }

            if let Err(e) = save_bookmarks(&bookmarks) {
                drop(bookmarks);
                s.add_layer(Dialog::info(format!("Failed to save: {}", e)));
                return;
            }

            drop(bookmarks);
            s.pop_layer();
            build_main_view_with_url(s);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn show_tag_filter_dialog(siv: &mut Cursive) {
    let state = siv.user_data::<AppState>().unwrap();
    let bookmarks = state.bookmarks.borrow();
    let tags = get_all_tags(&bookmarks);
    drop(bookmarks);

    if tags.is_empty() {
        siv.add_layer(Dialog::info("No tags found"));
        return;
    }

    let mut select = SelectView::<Option<String>>::new().on_submit(|s, tag: &Option<String>| {
        let state = s.user_data::<AppState>().unwrap();
        *state.tag_filter.borrow_mut() = tag.clone();
        s.pop_layer();
        build_main_view(s);
    });

    select.add_item("(All bookmarks)", None);
    for tag in tags {
        select.add_item(tag.clone(), Some(tag));
    }

    let dialog = Dialog::new()
        .title("Filter by Tag")
        .content(select.scrollable().max_height(10))
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn show_tag_filter_dialog_with_url(siv: &mut Cursive) {
    let state = siv.user_data::<AppStateWithUrl>().unwrap();
    let bookmarks = state.bookmarks.borrow();
    let tags = get_all_tags(&bookmarks);
    drop(bookmarks);

    if tags.is_empty() {
        siv.add_layer(Dialog::info("No tags found"));
        return;
    }

    let mut select = SelectView::<Option<String>>::new().on_submit(|s, tag: &Option<String>| {
        let state = s.user_data::<AppStateWithUrl>().unwrap();
        *state.tag_filter.borrow_mut() = tag.clone();
        s.pop_layer();
        build_main_view_with_url(s);
    });

    select.add_item("(All bookmarks)", None);
    for tag in tags {
        select.add_item(tag.clone(), Some(tag));
    }

    let dialog = Dialog::new()
        .title("Filter by Tag")
        .content(select.scrollable().max_height(10))
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn get_selected_key(siv: &mut Cursive) -> Option<String> {
    siv.call_on_name(BOOKMARK_LIST_NAME, |view: &mut SelectView<String>| {
        view.selected_id()
            .and_then(|id| view.get_item(id).map(|(_, key)| key.clone()))
    })
    .flatten()
}
