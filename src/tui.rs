use crate::bookmarks::{
    Bookmark, Bookmarks, add_bookmark, delete_bookmark, get_all_tags, load_bookmarks,
    save_bookmarks, update_bookmark,
};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use std::io;

// Catppuccin Mocha colors
mod colors {
    use ratatui::style::Color;

    pub const BASE: Color = Color::Rgb(30, 30, 46);
    pub const SURFACE0: Color = Color::Rgb(49, 50, 68);
    pub const SURFACE1: Color = Color::Rgb(69, 71, 90);
    pub const TEXT: Color = Color::Rgb(205, 214, 244);
    pub const SUBTEXT0: Color = Color::Rgb(166, 173, 200);
    pub const LAVENDER: Color = Color::Rgb(180, 190, 254);
    pub const MAUVE: Color = Color::Rgb(203, 166, 247);
    pub const RED: Color = Color::Rgb(243, 139, 168);
    pub const GREEN: Color = Color::Rgb(166, 227, 161);
}

#[derive(PartialEq, Clone)]
enum Mode {
    Normal,
    Search,
    Add(AddField),
    Edit(AddField),
    Delete,
    TagFilter,
}

#[derive(PartialEq, Clone)]
enum AddField {
    Name,
    Url,
    Desc,
    Tags,
}

struct App {
    bookmarks: Bookmarks,
    filtered_indices: Vec<usize>,
    list_state: ListState,
    mode: Mode,
    search_query: String,
    tag_filter: Option<String>,
    tag_list_state: ListState,
    // Form fields for add/edit
    form_name: String,
    form_url: String,
    form_desc: String,
    form_tags: String,
    edit_index: Option<usize>,
    should_quit: bool,
    url_to_open: Option<String>,
}

impl App {
    fn new(bookmarks: Bookmarks) -> Self {
        let filtered_indices: Vec<usize> = (0..bookmarks.len()).collect();
        let mut list_state = ListState::default();
        if !filtered_indices.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            bookmarks,
            filtered_indices,
            list_state,
            mode: Mode::Normal,
            search_query: String::new(),
            tag_filter: None,
            tag_list_state: ListState::default(),
            form_name: String::new(),
            form_url: String::new(),
            form_desc: String::new(),
            form_tags: String::new(),
            edit_index: None,
            should_quit: false,
            url_to_open: None,
        }
    }

    fn update_filter(&mut self) {
        let query = self.search_query.to_lowercase();
        let query_chars: Vec<char> = query.chars().collect();

        self.filtered_indices = self
            .bookmarks
            .iter()
            .enumerate()
            .filter_map(|(i, bm)| {
                // Tag filter
                if let Some(ref tag) = self.tag_filter
                    && !bm.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                        return None;
                    }

                // Fuzzy search
                if query.is_empty() {
                    return Some((i, 0i64));
                }

                let score = fuzzy_score(&query_chars, bm);
                if score >= 0 { Some((i, score)) } else { None }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .fold(Vec::new(), |mut acc, (i, score)| {
                acc.push((i, score));
                acc
            })
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .fold(Vec::new(), |mut acc, x| {
                acc.push(x);
                acc.sort_by(|a, b| b.1.cmp(&a.1));
                acc
            })
            .into_iter()
            .map(|(i, _)| i)
            .collect();

        // Reset selection
        if self.filtered_indices.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    fn selected_bookmark(&self) -> Option<&Bookmark> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered_indices.get(i))
            .and_then(|&idx| self.bookmarks.get(idx))
    }

    fn selected_index(&self) -> Option<usize> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered_indices.get(i).copied())
    }

    fn next(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.filtered_indices.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_indices.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn clear_form(&mut self) {
        self.form_name.clear();
        self.form_url.clear();
        self.form_desc.clear();
        self.form_tags.clear();
        self.edit_index = None;
    }

    fn start_add(&mut self) {
        self.clear_form();
        self.mode = Mode::Add(AddField::Name);
    }

    fn start_edit(&mut self) {
        if let Some(bm) = self.selected_bookmark().cloned() {
            self.edit_index = self.selected_index();
            self.form_name = bm.name;
            self.form_url = bm.url;
            self.form_desc = bm.desc;
            self.form_tags = bm.tags.join(", ");
            self.mode = Mode::Edit(AddField::Name);
        }
    }

    fn save_bookmark(&mut self) {
        let tags: Vec<String> = self
            .form_tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let bookmark = Bookmark {
            name: self.form_name.clone(),
            url: self.form_url.clone(),
            desc: self.form_desc.clone(),
            tags,
        };

        if let Some(idx) = self.edit_index {
            update_bookmark(&mut self.bookmarks, idx, bookmark);
        } else {
            add_bookmark(&mut self.bookmarks, bookmark);
        }

        let _ = save_bookmarks(&self.bookmarks);
        self.clear_form();
        self.mode = Mode::Normal;
        self.update_filter();
    }

    fn delete_selected(&mut self) {
        if let Some(idx) = self.selected_index() {
            delete_bookmark(&mut self.bookmarks, idx);
            let _ = save_bookmarks(&self.bookmarks);
            self.update_filter();
        }
        self.mode = Mode::Normal;
    }

    fn open_selected(&mut self) {
        if let Some(bm) = self.selected_bookmark() {
            self.url_to_open = Some(bm.url.clone());
            self.should_quit = true;
        }
    }
}

