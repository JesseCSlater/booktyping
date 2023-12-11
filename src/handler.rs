use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char(c) => {
            if key_event.modifiers == KeyModifiers::CONTROL && c.eq_ignore_ascii_case(&'c') {
                app.quit();
            } else {
                app.handle_char(c)?;
            }
        }
        KeyCode::Up => {
            app.following_typing = false;
            app.display_line = app.display_line.checked_sub(1).unwrap_or_default();
        }
        KeyCode::Down => {
            app.following_typing = false;
            app.display_line += 1;
        }
        KeyCode::Left => {
            app.following_typing = false;
            app.display_line = app.display_line.checked_sub(10).unwrap_or_default();
        }
        KeyCode::Right => {
            app.following_typing = false;
            app.display_line += 10;
        }
        KeyCode::Esc => {
            app.following_typing = true;
        }
        _ => {}
    }
    Ok(())
}
