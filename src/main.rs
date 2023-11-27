use std::fs::OpenOptions;
use std::io;
use std::panic;
use std::fs;
use std::io::Write;
use std::env;
use chrono::{DateTime, Utc};
use chrono::serde::ts_nanoseconds;
use regex::Regex;
use serde::{Serialize, Deserialize};
use ratatui::{backend::CrosstermBackend as Backend, prelude::*, widgets::*};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use crossterm::event::{self, Event as CrosstermEvent};
use std::time::{Duration, Instant};

const TEXT_WIDTH_PERCENT : u16 = 60;
const STARTING_SAMPLE_SIZE : usize = 100;

fn main() -> Result<(), io::Error> {
    let backend = Backend::new(io::stderr());
    let mut terminal = Terminal::new(backend)?;

    terminal::enable_raw_mode()?;
    crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

    let panic_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic| {
        terminal::disable_raw_mode().unwrap();
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
        panic_hook(panic);
    }));

    terminal.hide_cursor()?;
    terminal.clear()?;

    let args: Vec<String> = env::args().collect();
    let book_title = args.get(1).unwrap();

    let book = 
        Regex::new(r"\s+")
            .unwrap()
            .replace_all(
                &fs::read_to_string(
                        dirs::home_dir()
                        .unwrap()
                        .join(".booktyping")
                        .join(format!("{}.txt", book_title)))?,
                " "
                )
            .to_string();

    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dirs::home_dir()
            .unwrap()
            .join(".booktyping")
            .join(book_title)
            .join("keypresses.json"))
        .unwrap();
    
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(dirs::home_dir()
            .unwrap()
            .join(".booktyping")
            .join(book_title)
            .join("tests.json"))
        .unwrap();
    
    let (mut start_index, mut len) = get_next_sample(book_title)?;
    if start_index >= book.len() - 1 {
        terminal.show_cursor()?;
        terminal::disable_raw_mode().unwrap();
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
        terminal.clear()?;
        terminal.set_cursor(0, 0)?;
        println!("Book complete");
        terminal.set_cursor(0, 1)?;
        return Ok(());
    } 
    len = len.min(book.len() - start_index - 1);
    let mut start_time = Utc::now();
    let mut cur_char = 0;
    let mut following_typing = true;
    let mut display_line: usize = 0;
    let mut last_tick = Instant::now();

    let mut max_line_len = 
        (terminal.size()?.width as f64 
        * (TEXT_WIDTH_PERCENT as f64 / 100.0)) 
        as usize;
    let mut num_rows = terminal.size()?.height as usize - 2;

    let mut rows_to_center = num_rows / 2 - 2;

    let (mut all_lines, mut row_column) = split_lines(&book, max_line_len);
    draw(&row_column, start_index, cur_char, len, following_typing, display_line, all_lines.clone(), rows_to_center, num_rows, &mut terminal, &book_title);

    loop {
        let tick_rate = Duration::from_millis(5);
        let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

        if event::poll(timeout).expect("no events available") {
            match event::read().expect("unable to read event") {
                CrosstermEvent::Key(e) => {
                    match e.code {
                        KeyCode::Esc => {
                            terminal.show_cursor()?;
                            terminal::disable_raw_mode().unwrap();
                            crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
                            terminal.clear()?;
                            terminal.set_cursor(0, 0)?;
                            return Ok(());
                        }
                        KeyCode::Up => {
                            following_typing = false;
                            display_line = display_line.checked_sub(1).unwrap_or_default();
                        }
                        KeyCode::Down => {
                            following_typing = false;
                            display_line += 1;
                        }
                        KeyCode::Left => {
                            following_typing = false;
                            display_line = display_line.checked_sub(num_rows).unwrap_or_default();
                        }
                        KeyCode::Right => {
                            following_typing = false;
                            display_line += num_rows;
                        }
                        KeyCode::Char(c) => {
                            if !following_typing {
                                following_typing = true;
                            }
                            let correct = c == book.chars().nth(start_index + cur_char).unwrap();
        
                            if correct {
                                cur_char += 1
                            }
                            if !correct || cur_char == len {
                                log_test(&book_title, start_time, start_index, cur_char, correct);
                                start_time = Utc::now();
                                (start_index, len) = get_next_sample(book_title)?;
                                if start_index >= book.len() - 1 {
                                    terminal.show_cursor()?;
                                    terminal::disable_raw_mode().unwrap();
                                    crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
                                    terminal.clear()?;
                                    terminal.set_cursor(0, 0)?;
                                    println!("Book complete");
                                    terminal.set_cursor(0, 1)?;
                                    return Ok(());
                                } 
                                len = len.min(book.len() - start_index - 1);
                                cur_char = 0;
                            }
        
                            let log_entry = serde_json::to_vec(
                                &KeyPress {
                                    correct,
                                    key: c,
                                    time: Utc::now()
                                }).unwrap();
                            log.write_all(&log_entry)?;
                        }
                        _ => ()
                    }
                },
                CrosstermEvent::Resize(_, _) => {
                    max_line_len = 
                        (terminal.size()?.width as f64 
                        * (TEXT_WIDTH_PERCENT as f64 / 100.0)) 
                        as usize;
                    num_rows = terminal.size()?.height as usize - 2;
            
                    rows_to_center = num_rows / 2 - 2;
            
                    (all_lines, row_column) = split_lines(&book, max_line_len);
                },
                _ => (),
            }
            draw(&row_column, start_index, cur_char, len, following_typing, display_line, all_lines.clone(), rows_to_center, num_rows, &mut terminal, &book_title);
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn draw(row_column: &Vec<(usize, usize)>, start_index: usize, cur_char: usize, len: usize, following_typing: bool, mut display_line: usize, mut all_lines: Vec<String>, rows_to_center: usize, num_rows: usize, terminal: &mut Terminal<Backend<io::Stderr>>, book_title : &str) {
        let &(start_line, start_offset) = row_column.get(start_index).unwrap();
        let &(cur_line, cur_offset) = row_column.get(start_index + cur_char).unwrap();
        let &(end_line, end_offset) = row_column.get(start_index + len).unwrap();
        if following_typing {display_line = cur_line}
        
        display_line = usize::min(display_line, all_lines.len());

        let first_row = usize::checked_sub(rows_to_center,display_line)
            .unwrap_or(0);

        let num_skipped_lines = usize::checked_sub(display_line, rows_to_center)
            .unwrap_or(0);

        all_lines = all_lines.split_off(usize::min(num_skipped_lines, all_lines.len()));
        all_lines.truncate(num_rows - first_row);

        let mut lines: Vec<Line> = Vec::new();
        for (mut i, s) in all_lines.iter().enumerate() {
            i += num_skipped_lines;
            if i == cur_line {
                if i == start_line && i == end_line {
                    lines.push(Line::from(
                        vec![
                            s.chars().take(start_offset).collect::<String>().dim(),
                            s.chars().take(cur_offset).skip(start_offset).collect::<String>().white(),
                            s.chars().nth(cur_offset).unwrap().to_string().black().bg(Color::White),
                            s.chars().take(end_offset).skip(cur_offset+1).collect::<String>().blue(),
                            s.chars().skip(end_offset).collect::<String>().dim(),
                        ]));
                }
                else if i == start_line {
                    lines.push(Line::from(
                        vec![
                            s.chars().take(start_offset).collect::<String>().dim(),
                            s.chars().take(cur_offset).skip(start_offset).collect::<String>().white(),
                            s.chars().nth(cur_offset).unwrap().to_string().black().bg(Color::White),
                            s.chars().skip(cur_offset+1).collect::<String>().blue(),
                        ]));
                }
                else if i == end_line {
                    lines.push(Line::from(
                        vec![
                            s.chars().take(cur_offset).collect::<String>().white(),
                            s.chars().nth(cur_offset).unwrap().to_string().black().bg(Color::White),
                            s.chars().take(end_offset).skip(cur_offset+1).collect::<String>().blue(),
                            s.chars().skip(end_offset).collect::<String>().dim(),
                        ]));
                }
                else {
                    lines.push(Line::from(
                        vec![
                            s.chars().take(cur_offset).collect::<String>().white(),
                            s.chars().nth(cur_offset).unwrap().to_string().black().bg(Color::White),
                            s.chars().skip(cur_offset+1).collect::<String>().blue(),
                        ]));
                }
            }
            else if i < cur_line {
                if i == start_line {
                    lines.push(Line::from(
                        vec![
                            s.chars().take(start_offset).collect::<String>().dim(),
                            s.chars().skip(start_offset).collect::<String>().white(),
                        ]));
                }
                else if i < start_line  {
                    lines.push(s.clone().dim().into());
                }
                else {
                    lines.push(s.clone().white().into());
                }
            }
            else {
                if i == end_line {
                    lines.push(Line::from(
                        vec![
                            s.chars().take(end_offset).collect::<String>().blue(),
                            s.chars().skip(end_offset).collect::<String>().dim(),
                        ]));
                }
                else if i < end_line  {
                    lines.push(s.clone().blue().into());
                }
                else {
                    lines.push(s.clone().dim().into());
                }
            }
        }

        let graph = Paragraph::new::<Text>(lines.into()).style(Style::default());
        
        terminal.draw(|frame| {
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
                    Constraint::Percentage((100 - TEXT_WIDTH_PERCENT) / 2),
                    Constraint::Percentage(TEXT_WIDTH_PERCENT),
                    Constraint::Percentage((100 - TEXT_WIDTH_PERCENT) / 2),
                ])
                .split(vert[1])[1];

            // Render into the second chunk of the layout.
            frame.render_widget(graph, horiz);
            frame.render_widget(Block::default()
                .title("BookTyping")
                .title(
                    block::Title::from(format!("{}", get_rolling_average(book_title)))
                    .alignment(Alignment::Right)
                ).borders(Borders::ALL).border_style(Style::new().white()), screen);
            }).unwrap();
}

