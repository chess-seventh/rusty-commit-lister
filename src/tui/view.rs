//! Pure `view` renderer (Elm/MVU) plus its widget-building helpers.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState};

use crate::domain::model::{AppMode, AppModel, CommitRecord};
use crate::domain::update::distinct_repos;

/// Accent color for the commit-table header row.
const HEADER_COLOR: Color = Color::Cyan;
/// Color used to highlight the matched filter substring (fzf-style).
const MATCH_COLOR: Color = Color::Red;

/// Returns the (even-row, odd-row) background colors for zebra striping, both
/// derived from a single user-chosen base color name (config `zebra_color`).
///
/// Uses 256-palette indexed colors (not truecolor `Rgb`) so the stripes render
/// on any 256-color terminal — including tmux without truecolor passthrough.
/// The even shade is darker than the odd shade of the same hue; unknown names
/// (and "default") fall back to a subtle dark grey.
///
/// Pure function - no I/O, no mutation.
fn zebra_colors(name: &str) -> (Color, Color) {
    let (even, odd) = match name.trim().to_lowercase().as_str() {
        "green" => (22, 28),
        "blue" => (18, 25),
        "red" => (52, 88),
        "cyan" => (23, 30),
        "magenta" | "purple" => (53, 90),
        "yellow" => (58, 100),
        "orange" => (94, 130),
        "gray" | "grey" => (236, 240),
        _ => (234, 237),
    };
    (Color::Indexed(even), Color::Indexed(odd))
}

/// Splits `message` into spans, highlighting the first case-insensitive match of
/// `query` in [`MATCH_COLOR`]. Returns a single plain span when the query is
/// empty or unmatched.
///
/// Highlighting is applied only when both strings are ASCII, which keeps byte
/// offsets valid and avoids slicing on a non-char-boundary; non-ASCII messages
/// render un-highlighted (never panics).
///
/// Pure function - no I/O, no mutation.
fn message_spans(message: &str, query: &str) -> Vec<Span<'static>> {
    let q = query.trim();
    if q.is_empty() || !message.is_ascii() || !q.is_ascii() {
        return vec![Span::raw(message.to_string())];
    }
    match message.to_ascii_lowercase().find(&q.to_ascii_lowercase()) {
        Some(start) => {
            let end = start + q.len();
            vec![
                Span::raw(message[..start].to_string()),
                Span::styled(
                    message[start..end].to_string(),
                    Style::new().fg(MATCH_COLOR),
                ),
                Span::raw(message[end..].to_string()),
            ]
        }
        None => vec![Span::raw(message.to_string())],
    }
}

/// Returns the final path segment of `folder` (the folder name), ignoring any
/// trailing slash. Falls back to the whole string when there is no separator.
///
/// Pure function - no I/O, no mutation.
fn folder_name(folder: &str) -> &str {
    let trimmed = folder.trim_end_matches('/');
    trimmed.rsplit('/').next().unwrap_or(trimmed)
}

/// Truncates a string to at most `max_chars` Unicode scalar values.
///
/// If the string's char count exceeds `max_chars`, returns the first
/// `(max_chars - 3)` chars followed by `"..."`. Otherwise returns the
/// string unchanged. Uses char-boundary-safe slicing via `char_indices`.
///
/// Pure function - no I/O, no mutation.
fn truncate(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    let keep = max_chars.saturating_sub(3);
    let end_byte = s
        .char_indices()
        .nth(keep)
        .map_or(0, |(byte_pos, _)| byte_pos);
    format!("{}...", &s[..end_byte])
}

/// Formats the Browse status bar: the live filter query (when any), the active
/// repo filter (when any), the visible/total counts, and the key hints.
///
/// - no filter: `"3/3 | type: filter · ^Y/^E/^P/^U copy · ^F repo ^R reload · Esc quit"`
/// - filtering: `"/feat · 1/3 | ^Y/^E/^P/^U copy · Esc quit"`
///
/// Pure function - no I/O, no mutation.
fn browse_status_text(
    query: &str,
    repo_filter: Option<&str>,
    filtered: usize,
    total: usize,
) -> String {
    let copy_hints = "^Y/^E/^P/^U copy";
    if !query.is_empty() {
        return format!("/{query} \u{2022} {filtered}/{total} | {copy_hints} \u{2022} Esc clear");
    }
    if let Some(name) = repo_filter {
        return format!(
            "repo:{name} \u{2022} {filtered}/{total} | ^F clear \u{2022} {copy_hints} \u{2022} Esc quit"
        );
    }
    format!(
        "{filtered}/{total} | type: filter \u{2022} {copy_hints} \u{2022} ^F repo ^R reload \u{2022} Esc quit"
    )
}

