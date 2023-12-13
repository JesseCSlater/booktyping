use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::{prelude::*, widgets::*};

use crate::app::App;
//TODO fix panic on end of short input
/// Renders the user interface widgets. 
pub fn render(app: &mut App, frame: &mut Frame) {
    let &(start_line, start_offset) = app.line_index.get(app.sample_start_index).unwrap();
    let &(cur_line, cur_offset) = app
        .line_index
        .get(app.sample_start_index + app.cur_char)
        .unwrap();
    let &(end_line, end_offset) = app
        .line_index
        .get(app.sample_start_index + app.sample_len)
        .unwrap();
    let mut lines: Vec<String> = app.book_lines.clone();
    let num_rows = frame.size().height as usize - 2;
    let rows_to_center = num_rows / 2 - 2;

    if app.following_typing {
        app.display_line = cur_line
    }

    app.display_line = usize::min(app.display_line, lines.len());

    let first_row = usize::checked_sub(rows_to_center, app.display_line).unwrap_or(0);

    let num_skipped_lines = usize::checked_sub(app.display_line, rows_to_center).unwrap_or(0);
    lines = lines.split_off(usize::min(num_skipped_lines, lines.len()));
    lines.truncate(num_rows - first_row);

    let mut display_lines: Vec<Line> = Vec::new();
    for (mut i, s) in lines.iter().enumerate() {
        i += num_skipped_lines;
        if i == cur_line {
            if i == start_line && i == end_line {
                display_lines.push(Line::from(vec![
                    s.chars().take(start_offset).collect::<String>().dim(),
                    s.chars()
                        .take(cur_offset)
                        .skip(start_offset)
                        .collect::<String>()
                        .white(),
                    s.chars()
                        .nth(cur_offset)
                        .unwrap()
                        .to_string()
                        .black()
                        .bg(Color::White),
                    s.chars()
                        .take(end_offset)
                        .skip(cur_offset + 1)
                        .collect::<String>()
                        .blue(),
                    s.chars().skip(end_offset).collect::<String>().dim(),
                ]));
            } else if i == start_line {
                display_lines.push(Line::from(vec![
                    s.chars().take(start_offset).collect::<String>().dim(),
                    s.chars()
                        .take(cur_offset)
                        .skip(start_offset)
                        .collect::<String>()
                        .white(),
                    s.chars()
                        .nth(cur_offset)
                        .unwrap()
                        .to_string()
                        .black()
                        .bg(Color::White),
                    s.chars().skip(cur_offset + 1).collect::<String>().blue(),
                ]));
            } else if i == end_line {
                display_lines.push(Line::from(vec![
                    s.chars().take(cur_offset).collect::<String>().white(),
                    s.chars()
                        .nth(cur_offset)
                        .unwrap()
                        .to_string()
                        .black()
                        .bg(Color::White),
                    s.chars()
                        .take(end_offset)
                        .skip(cur_offset + 1)
                        .collect::<String>()
                        .blue(),
                    s.chars().skip(end_offset).collect::<String>().dim(),
                ]));
            } else {
                display_lines.push(Line::from(vec![
                    s.chars().take(cur_offset).collect::<String>().white(),
                    s.chars()
                        .nth(cur_offset)
                        .unwrap()
                        .to_string()
                        .black()
                        .bg(Color::White),
                    s.chars().skip(cur_offset + 1).collect::<String>().blue(),
                ]));
            }
        } else if i < cur_line {
            if i == start_line {
                display_lines.push(Line::from(vec![
                    s.chars().take(start_offset).collect::<String>().dim(),
                    s.chars().skip(start_offset).collect::<String>().white(),
                ]));
            } else if i < start_line {
                display_lines.push(s.clone().dim().into());
            } else {
                display_lines.push(s.clone().white().into());
            }
        } else {
            if i == end_line {
                display_lines.push(Line::from(vec![
                    s.chars().take(end_offset).collect::<String>().blue(),
                    s.chars().skip(end_offset).collect::<String>().dim(),
                ]));
            } else if i < end_line {
                display_lines.push(s.clone().blue().into());
            } else {
                display_lines.push(s.clone().dim().into());
            }
        }
    }

    let graph = Paragraph::new::<Text>(display_lines.into()).style(Style::default());

    let screen = Rect::new(0, 0, frame.size().width, frame.size().height);

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(first_row as u16 + 1),
            Constraint::Percentage(100),
        ])
        .split(screen);
    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - app.text_width_percent) / 2),
            Constraint::Percentage(app.text_width_percent),
            Constraint::Percentage((100 - app.text_width_percent) / 2),
        ])
        .split(vert[1])[1];

    // Render into the second chunk of the layout.
    frame.render_widget(graph, horiz);
    frame.render_widget(
        Block::default()
            .title("BookTyping")
            .title(
                block::Title::from(format!("{}", app.get_rolling_average().unwrap()))
                    .alignment(Alignment::Right),
            )
            .borders(Borders::ALL)
            .border_style(Style::new().white()),
        screen,
    );
}
