use booktyping::app::{App, AppResult};
use booktyping::event::{Event, EventHandler};
use booktyping::handler::handle_key_events;
use booktyping::tui::Tui;
use clap::Parser;
use pdf_extract::extract_text;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;

/// Please provide the name of a book in the ~/.booktyping directory or -c to move it there
#[derive(Parser, Debug)]
struct Cli {
    title: String,
    /// Whether to try to convert this
    #[clap(short, long)]
    convert: bool,
}

fn main() -> AppResult<()> {
    let Cli { mut title, convert } = Cli::parse();

    if convert {
        println!("Converting {title}");
        let p = PathBuf::from(&title);
        let ext = p.extension().unwrap_or_default().to_str().unwrap();
        title = p.file_stem().unwrap().to_str().unwrap().to_owned();
        let mut name = dirs::home_dir().unwrap();
        name.push(".booktyping");
        name.push(&title);
        name.set_extension("txt");

        let text = if ext == "pdf" {
            println!("pdf format detected. Extracting...");
            extract_text(p).unwrap()
        } else {
            std::fs::read_to_string(p).unwrap() // For utf8 validation
        };
        println!("Convertion successful.\nSaving to {name:?}\nOpening file now:");

        std::fs::write(name, text).unwrap();
    }

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;

    let mut app = App::new(&title, terminal.size()?.width)?;

    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);

    tui.init()?;
    tui.draw(&mut app)?; //Draw first frame

    // Start the main loop.
    while app.running {
        // Handle events.
        match tui.events.next()? {
            Event::Key(key_event) => {
                handle_key_events(key_event, &mut app)?;
                tui.draw(&mut app)?;
            }
            Event::Resize(width, _) => {
                app.terminal_width = width;
                app.generate_lines();
                tui.draw(&mut app)?;
            }
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