/// Pure render function - Elm/MVU View.
///
/// Takes a reference to the current `AppModel` and a mutable Frame reference.
/// Does NOT mutate model state.
/// Renders the appropriate widget tree for the current `AppMode`.
///
/// Layout: an explicit search box on top (Browse only), then the main area, then
/// the status bar. The search box shows the live filter query as you type; there
/// is no separate search mode.
pub fn view(model: &AppModel, frame: &mut Frame) {
    if model.mode == AppMode::Browse {
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());
        render_search_box(model, frame, chunks[0]);
        render_main_area(model, frame, chunks[1]);
        render_status_bar(model, frame, chunks[2]);
    } else {
        let chunks =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(frame.area());
        render_main_area(model, frame, chunks[0]);
        render_status_bar(model, frame, chunks[1]);
    }
}

/// Renders the explicit search box (fzf-style prompt) showing the live filter
/// query with a trailing cursor. Shows a placeholder hint when the query is empty.
fn render_search_box(model: &AppModel, frame: &mut Frame, area: Rect) {
    let content = if model.search_query.is_empty() {
        Line::from(Span::styled(
            "type to filter…",
            Style::new().fg(Color::DarkGray),
        ))
    } else {
        Line::from(format!("{}\u{2588}", model.search_query))
    };
    let paragraph = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(HEADER_COLOR))
            .title("Search"),
    );
    frame.render_widget(paragraph, area);
}

/// Builds the display lines for the Detail overlay from a `CommitRecord`.
///
/// Returns a Vec of exactly 5 formatted strings:
///   0. `Date:    {date}`
///   1. `Time:    {time}`
///   2. `Message: {message}`   (full, NOT truncated)
///   3. `Folder:  {folder}`    (full, NOT truncated)
///   4. `URL:     {url}` or `URL:     - not available -` when url is None
///
/// Pure function - no I/O, no mutation. pub so integration test files can call it.
pub fn detail_lines(record: &CommitRecord) -> Vec<String> {
    vec![
        format!("Date:    {}", record.date),
        format!("Time:    {}", record.time),
        format!("Message: {}", record.message),
        format!("Folder:  {}", record.folder),
        format!(
            "URL:     {}",
            record.url.as_deref().unwrap_or("- not available -")
        ),
    ]
}

/// Renders the Detail overlay for the selected commit, plus any status message.
fn render_detail_overlay(model: &AppModel, frame: &mut Frame, area: ratatui::layout::Rect) {
    // Safety: Detail mode is only entered from Browse when filtered_rows is non-empty
    // (handle_browse_key guards this). The cursor is always a valid index.
    let record = &model.filtered_rows[model.cursor];
    let mut lines: Vec<ratatui::text::Line> = detail_lines(record)
        .into_iter()
        .map(ratatui::text::Line::from)
        .collect();
    if let Some(status) = &model.status_message {
        lines.push(ratatui::text::Line::from(""));
        lines.push(ratatui::text::Line::from(status.clone()));
    }
    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Commit Detail"),
    );
    frame.render_widget(paragraph, area);
}

/// Renders the repository picker overlay listing all distinct repos with their commit counts.
///
/// Each entry is formatted as "{`repo_name`} ({`count`})".
/// The row at `model.picker_cursor` is highlighted with a reversed style.
/// Repos are listed in the order returned by `distinct_repos`: count descending, name ascending.
///
/// Pure render - reads model, writes frame, no mutation.
fn render_repo_picker(model: &AppModel, frame: &mut Frame, area: ratatui::layout::Rect) {
    let repos = distinct_repos(&model.commit_rows);
    let items: Vec<ListItem> = repos
        .iter()
        .enumerate()
        .map(|(i, (name, count))| {
            let text = format!("{name} ({count})");
            let style = if i == model.picker_cursor {
                Style::new().reversed()
            } else {
                Style::default()
            };
            ListItem::new(text).style(style)
        })
        .collect();
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Repo Filter"));
    frame.render_widget(list, area);
}

