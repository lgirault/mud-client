//use std::{
//    io::{self, stdout, Write},
//    sync::mpsc,
//    thread,
//    time::Duration,
//};
//
//use crossterm::{
//    event::{self, Event as CEvent, KeyEvent, KeyCode, EventStream},
//    execute,
//    Command,
//    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
//};
//
//
//use tui::Terminal;
//use tui::backend::CrosstermBackend;
//use tui::layout::{Layout, Constraint, Direction};
//use tui::style::{Color, Style, Modifier};
//use tui::symbols::DOT;
//use tui::widgets::{Widget, Block, Borders, Tabs, BorderType};
//
//use mct::events::{Events, Event};
//use mct::app::App;
//use mct::app_events;
//use mct::ui;
//
///*
//  	0 	1 	2 	3 	4 	5 	6 	7 	8 	9 	A 	B 	C 	D 	E 	F
//U+250x 	─ 	━ 	│ 	┃ 	┄ 	┅ 	┆ 	┇ 	┈ 	┉ 	┊ 	┋ 	┌ 	┍ 	┎ 	┏
//U+251x 	┐ 	┑ 	┒ 	┓ 	└ 	┕ 	┖ 	┗ 	┘ 	┙ 	┚ 	┛ 	├ 	┝ 	┞ 	┟
//U+252x 	┠ 	┡ 	┢ 	┣ 	┤ 	┥ 	┦ 	┧ 	┨ 	┩ 	┪ 	┫ 	┬ 	┭ 	┮ 	┯
//U+253x 	┰ 	┱ 	┲ 	┳ 	┴ 	┵ 	┶ 	┷ 	┸ 	┹ 	┺ 	┻ 	┼ 	┽ 	┾ 	┿
//U+254x 	╀ 	╁ 	╂ 	╃ 	╄ 	╅ 	╆ 	╇ 	╈ 	╉ 	╊ 	╋ 	╌ 	╍ 	╎ 	╏
//U+255x 	═ 	║ 	╒ 	╓ 	╔ 	╕ 	╖ 	╗ 	╘ 	╙ 	╚ 	╛ 	╜ 	╝ 	╞ 	╟
//U+256x 	╠ 	╡ 	╢ 	╣ 	╤ 	╥ 	╦ 	╧ 	╨ 	╩ 	╪ 	╫ 	╬ 	╭ 	╮ 	╯
//U+257x 	╰ 	╱ 	╲ 	╳ 	╴ 	╵ 	╶ 	╷ 	╸ 	╹ 	╺ 	╻ 	╼ 	╽ 	╾ 	╿
//*/
//
//fn main() -> Result<(), failure::Error> {
//    enable_raw_mode()?;
//
//    let mut stdout = stdout();
//    execute!(stdout, EnterAlternateScreen)?;
//
//    let backend = CrosstermBackend::new(stdout);
//    let mut terminal = Terminal::new(backend)?;
//
////    let mut reader = EventStream::new();
////    let mut event = reader.next().fuse();
//
//    let events = Events::new();
//    let mut app = App::new();
//    loop {
//        ui::draw_app(&mut terminal, &app)?;
//
//        match events.next()? {
//            Event::Input(CEvent::Key(KeyEvent { code: KeyCode::Esc, modifiers: _ })) => {
//                break;
//            }
//            Event::Input(CEvent::Key(key_event)) =>
//                app_events::handle_key_event(&mut app, key_event),
//            Event::Input(_) => {}
//            Event::Tick => {}
//        }
//    }
//
//    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
//    disable_raw_mode()?;
//    Ok(())
//}

use telnet::{Telnet, TelnetOption};
//use std::net::{TcpStream, Shutdown};
use std::io;
use std::process;
use log::{debug, warn, error};
use mct::tn;


fn main() -> Result<(), failure::Error> {
    stderrlog::new()
        .module(module_path!())
        .verbosity(4)
        .init()?;

    //aardwolf.org (23.111.136.202) port 4000
    //let host = "aardwolf.org:4000";
    let host = ("localhost", 9696); //currymud
    //let host = ("localhost", 27733);
    let mut telnet: Telnet = Telnet::connect(host, 256)
        .unwrap_or_else(|_| -> Telnet {
            error!("failed to establish connection with {:?}", host);
            process::exit(1);
        });


    debug!("Connected to the server!");

    let config = tn::MudConfig::default();
    let mut cnx_state = tn::CnxState::new();
    let mut input = String::new();

    loop {
        let (new_cnx_state, data) = tn::next(&mut telnet, &config, &cnx_state)?;
        cnx_state = new_cnx_state;

        println!("{}", data);

        input.clear();
        io::stdin().read_line(&mut input)?;
        debug!("read {:?}", input);
        let trimmed = input.trim();
        if trimmed == ":q" {
            break;
        } else if trimmed == ":ttype" {
            tn::negotiate(&mut telnet, &mut cnx_state, &TelnetOption::TTYPE)?;
        } else if trimmed == ":gmcp" {
            tn::negotiate(&mut telnet, &mut cnx_state, &TelnetOption::UnknownOption(tn::mud::options::GMCP))?;
        } else if trimmed == ":list" {
            tn::gmcp::list_command(& mut telnet)?;
        }
        else if trimmed.len() > 0 {
            telnet.write(input.as_bytes())?;
        } else {
            debug!("writing nothing !")
        }

        debug!("loop end");
    }


    debug!("end of loop !");

//    println!("{:?}", mct::APP_NAME.as_bytes());

    // stream.shutdown();
    Ok(())
}