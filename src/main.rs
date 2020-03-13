use std::{
    io::{self, stdout, Write},
    time::Duration,
};
use std::process;

use crossterm::{
    event::{self, Event as CEvent, KeyEvent, KeyCode, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use log::{debug, warn, error};
use telnet::{Telnet, TelnetOption, TelnetWriter};
use tokio::prelude::*;
use tokio::task;
use tokio::sync::mpsc::{self, Receiver, Sender, error::TryRecvError};
use futures::Future;


use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Color, Style, Modifier};
use tui::symbols::DOT;
use tui::widgets::{Widget, Block, Borders, Tabs, BorderType};

use mct::mudnet::{self, CnxOutput};
use mct::ui::events::{Events, Event};
use mct::ui::app::App;
use mct::ui::app_events;
use mct::ui;
use std::fs::read;

/*
 	0 	1 	2 	3 	4 	5 	6 	7 	8 	9 	A 	B 	C 	D 	E 	F
U+250x 	─ 	━ 	│ 	┃ 	┄ 	┅ 	┆ 	┇ 	┈ 	┉ 	┊ 	┋ 	┌ 	┍ 	┎ 	┏
U+251x 	┐ 	┑ 	┒ 	┓ 	└ 	┕ 	┖ 	┗ 	┘ 	┙ 	┚ 	┛ 	├ 	┝ 	┞ 	┟
U+252x 	┠ 	┡ 	┢ 	┣ 	┤ 	┥ 	┦ 	┧ 	┨ 	┩ 	┪ 	┫ 	┬ 	┭ 	┮ 	┯
U+253x 	┰ 	┱ 	┲ 	┳ 	┴ 	┵ 	┶ 	┷ 	┸ 	┹ 	┺ 	┻ 	┼ 	┽ 	┾ 	┿
U+254x 	╀ 	╁ 	╂ 	╃ 	╄ 	╅ 	╆ 	╇ 	╈ 	╉ 	╊ 	╋ 	╌ 	╍ 	╎ 	╏
U+255x 	═ 	║ 	╒ 	╓ 	╔ 	╕ 	╖ 	╗ 	╘ 	╙ 	╚ 	╛ 	╜ 	╝ 	╞ 	╟
U+256x 	╠ 	╡ 	╢ 	╣ 	╤ 	╥ 	╦ 	╧ 	╨ 	╩ 	╪ 	╫ 	╬ 	╭ 	╮ 	╯
U+257x 	╰ 	╱ 	╲ 	╳ 	╴ 	╵ 	╶ 	╷ 	╸ 	╹ 	╺ 	╻ 	╼ 	╽ 	╾ 	╿
*/

/*
"We\'ll create a new character named \"Lorilan.\" OK? [\u{1b}[36my\u{1b}[0mes/\u{1b}[36mn\u{1b}[0mo]\r\n"
We'll create a new character named "Lorilan." OK? [yes/no]
*/





#[tokio::main]
async fn main()  -> Result<(), failure::Error> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    enable_raw_mode()?;

   let mut stdout = stdout();
   execute!(stdout, EnterAlternateScreen)?;

   let backend = CrosstermBackend::new(stdout);
   let mut terminal = Terminal::new(backend)?;
   let mut app = App::new();

    let (mut command_sender, command_receiver): (Sender<String>, Receiver<String>) =
        mpsc::channel(100);
    let (cnx_sender, cnx_receiver): (Sender<CnxOutput>, Receiver<CnxOutput>) =
        mpsc::channel(100);

    //let host = ("edge.xen.prgmr.com",4000);
    //RcSmxqq6&
    //aardwolf.org (23.111.136.202) port 4000
    //let host = "aardwolf.org:4000";
    let host = ("localhost", 9696); //currymud
    //let host = ("localhost", 27733);
    let tcp_stream = Box::new(tokio::net::TcpStream::connect(host).await
        .unwrap_or_else(|_| -> tokio::net::TcpStream {
            error!("failed to establish connection with {:?}", host);
            process::exit(1);
        }));

    tokio::spawn(mudnet::handler(tcp_stream, command_receiver, cnx_sender));

    let mut events = Events::new(cnx_receiver);

    loop {

       ui::draw_app(&mut terminal, &app)?;

       match events.next().await {
           Some(Event::Input(CEvent::Key(KeyEvent { code: KeyCode::Esc, modifiers: _ }))) => {
               break;
           }
           Some(Event::Input(CEvent::Key(key_event))) =>
               if app_events::handle_key_event(&mut app, &mut command_sender, key_event).await {
                   break;
               },
           Some(Event::Input(_)) => {}
           Some(Event::Tick) => {}
           Some(Event::Network(msg)) => app.apply_event(msg),

           None => {}
       }
   }

   execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
   disable_raw_mode()?;
   Ok(())
}