/// Renders the primary content area, dispatching on loading/error/empty state
/// and the current `AppMode` (table, detail overlay, or repo picker).
fn render_main_area(model: &AppModel, frame: &mut Frame, area: ratatui::layout::Rect) {
    if model.loading {
        frame.render_widget(Paragraph::new("Loading..."), area);
        return;
    }

    if let Some(error) = &model.error_message {
        frame.render_widget(Paragraph::new(error.as_str()), area);
        return;
    }

    if model.filtered_rows.is_empty() {
        frame.render_widget(Paragraph::new("No commits found in scan window"), area);
        return;
    }

    if model.mode == AppMode::Detail {
        render_detail_overlay(model, frame, area);
        return;
    }

    if model.mode == AppMode::RepoPicker {
        render_repo_picker(model, frame, area);
        return;
    }

    // Browse with rows: commit table on top, always-on detail bubble below so the
    // full info for the selected line is visible without opening a separate view.
    let bubble_h = detail_bubble_height(model.status_message.is_some());
    let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(bubble_h)]).split(area);
    render_commit_table(model, frame, chunks[0]);
    render_detail_bubble(model, frame, chunks[1]);
}

/// Height (including borders) of the always-on detail bubble.
///
/// 5 detail lines + top/bottom border = 7; two extra lines when a status message
/// (e.g. a copy confirmation) is present.
///
/// Pure function - no I/O, no mutation.
fn detail_bubble_height(has_status: bool) -> u16 {
    if has_status { 9 } else { 7 }
}

/// Renders the always-on "bubble" showing the full detail of the selected commit
/// (never truncated), plus any transient status message, in a bordered box.
fn render_detail_bubble(model: &AppModel, frame: &mut Frame, area: Rect) {
    // Clamp defensively: filtered_rows is non-empty here, but the cursor may lag
    // a shrink after filtering until the next update.
    let idx = model.cursor.min(model.filtered_rows.len() - 1);
    let record = &model.filtered_rows[idx];
    let mut lines: Vec<ratatui::text::Line> = detail_lines(record)
        .into_iter()
        .map(ratatui::text::Line::from)
        .collect();
    if let Some(status) = &model.status_message {
        lines.push(ratatui::text::Line::from(""));
        lines.push(ratatui::text::Line::from(status.clone()));
    }
    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(HEADER_COLOR))
            .title("Commit Detail"),
    );
    frame.render_widget(paragraph, area);
}

/// Computes the Message and Folder column widths for a table of inner width
/// `area_width`, given fixed Date/Time widths and inter-column spacing.
///
/// Message takes ~70% of the flexible space, Folder ~30% (clamped 8..=30), so
/// both columns grow with the terminal instead of being capped at a constant.
///
/// Pure function - no I/O, no mutation.
fn table_column_widths(area_width: u16) -> (u16, u16, u16, u16) {
    let (date_w, time_w) = (10u16, 5u16);
    let inner = area_width.saturating_sub(2); // block borders
    let gaps = 3u16; // column_spacing between 4 columns
    let flex = inner.saturating_sub(date_w + time_w + gaps);
    let folder_w = (flex * 3 / 10).clamp(8, 30);
    let msg_w = flex.saturating_sub(folder_w);
    (date_w, time_w, msg_w, folder_w)
}

/// Renders the scrollable commit table (Date/Time/Message/Folder) with a colored
/// header, zebra-striped rows, and the selected row highlighted. Message and
/// folder cells are truncated to the responsive column widths; the Folder cell
/// shows only the final path segment (see [`folder_name`]).
fn render_commit_table(model: &AppModel, frame: &mut Frame, area: Rect) {
    let header = Row::new(vec!["Date", "Time", "Message", "Folder"])
        .style(Style::new().fg(HEADER_COLOR).bold());

    let (date_w, time_w, msg_w, folder_w) = table_column_widths(area.width);
    let (even_bg, odd_bg) = zebra_colors(&model.config.zebra_color);

    let data_rows: Vec<Row> = model
        .filtered_rows
        .iter()
        .enumerate()
        .map(|(i, record)| {
            let bg = if i % 2 == 1 { odd_bg } else { even_bg };
            Row::new(vec![
                Cell::from(record.date.as_str()),
                Cell::from(record.time.as_str()),
                Cell::from(Line::from(message_spans(
                    &record.message,
                    &model.search_query,
                ))),
                Cell::from(truncate(folder_name(&record.folder), folder_w as usize)),
            ])
            .style(Style::new().bg(bg))
        })
        .collect();

    let column_widths = [
        Constraint::Length(date_w),
        Constraint::Length(time_w),
        Constraint::Length(msg_w),
        Constraint::Length(folder_w),
    ];

    let table = Table::new(data_rows, column_widths)
        .header(header)
        .block(Block::new().borders(Borders::ALL))
        .row_highlight_style(Style::new().reversed());

    // render_commit_table is only called when filtered_rows is non-empty
    let mut table_state = TableState::default().with_selected(Some(model.cursor));

    frame.render_stateful_widget(table, area, &mut table_state);
}

