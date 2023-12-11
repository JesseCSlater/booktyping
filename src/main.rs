use booktyping::app::{App, AppResult};
use booktyping::event::{Event, EventHandler};
use booktyping::handler::handle_key_events;
use booktyping::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::{env, io};
fn main() -> AppResult<()> {
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;

    let book_title = if let Some(arg_1) = env::args().collect::<Vec<_>>().get(1) {
        arg_1.clone()
    } else {
        println!("Please provide the name of a book in the ~/.booktyping directory");
        return Ok(());
    };

    let mut app = App::new(&book_title, terminal.size()?.width)?;

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
                app.resize(width);
                tui.draw(&mut app)?;
            }
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