fn fuzzy_score(pattern: &[char], bookmark: &Bookmark) -> i64 {
    let name_score = fuzzy_match(pattern, &bookmark.name.to_lowercase());
    let url_score = fuzzy_match(pattern, &bookmark.url.to_lowercase());
    let desc_score = fuzzy_match(pattern, &bookmark.desc.to_lowercase());
    let tag_score = bookmark
        .tags
        .iter()
        .map(|t| fuzzy_match(pattern, &t.to_lowercase()))
        .max()
        .unwrap_or(-1);

    if name_score >= 0 {
        name_score + 1000
    } else if url_score >= 0 {
        url_score + 500
    } else if desc_score >= 0 {
        desc_score + 100
    } else if tag_score >= 0 {
        tag_score
    } else {
        -1
    }
}

fn fuzzy_match(pattern: &[char], text: &str) -> i64 {
    if pattern.is_empty() {
        return 0;
    }

    let text_chars: Vec<char> = text.chars().collect();
    let mut pattern_idx = 0;
    let mut score: i64 = 0;
    let mut last_match: Option<usize> = None;
    let mut consecutive = 0i64;

    for (i, &c) in text_chars.iter().enumerate() {
        if pattern_idx < pattern.len() && c == pattern[pattern_idx] {
            if let Some(last) = last_match {
                if i == last + 1 {
                    consecutive += 10;
                } else {
                    consecutive = 0;
                }
            }

            let boundary_bonus = if i == 0
                || text_chars
                    .get(i.wrapping_sub(1))
                    .is_some_and(|&c| matches!(c, '/' | '.' | '-' | '_' | ' '))
            {
                20
            } else {
                0
            };

            let position_bonus = 10 - (i.min(10) as i64);
            score += 10 + consecutive + boundary_bonus + position_bonus;
            last_match = Some(i);
            pattern_idx += 1;
        }
    }

    if pattern_idx == pattern.len() {
        score
    } else {
        -1
    }
}