/// Renders the bottom status bar, whose contents depend on the current `AppMode`.
fn render_status_bar(model: &AppModel, frame: &mut Frame, area: Rect) {
    let status_text = match model.mode {
        AppMode::Detail => "^U copy URL | Esc return".to_string(),
        AppMode::RepoPicker => "j/k select | Enter confirm | Esc cancel".to_string(),
        AppMode::Browse => browse_status_text(
            &model.search_query,
            model.active_repo_filter.as_deref(),
            model.filtered_rows.len(),
            model.commit_rows.len(),
        ),
    };
    frame.render_widget(Paragraph::new(status_text), area);
}

#[cfg(test)]
mod tests {
    use super::{
        browse_status_text, folder_name, message_spans, table_column_widths, truncate, zebra_colors,
    };
    use ratatui::style::Color;

    /// Scenario: a base color derives two distinct indexed shades
    ///   Given zebra_color = "green"
    ///   Then both rows are 256-palette indexed colors and the shades differ.
    #[test]
    fn zebra_colors_derives_two_green_shades() {
        let (even, odd) = zebra_colors("green");
        match (even, odd) {
            (Color::Indexed(e), Color::Indexed(o)) => {
                assert_ne!(e, o, "the two shades must differ");
            }
            _ => panic!("zebra_colors must return indexed (256-palette) colors"),
        }
    }

    /// Scenario: an unknown color name falls back to the default without panic
    #[test]
    fn zebra_colors_unknown_name_falls_back() {
        let (even, odd) = zebra_colors("not-a-real-color");
        assert_ne!(even, odd, "the two derived shades must differ");
    }

    /// Scenario: folder_name returns the final path segment
    ///   Given folder = "/projects/rcl/src"
    ///   Then folder_name returns "src"
    #[test]
    fn folder_name_returns_last_segment() {
        assert_eq!(folder_name("/projects/rcl/src"), "src");
    }

    /// Scenario: folder_name ignores a trailing slash
    ///   Given folder = "/projects/rcl/src/"
    ///   Then folder_name returns "src" (not "")
    #[test]
    fn folder_name_ignores_trailing_slash() {
        assert_eq!(folder_name("/projects/rcl/src/"), "src");
    }

    /// Scenario: folder_name returns the whole string when there is no separator
    ///   Given folder = "dotfiles"
    ///   Then folder_name returns "dotfiles"
    #[test]
    fn folder_name_without_separator_returns_whole() {
        assert_eq!(folder_name("dotfiles"), "dotfiles");
    }

    /// Scenario: Message and Folder columns share the flexible width
    ///   Given a 120-col area (118 inner)
    ///   Then Date=10, Time=5, Folder is clamped to 8..=30, and all four column
    ///   widths plus the 3 inter-column gaps fit within the inner width.
    #[test]
    fn table_column_widths_share_flexible_space() {
        let (date_w, time_w, msg_w, folder_w) = table_column_widths(120);
        assert_eq!(date_w, 10);
        assert_eq!(time_w, 5);
        assert!((8..=30).contains(&folder_w), "folder width clamped 8..=30");
        assert_eq!(date_w + time_w + msg_w + folder_w + 3, 118);
    }

    /// Scenario: widths never underflow on a tiny terminal
    ///   Given a 4-col area (narrower than the fixed columns)
    ///   Then the computation saturates instead of panicking.
    #[test]
    fn table_column_widths_saturate_when_area_tiny() {
        let (_d, _t, msg_w, folder_w) = table_column_widths(4);
        assert_eq!(msg_w, 0, "message collapses to 0 on a tiny area");
        assert_eq!(folder_w, 8, "folder stays at its lower clamp");
    }

    /// Scenario: the detail bubble is 7 rows tall without a status message
    #[test]
    fn detail_bubble_height_is_seven_without_status() {
        assert_eq!(super::detail_bubble_height(false), 7);
    }

    /// Scenario: the detail bubble grows to 9 rows to fit a status message
    #[test]
    fn detail_bubble_height_is_nine_with_status() {
        assert_eq!(super::detail_bubble_height(true), 9);
    }

