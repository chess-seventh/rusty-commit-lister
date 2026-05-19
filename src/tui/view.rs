use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};

use crate::domain::model::AppModel;

/// Pure render function — Elm/MVU View.
///
/// Takes a reference to the current AppModel and a mutable Frame reference.
/// Does NOT mutate model state.
/// Renders the appropriate widget tree for the current AppMode.
pub fn view(model: &AppModel, frame: &mut Frame) {
    let vertical_chunks =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(frame.size());

    let main_area = vertical_chunks[0];
    let status_area = vertical_chunks[1];

    render_main_area(model, frame, main_area);
    render_status_bar(model, frame, status_area);
}

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

    render_commit_table(model, frame, area);
}

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
                Cell::from(record.message.clone()),
                Cell::from(record.folder.clone()),
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
        .highlight_style(Style::new().reversed());

    // render_commit_table is only called when filtered_rows is non-empty
    let mut table_state = TableState::default().with_selected(Some(model.cursor));

    frame.render_stateful_widget(table, area, &mut table_state);
}

fn render_status_bar(model: &AppModel, frame: &mut Frame, area: ratatui::layout::Rect) {
    let commit_count = model.filtered_rows.len();
    let status_text = format!("{commit_count} commits | q quit");
    frame.render_widget(Paragraph::new(status_text), area);
}
