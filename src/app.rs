use std::{error, fs, fs::File};
use regex::Regex;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    book_title: String,
    book_text: String,
    keypress_log: File,
    test_log: File,
    book_lines: Vec<String>,
    line_index: Vec<(usize, usize)>,
    
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(book_title : &str) -> AppResult<Self> {
        Ok(Self {
            running: true,
            book_title: book_title.to_string(),
            book_text: App::load_book(book_title)?,
            keypress_log: App::get_keypress_log(book_title)?,
            test_log: App::get_test_log(book_title)?,
            book_lines: Vec::new(),
            line_index: Vec::new(),
        })
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn handle_char(&mut self, c : char) {

    }

    fn load_book(book_title: &str) -> AppResult<String>{
        Ok(Regex::new(r"\s+")
            .unwrap()
            .replace_all(
                &fs::read_to_string(
                        dirs::home_dir()
                        .unwrap()
                        .join(".booktyping")
                        .join(format!("{}.txt", book_title)))?,
                " "
                )
            .to_string())
    }

    fn get_keypress_log(book_title: &str) -> AppResult<fs::File>{
        Ok(fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dirs::home_dir()
                .unwrap()
                .join(".booktyping")
                .join(book_title)
                .join("keypresses.json"))?)
    }

    fn get_test_log(book_title: &str) -> AppResult<fs::File>{

        let closure_annotated = |i: i32| -> i32 { i };

        Ok(fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(dirs::home_dir()
                .unwrap()
                .join(".booktyping")
                .join(book_title)
                .join("tests.json"))?)
    }

    pub fn resize(x: usize, y: usize) {

    }

    pub fn generate_lines(&mut self, max_line_len: usize) -> (Vec<String>, Vec<(usize, usize)>) {
        let mut lines = Vec::new();
        let mut line_index: Vec<(usize, usize)> = Vec::new();
        let mut line = "".to_owned();
        let mut word = "".to_owned();
        let mut row_i = 0;
        let mut column_i = 0;
    
        for c in self.book_text.chars() {
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
                    line_index.push((row_i, column_i));
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
            line_index.push((row_i, column_i));
            column_i += 1;
        }
    
        (lines, line_index)
    }
}
