use super::{App, AppArea};
use crate::ui::app::Message;
use crossterm::event::{KeyCode, KeyEvent};
use log::debug;
use tokio::sync::mpsc::Sender;

pub type ShouldQuit = bool;

pub const SHOULD_QUIT: bool = true;

pub async fn handle_string(
    app: &mut App,
    command_sender: &mut Sender<String>,
    input: String,
) -> ShouldQuit {
    debug!("read {:?}", input);

    let trimmed = input.trim();

    if trimmed == ":q" {
        true
    } else {
        command_sender.send(input.clone()).await;
        app.messages.push(Message::UserInput(input));
        false
    }
    // else if trimmed == ":n" {} else if trimmed == ":ttype" {
    //     mudnet::negotiate(&mut telnet, &mut cnx_state, &TelnetOption::TTYPE)?;
    // } else if trimmed == ":gmcp" {
    //     mudnet::negotiate(&mut telnet, &mut cnx_state, &TelnetOption::UnknownOption(mudnet::mud::options::GMCP))?;
    // } else if trimmed == ":list" {
    //     mudnet::gmcp::list_command(&mut telnet)?;
    // }
    // else if trimmed.len() > 0 {
    //     telnet.write(input.as_bytes())?;
    // } else {
    //     debug!("writing nothing !")
    // }
}

pub async fn handle_key_event(
    app: &mut App,
    command_sender: &mut Sender<String>,
    event: KeyEvent,
) -> ShouldQuit {
    let KeyEvent { code, modifiers: _ } = event;
    app.focused_area == AppArea::Input && {
        match code {
            KeyCode::Enter => {
                //let input = app.input.drain(..).collect();
                app.input.push_str("\r\n");
                let input = app.input.clone();
                app.input.clear();
                handle_string(app, command_sender, input).await
            }
            KeyCode::Char(c) => {
                app.input.push(c);
                false
            }
            KeyCode::Backspace => {
                app.input.pop();
                false
            }
            _ => false,
        }
    }
}