fn split_lines(s : &str, max_line_len : usize) -> (Vec<String>, Vec<(usize, usize)>) {
    let mut lines = Vec::new();
    let mut row_column: Vec<(usize, usize)> = Vec::new();
    let mut line = "".to_owned();
    let mut word = "".to_owned();
    let mut row_i = 0;
    let mut column_i = 0;

    for c in s.chars() {
        word.push(c);
        if c == ' ' {
            if line.len() + word.len() < max_line_len {
                line.push_str(&word);
            }
            else {
                lines.push(line);
                line = word.to_owned();
                row_i += 1;
                column_i = 0;
            }
            for _ in 0..word.len() {
                row_column.push((row_i, column_i));
                column_i += 1;
            }
            word = "".to_owned();
        }
    }
    if line.len() + word.len() < max_line_len {
        line.push_str(&word);
        lines.push(line);
    }
    else {
        lines.push(line);
        lines.push(word.clone());
        row_i += 1;
    }
    for _ in 0..word.len() {
        row_column.push((row_i, column_i));
        column_i += 1;
    }

    (lines, row_column)
}

fn get_rolling_average(book_title: &str) -> usize {
    let tests: Vec<Test> = serde_json::from_str(
        &fs::read_to_string(
            dirs::home_dir()
                    .unwrap()
                    .join(".booktyping")
                    .join(book_title)
                    .join("tests.json")
                ).unwrap()
        ).unwrap_or(Vec::new());
    
    tests.iter()
        .map(|t| t.end_index - t.start_index)
        .filter(|&len| {len > 5})
        .rev()
        .take(10)
        .sum::<usize>()
        / 10
}

