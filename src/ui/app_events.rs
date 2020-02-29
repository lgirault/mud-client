
use super::{App, AppArea};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_event(app: &mut App, event: KeyEvent) {
    let KeyEvent { code, modifiers } = event;
    if app.focused_area == AppArea::Input {
        match code {
            KeyCode::Enter => app.messages.push(app.input.drain(..).collect()),
            KeyCode::Char(c) => app.input.push(c),
            KeyCode::Backspace => {
                app.input.pop();
            }
            _ => (),
        }
    }
}
