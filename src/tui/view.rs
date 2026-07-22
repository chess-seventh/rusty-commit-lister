//! Pure `view` renderer (Elm/MVU) plus its widget-building helpers.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState};

use crate::domain::model::{AppMode, AppModel, CommitRecord};
use crate::domain::update::distinct_repos;

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

/// Formats the status bar text based on row count and cursor position.
///
/// Returns `"Row 0/0 | q quit"` when total is 0.
/// Returns `"Row {cursor}/Total | q quit"` otherwise (cursor is already 1-based).
///
/// Pure function - no I/O, no mutation.
fn format_status_text(cursor_one_based: usize, total: usize) -> String {
    if total == 0 {
        "Row 0/0 | q quit".to_string()
    } else {
        format!("Row {cursor_one_based}/{total} | q quit")
    }
}

/// Formats the search mode status bar text showing filtered vs total commit count.
///
/// Returns `"{filtered} of {total} commits | Esc cancel"`.
///
/// Pure function - no I/O, no mutation.
fn search_status_text(filtered: usize, total: usize) -> String {
    format!("{filtered} of {total} commits | Esc cancel")
}

/// Pure render function - Elm/MVU View.
///
/// Takes a reference to the current `AppModel` and a mutable Frame reference.
/// Does NOT mutate model state.
/// Renders the appropriate widget tree for the current `AppMode`.
///
/// Layout: 2 vertical chunks in Browse/Detail/RepoPicker mode (table area + status bar).
/// In Search mode: 3 vertical chunks (table area + search bar + status bar).
pub fn view(model: &AppModel, frame: &mut Frame) {
    if model.mode == AppMode::Search {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(frame.area());

        let main_area = vertical_chunks[0];
        let search_bar_area = vertical_chunks[1];
        let status_area = vertical_chunks[2];

        render_main_area(model, frame, main_area);
        render_search_bar(model, frame, search_bar_area);
        render_status_bar(model, frame, status_area);
    } else {
        let vertical_chunks =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(frame.area());

        let main_area = vertical_chunks[0];
        let status_area = vertical_chunks[1];

        render_main_area(model, frame, main_area);
        render_status_bar(model, frame, status_area);
    }
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

    render_commit_table(model, frame, area);
}

/// Renders the scrollable commit table (Date/Time/Message/Folder) with the
/// selected row highlighted. Message and folder cells are truncated to fit.
fn render_commit_table(model: &AppModel, frame: &mut Frame, area: ratatui::layout::Rect) {
    let header = Row::new(vec![
        Cell::from(Text::from("Date").style(Style::new().bold())),
        Cell::from(Text::from("Time").style(Style::new().bold())),
        Cell::from(Text::from("Message").style(Style::new().bold())),
        Cell::from(Text::from("Folder").style(Style::new().bold())),
    ]);

    let data_rows: Vec<Row> = model
        .filtered_rows
        .iter()
        .map(|record| {
            Row::new(vec![
                Cell::from(record.date.as_str()),
                Cell::from(record.time.as_str()),
                Cell::from(truncate(&record.message, 40)),
                Cell::from(truncate(&record.folder, 20)),
            ])
        })
        .collect();

    let column_widths = [
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Min(20),
        Constraint::Min(10),
    ];

    let table = Table::new(data_rows, column_widths)
        .header(header)
        .block(Block::new().borders(Borders::ALL))
        .row_highlight_style(Style::new().reversed());

    // render_commit_table is only called when filtered_rows is non-empty
    let mut table_state = TableState::default().with_selected(Some(model.cursor));

    frame.render_stateful_widget(table, area, &mut table_state);
}

/// Renders the search input line showing the current search query with a cursor indicator.
///
/// Displays `"/ <query>_"` where the trailing underscore acts as a cursor indicator.
/// Pure render - reads model, writes frame, no mutation.
fn render_search_bar(model: &AppModel, frame: &mut Frame, area: ratatui::layout::Rect) {
    let search_text = format!("/ {}_", model.search_query);
    frame.render_widget(Paragraph::new(search_text), area);
}

/// Renders the bottom status bar, whose contents depend on the current `AppMode`.
fn render_status_bar(model: &AppModel, frame: &mut Frame, area: ratatui::layout::Rect) {
    let status_text = match model.mode {
        AppMode::Search => search_status_text(model.filtered_rows.len(), model.commit_rows.len()),
        AppMode::Detail => "c copy | Esc return".to_string(),
        AppMode::RepoPicker => "j/k select | Enter confirm | Esc cancel".to_string(),
        AppMode::Browse => {
            if let Some(ref name) = model.active_repo_filter {
                format!(
                    "{name} \u{2022} {}/{} commits | f clear | q quit",
                    model.filtered_rows.len(),
                    model.commit_rows.len()
                )
            } else {
                let total = model.filtered_rows.len();
                let cursor_one_based = if total == 0 { 0 } else { model.cursor + 1 };
                format_status_text(cursor_one_based, total)
            }
        }
    };
    frame.render_widget(Paragraph::new(status_text), area);
}

#[cfg(test)]
mod tests {
    use super::{format_status_text, search_status_text, truncate};

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

    /// Scenario: status bar shows "Row 0/0 | q quit" when `filtered_rows` is empty
    ///
    /// This test validates the `format_status_text()` helper directly (pure function).
    /// Given `filtered_rows` is empty
    /// Then status text = "Row 0/0 | q quit"
    #[test]
    fn status_text_is_row_zero_of_zero_when_no_rows() {
        let text = format_status_text(0, 0);
        assert_eq!(text, "Row 0/0 | q quit");
    }

    /// Scenario: status bar shows "Row N/Total | q quit" when rows are present
    ///   Given cursor = 0 and total = 5
    ///   Then status text = "Row 1/5 | q quit" (cursor+1 for 1-based display)
    #[test]
    fn status_text_shows_one_based_row_and_total() {
        let text = format_status_text(1, 5);
        assert_eq!(text, "Row 1/5 | q quit");
    }

    /// Scenario: search status bar shows "N of M commits | Esc cancel" with partial match
    ///   Given filtered = 3, total = 10
    ///   Then `search_status_text` returns "3 of 10 commits | Esc cancel"
    #[test]
    fn search_status_text_shows_filtered_count_of_total() {
        assert_eq!(search_status_text(3, 10), "3 of 10 commits | Esc cancel");
    }

    /// Scenario: search status bar shows "0 of M commits | Esc cancel" when no match
    ///   Given filtered = 0, total = 10
    ///   Then `search_status_text` returns "0 of 10 commits | Esc cancel"
    #[test]
    fn search_status_text_shows_zero_when_no_match() {
        assert_eq!(search_status_text(0, 10), "0 of 10 commits | Esc cancel");
    }

    /// Scenario: search status bar shows "M of M commits | Esc cancel" when all match
    ///   Given filtered = 10, total = 10
    ///   Then `search_status_text` returns "10 of 10 commits | Esc cancel"
    #[test]
    fn search_status_text_shows_full_count_when_all_match() {
        assert_eq!(search_status_text(10, 10), "10 of 10 commits | Esc cancel");
    }
}