fn get_next_sample(book_title : &str) -> Result<(usize, usize), io::Error> {
    let book = 
        Regex::new(r"\s+")
            .unwrap()
            .replace_all(
                &fs::read_to_string(dirs::home_dir()
                    .unwrap()
                    .join(".booktyping")
                    .join(format!("{}.txt", book_title)))?,
                " "
                )
            .to_string();

    let tests: Vec<Test> = serde_json::from_str(
        &fs::read_to_string(
            dirs::home_dir()
                    .unwrap()
                    .join(".booktyping")
                    .join(book_title)
                    .join("tests.json")
                ).unwrap()
        ).unwrap_or(Vec::new());

    let mut start_index = 0;
    for t in &tests {
        if t.succeeded && t.end_index > start_index {
            start_index = t.end_index;
        }
    }

    let avg_50 = tests.iter()
        .map(|t| t.end_index - t.start_index)
        .filter(|&len| {len > 5})
        .rev()
        .take(50)
        .sum::<usize>()
        / 50;
    let max_10 = tests.iter()
        .map(|t| t.end_index - t.start_index)
        .filter(|&len| {len > 5})
        .rev()
        .take(10)
        .max()
        .unwrap_or(STARTING_SAMPLE_SIZE);
    let best = usize::max(avg_50, max_10) + 5;

    let (wrong_total, wrong_num) = tests.iter()
    .rev()
    .take_while(|t| !t.succeeded)
    .map(|t| t.end_index - t.start_index)
    .filter(|&len| {len > 5})
    .fold((0,0), 
        |(total, sum), len| 
            (total + len, sum + 1)
    );
    let wrong_avg = wrong_total.checked_div(wrong_num).unwrap_or(0); 
    let x = wrong_num * wrong_num;
    let sample_len = (best * 2 + wrong_avg * x) / (2 + x);

    let len = book.chars()
        .skip(start_index)
        .take(sample_len)
        .collect::<String>()
        .rfind(' ')
        .unwrap_or(sample_len - 1) + 1;

    Ok((
        start_index,
        len.min(book.len() - 1),
    ))
}

#[derive(Serialize, Deserialize)]
struct KeyPress {
    correct : bool,
    key : char,
    #[serde(with = "ts_nanoseconds")]
    time : DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct Test {
    succeeded : bool,
    start_index : usize,
    end_index : usize,
    #[serde(with = "ts_nanoseconds")]
    started : DateTime<Utc>,
    #[serde(with = "ts_nanoseconds")]
    completed : DateTime<Utc>,
}

fn log_test(book_title: &str, start_time: DateTime<Utc>, start_index: usize, len: usize, succeeded : bool) {
    let mut tests: Vec<Test> = serde_json::from_str(
        &fs::read_to_string(
            dirs::home_dir()
                    .unwrap()
                    .join(".booktyping")
                    .join(book_title)
                    .join("tests.json")
                ).unwrap()
        ).unwrap_or(Vec::new());
    tests.push(
        Test {
            succeeded,
            start_index,
            end_index: start_index + len,
            started: start_time,
            completed: Utc::now(),
        }
    );
    fs::write(
        dirs::home_dir()
                    .unwrap()
                    .join(".booktyping")
                    .join(book_title)
                    .join("tests.json"),
        serde_json::to_vec(&tests).unwrap()
    ).unwrap();
}