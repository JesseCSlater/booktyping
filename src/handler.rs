use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC`
        KeyCode::Esc => {
            app.quit();
        }
        KeyCode::Char(c) => {
            if key_event.modifiers == KeyModifiers::CONTROL 
                    && c.eq_ignore_ascii_case(&'c'){
                app.quit();
            }
            else {
                app.handle_char(c);
            }
        }
        KeyCode::Right => {

        }
        KeyCode::Left => {

        }
        _ => {}
    }
    Ok(())
}