    /// Scenario: string shorter than `max_chars` is returned unchanged
    ///   Given s = "hello" and `max_chars` = 10
    ///   Then truncate returns "hello" (no ellipsis)
    #[test]
    fn truncate_returns_short_string_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    /// Scenario: string exactly equal to `max_chars` is returned unchanged
    ///   Given s = "1234567890" (10 chars) and `max_chars` = 10
    ///   Then truncate returns the string unchanged
    #[test]
    fn truncate_returns_string_equal_to_max_unchanged() {
        assert_eq!(truncate("1234567890", 10), "1234567890");
    }

    /// Scenario: string longer than `max_chars` is truncated with '...' suffix
    ///   Given s = "hello world extra text" and `max_chars` = 10
    ///   Then truncate returns a string of exactly 10 chars ending with "..."
    #[test]
    fn truncate_adds_ellipsis_when_string_exceeds_max_chars() {
        let result = truncate("hello world extra text", 10);
        assert_eq!(
            result, "hello w...",
            "truncated string must be 10 chars with '...' suffix"
        );
        assert_eq!(
            result.chars().count(),
            10,
            "result length must equal max_chars"
        );
    }

    /// Scenario: empty string is returned as empty string
    ///   Given s = "" and `max_chars` = 10
    ///   Then truncate returns ""
    #[test]
    fn truncate_empty_string_returns_empty() {
        assert_eq!(truncate("", 10), "");
    }

    /// Scenario: `max_chars` = 3 (minimum meaningful truncation)
    ///   Given s = "abcdef" and `max_chars` = 3
    ///   Then truncate returns "..." (no content prefix, all 3 chars are ellipsis)
    #[test]
    fn truncate_with_max_equal_to_ellipsis_length_returns_only_ellipsis() {
        let result = truncate("abcdef", 3);
        assert_eq!(result, "...");
        assert_eq!(result.chars().count(), 3);
    }

    /// Scenario: Browse status with no filter shows counts and the copy/nav hints
    ///   Given no query, no repo filter, 3 of 3 rows
    ///   Then it shows the total counts, the copy hints, and "Esc quit"
    #[test]
    fn browse_status_without_filter_shows_counts_and_hints() {
        let text = browse_status_text("", None, 3, 3);
        assert!(
            text.starts_with("3/3 "),
            "must lead with counts; got: {text}"
        );
        assert!(text.contains("^Y/^E/^P/^U copy"), "must list copy hints");
        assert!(text.contains("Esc quit"), "must show quit hint");
    }

    /// Scenario: Browse status while filtering shows the query and the match count
    ///   Given query "feat", 1 of 3 rows
    ///   Then it shows "/feat" and "1/3"
    #[test]
    fn browse_status_while_filtering_shows_query_and_matches() {
        let text = browse_status_text("feat", None, 1, 3);
        assert!(text.contains("/feat"), "must echo the query; got: {text}");
        assert!(
            text.contains("1/3"),
            "must show filtered/total; got: {text}"
        );
    }

    /// Scenario: Browse status with an active repo filter names it and offers clear
    #[test]
    fn browse_status_with_repo_filter_names_it() {
        let text = browse_status_text("", Some("dotfiles"), 2, 5);
        assert!(
            text.contains("repo:dotfiles"),
            "must name the repo; got: {text}"
        );
        assert!(text.contains("^F clear"), "must offer clear; got: {text}");
    }

    /// Scenario: an empty query yields a single plain span (no highlight)
    #[test]
    fn message_spans_without_query_is_single_plain_span() {
        let spans = message_spans("feat: add thing", "");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "feat: add thing");
    }

    /// Scenario: a matching query splits into before/match/after with the match styled
    ///   Given message "feat: add thing" and query "add"
    ///   Then the middle span is exactly "add" and carries the match color
    #[test]
    fn message_spans_highlights_case_insensitive_match() {
        let spans = message_spans("feat: ADD thing", "add");
        assert_eq!(spans.len(), 3, "before/match/after");
        assert_eq!(spans[0].content, "feat: ");
        assert_eq!(spans[1].content, "ADD", "match preserves original case");
        assert_eq!(spans[1].style.fg, Some(super::MATCH_COLOR));
        assert_eq!(spans[2].content, " thing");
    }

    /// Scenario: a non-matching query yields a single plain span
    #[test]
    fn message_spans_without_match_is_single_plain_span() {
        let spans = message_spans("feat: add thing", "zzz");
        assert_eq!(spans.len(), 1);
    }

    /// Scenario: a non-ASCII message is never highlighted (no panic on boundaries)
    #[test]
    fn message_spans_non_ascii_is_not_highlighted() {
        let spans = message_spans("✨ féat: add", "add");
        assert_eq!(spans.len(), 1, "non-ASCII messages render un-highlighted");
    }
}