pub fn run_tui_and_open() -> Result<Option<String>> {
    let bookmarks = load_bookmarks()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(bookmarks);
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result?;
    Ok(app.url_to_open)
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match &app.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    KeyCode::Enter => app.open_selected(),
                    KeyCode::Char('/') => {
                        app.mode = Mode::Search;
                        app.search_query.clear();
                    }
                    KeyCode::Char('a') => app.start_add(),
                    KeyCode::Char('e') => app.start_edit(),
                    KeyCode::Char('d') => {
                        if app.selected_bookmark().is_some() {
                            app.mode = Mode::Delete;
                        }
                    }
                    KeyCode::Char('t') => {
                        let tags = get_all_tags(&app.bookmarks);
                        if !tags.is_empty() {
                            app.tag_list_state.select(Some(0));
                            app.mode = Mode::TagFilter;
                        }
                    }
                    KeyCode::Char('c') => {
                        app.tag_filter = None;
                        app.update_filter();
                    }
                    _ => {}
                },
                Mode::Search => match key.code {
                    KeyCode::Esc => {
                        app.mode = Mode::Normal;
                        app.search_query.clear();
                        app.update_filter();
                    }
                    KeyCode::Enter => {
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        app.update_filter();
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        app.update_filter();
                    }
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    _ => {}
                },
                Mode::Add(field) | Mode::Edit(field) => {
                    let is_edit = matches!(app.mode, Mode::Edit(_));
                    match key.code {
                        KeyCode::Esc => {
                            app.clear_form();
                            app.mode = Mode::Normal;
                        }
                        KeyCode::Tab | KeyCode::Enter => {
                            let next = match field {
                                AddField::Name => AddField::Url,
                                AddField::Url => AddField::Desc,
                                AddField::Desc => AddField::Tags,
                                AddField::Tags => {
                                    if !app.form_name.is_empty() && !app.form_url.is_empty() {
                                        app.save_bookmark();
                                        continue;
                                    }
                                    AddField::Tags
                                }
                            };
                            app.mode = if is_edit {
                                Mode::Edit(next)
                            } else {
                                Mode::Add(next)
                            };
                        }
                        KeyCode::BackTab => {
                            let prev = match field {
                                AddField::Name => AddField::Name,
                                AddField::Url => AddField::Name,
                                AddField::Desc => AddField::Url,
                                AddField::Tags => AddField::Desc,
                            };
                            app.mode = if is_edit {
                                Mode::Edit(prev)
                            } else {
                                Mode::Add(prev)
                            };
                        }
                        KeyCode::Backspace => {
                            let field_ref = match field {
                                AddField::Name => &mut app.form_name,
                                AddField::Url => &mut app.form_url,
                                AddField::Desc => &mut app.form_desc,
                                AddField::Tags => &mut app.form_tags,
                            };
                            field_ref.pop();
                        }
                        KeyCode::Char(c) => {
                            let field_ref = match field {
                                AddField::Name => &mut app.form_name,
                                AddField::Url => &mut app.form_url,
                                AddField::Desc => &mut app.form_desc,
                                AddField::Tags => &mut app.form_tags,
                            };
                            field_ref.push(c);
                        }
                        _ => {}
                    }
                }
                Mode::Delete => match key.code {
                    KeyCode::Char('y') | KeyCode::Enter => app.delete_selected(),
                    KeyCode::Char('n') | KeyCode::Esc => app.mode = Mode::Normal,
                    _ => {}
                },
                Mode::TagFilter => match key.code {
                    KeyCode::Esc => app.mode = Mode::Normal,
                    KeyCode::Enter => {
                        let tags = get_all_tags(&app.bookmarks);
                        if let Some(i) = app.tag_list_state.selected() {
                            if i == 0 {
                                app.tag_filter = None;
                            } else {
                                app.tag_filter = tags.get(i - 1).cloned();
                            }
                        }
                        app.update_filter();
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let tags = get_all_tags(&app.bookmarks);
                        let len = tags.len() + 1;
                        let i = app.tag_list_state.selected().unwrap_or(0);
                        app.tag_list_state.select(Some((i + 1) % len));
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let tags = get_all_tags(&app.bookmarks);
                        let len = tags.len() + 1;
                        let i = app.tag_list_state.selected().unwrap_or(0);
                        app.tag_list_state
                            .select(Some(if i == 0 { len - 1 } else { i - 1 }));
                    }
                    _ => {}
                },
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    // Background
    f.render_widget(
        Block::default().style(Style::default().bg(colors::BASE)),
        size,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(size);

    // Title with tag filter indicator
    let title = if let Some(ref tag) = app.tag_filter {
        format!(" Bookmarks [tag: {}] ", tag)
    } else {
        " Bookmarks ".to_string()
    };

    // Bookmark list
    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .filter_map(|&i| app.bookmarks.get(i))
        .map(|bm| {
            let tags = if bm.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", bm.tags.join(", "))
            };

            let desc = if bm.desc.is_empty() {
                String::new()
            } else {
                format!(" - {}", bm.desc)
            };

            let line = Line::from(vec![
                Span::styled(&bm.name, Style::default().fg(colors::LAVENDER).bold()),
                Span::styled(desc, Style::default().fg(colors::SUBTEXT0)),
                Span::styled(tags, Style::default().fg(colors::MAUVE)),
            ]);

            let url_line = Line::from(Span::styled(
                format!("  {}", bm.url),
                Style::default().fg(colors::SUBTEXT0).dim(),
            ));

            ListItem::new(vec![line, url_line])
        })
        .collect();

    let items = if items.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "No bookmarks. Press 'a' to add one.",
            Style::default().fg(colors::SUBTEXT0).italic(),
        )))]
    } else {
        items
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::SURFACE1))
                .title(Span::styled(
                    title,
                    Style::default().fg(colors::MAUVE).bold(),
                ))
                .style(Style::default().bg(colors::BASE)),
        )
        .highlight_style(
            Style::default()
                .bg(colors::SURFACE0)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, chunks[0], &mut app.list_state.clone());

    // Search bar / status
    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::SURFACE1))
        .style(Style::default().bg(colors::BASE));

    let search_content = match &app.mode {
        Mode::Search => {
            let cursor = "█";
            Paragraph::new(Line::from(vec![
                Span::styled(" / ", Style::default().fg(colors::MAUVE)),
                Span::styled(&app.search_query, Style::default().fg(colors::TEXT)),
                Span::styled(cursor, Style::default().fg(colors::LAVENDER)),
            ]))
        }
        _ if !app.search_query.is_empty() => Paragraph::new(Line::from(vec![
            Span::styled(" Filter: ", Style::default().fg(colors::SUBTEXT0)),
            Span::styled(&app.search_query, Style::default().fg(colors::TEXT)),
        ])),
        _ => Paragraph::new(Line::from(Span::styled(
            " Type / to search",
            Style::default().fg(colors::SUBTEXT0),
        ))),
    };

    f.render_widget(search_content.block(search_block), chunks[1]);

    // Help bar
    let help = match &app.mode {
        Mode::Normal => {
            "↑↓/jk: Navigate │ Enter: Open │ /: Search │ a: Add │ e: Edit │ d: Delete │ t: Tags │ c: Clear filter │ q: Quit"
        }
        Mode::Search => "Type to filter │ ↑↓: Navigate │ Enter: Confirm │ Esc: Cancel",
        Mode::Add(_) | Mode::Edit(_) => {
            "Tab: Next field │ Shift+Tab: Previous │ Enter on Tags: Save │ Esc: Cancel"
        }
        Mode::Delete => "y/Enter: Confirm │ n/Esc: Cancel",
        Mode::TagFilter => "↑↓/jk: Navigate │ Enter: Select │ Esc: Cancel",
    };

    let help_paragraph = Paragraph::new(Span::styled(help, Style::default().fg(colors::SUBTEXT0)))
        .style(Style::default().bg(colors::BASE));

    f.render_widget(help_paragraph, chunks[2]);

    // Render modals
    match &app.mode {
        Mode::Add(field) => render_form_modal(f, "Add Bookmark", field, app),
        Mode::Edit(field) => render_form_modal(f, "Edit Bookmark", field, app),
        Mode::Delete => render_delete_modal(f, app),
        Mode::TagFilter => render_tag_modal(f, app),
        _ => {}
    }
}

