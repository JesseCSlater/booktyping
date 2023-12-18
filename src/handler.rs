use crate::app::{App, AppResult, DEFAULT_TEXT_WIDTH_PERCENT, FULL_TEXT_WIDTH_PERCENT};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    use KeyCode as C;
    use KeyModifiers as M;
    match (key_event.modifiers, key_event.code) {
        (M::CONTROL, C::Char('c')) => app.quit(),
        (M::CONTROL, C::Char('f')) => {
            app.full_text_width = !app.full_text_width;
            app.text_width_percent = if app.full_text_width {
                FULL_TEXT_WIDTH_PERCENT
            } else {
                DEFAULT_TEXT_WIDTH_PERCENT
            };
            app.generate_lines()
        }
        (_, C::Char(c)) => app.handle_char(c)?,
        (M::CONTROL, C::Up) => {
            app.following_typing = false;
            app.display_line = app.display_line.checked_sub(10).unwrap_or_default();
        }
        (M::CONTROL, C::Down) => {
            app.following_typing = false;
            app.display_line += 10;
        }
        (_, C::Up) => {
            app.following_typing = false;
            app.display_line = app.display_line.checked_sub(1).unwrap_or_default();
        }
        (_, C::Down) => {
            app.following_typing = false;
            app.display_line += 1;
        }
        (_, C::Esc) => {
            app.following_typing = true;
        }
        _ => {}
    }
    Ok(())
}