fn render_form_modal(f: &mut Frame, title: &str, current_field: &AddField, app: &App) {
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(colors::MAUVE).bold(),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::LAVENDER))
        .style(Style::default().bg(colors::BASE));

    f.render_widget(block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    let fields = [
        ("Name", &app.form_name, AddField::Name),
        ("URL", &app.form_url, AddField::Url),
        ("Description", &app.form_desc, AddField::Desc),
        ("Tags (comma-separated)", &app.form_tags, AddField::Tags),
    ];

    for (i, (label, value, field)) in fields.iter().enumerate() {
        let is_active = current_field == field;
        let style = if is_active {
            Style::default().fg(colors::LAVENDER)
        } else {
            Style::default().fg(colors::SURFACE1)
        };

        let cursor = if is_active { "█" } else { "" };
        let content = format!("{}{}", value, cursor);

        let input = Paragraph::new(content)
            .style(Style::default().fg(colors::TEXT))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(style)
                    .title(Span::styled(
                        format!(" {} ", label),
                        if is_active {
                            Style::default().fg(colors::LAVENDER)
                        } else {
                            Style::default().fg(colors::SUBTEXT0)
                        },
                    ))
                    .style(Style::default().bg(colors::BASE)),
            );

        f.render_widget(input, inner[i]);
    }
}

fn render_delete_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let name = app
        .selected_bookmark()
        .map(|b| b.name.as_str())
        .unwrap_or("this bookmark");

    let block = Block::default()
        .title(Span::styled(
            " Delete ",
            Style::default().fg(colors::RED).bold(),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::RED))
        .style(Style::default().bg(colors::BASE));

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("Delete '{}'?", name),
            Style::default().fg(colors::TEXT),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("y", Style::default().fg(colors::GREEN).bold()),
            Span::styled(": Yes  ", Style::default().fg(colors::SUBTEXT0)),
            Span::styled("n", Style::default().fg(colors::RED).bold()),
            Span::styled(": No", Style::default().fg(colors::SUBTEXT0)),
        ]),
    ])
    .block(block)
    .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(text, area);
}

fn render_tag_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(40, 50, f.area());
    f.render_widget(Clear, area);

    let tags = get_all_tags(&app.bookmarks);
    let mut items: Vec<ListItem> = vec![ListItem::new(Span::styled(
        "(All bookmarks)",
        Style::default().fg(colors::SUBTEXT0),
    ))];

    items.extend(
        tags.iter()
            .map(|t| ListItem::new(Span::styled(t, Style::default().fg(colors::TEXT)))),
    );

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    " Filter by Tag ",
                    Style::default().fg(colors::MAUVE).bold(),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::LAVENDER))
                .style(Style::default().bg(colors::BASE)),
        )
        .highlight_style(
            Style::default()
                .bg(colors::SURFACE0)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut app.tag_list_state.clone());
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
