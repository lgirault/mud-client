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
use mct::mudnet::{self, CnxOutput};


// fn main() -> Result<(), failure::Error> {
//
//     log4rs::init_file("log4rs.yml", Default::default()).unwrap();
//     // stderrlog::new()
//     //     .module(module_path!())
//     //     .verbosity(4)
//     //     .init()?;
//
//     //let host = ("edge.xen.prgmr.com",4000);
//     //RcSmxqq6&
//     //aardwolf.org (23.111.136.202) port 4000
//     //let host = "aardwolf.org:4000";
//     let host = ("localhost", 9696); //currymud
//     //let host = ("localhost", 27733);
//     let mut telnet: Telnet = Telnet::connect(host, 256)
//         .unwrap_or_else(|_| -> Telnet {
//             error!("failed to establish connection with {:?}", host);
//             process::exit(1);
//         });
//
//
//     debug!("Connected to the server!");
//
//     let config = mudnet::MudConfig::default();
//     let mut cnx_state = mudnet::CnxState::new();
//     let mut input = String::new();
//
//     loop {
//         let (new_cnx_state, data) = mudnet::next(&mut telnet, &config, &cnx_state)?;
//         cnx_state = new_cnx_state;
//
//         for d in data.iter() {
//             match d {
//                 CnxOutput::Data(str) =>
//                     println!("{}", str),
//                 CnxOutput::Msdp(msdp) =>
//                     println!("{:?}", msdp)
//             }
//         }
//
//         println!("type command:");
//         input.clear();
//         io::stdin().read_line(&mut input)?;
//         debug!("read {:?}", input);
//         let trimmed = input.trim();
//         if trimmed == ":q" {
//             break;
//         } else if trimmed == ":n" {
//         } else if trimmed == ":ttype" {
//             mudnet::negotiate(&mut telnet, &mut cnx_state, &TelnetOption::TTYPE)?;
//         } else if trimmed == ":gmcp" {
//             mudnet::negotiate(&mut telnet, &mut cnx_state, &TelnetOption::UnknownOption(mudnet::mud::options::GMCP))?;
//         } else if trimmed == ":list" {
//             mudnet::gmcp::list_command(& mut telnet)?;
//         }
//         // else if trimmed.len() > 0 {
//         //     telnet.write(input.as_bytes())?;
//         // } else {
//         //     debug!("writing nothing !")
//         // }
//         else {
//             telnet.write(input.as_bytes())?;
//         }
//
//         debug!("loop end");
//     }
//
//
//     debug!("end of loop !");
//
// //    println!("{:?}", mct::APP_NAME.as_bytes());
//
//     // stream.shutdown();
//     Ok(())
// }

use tokio::prelude::*;
use tokio::task;
use tokio::sync::mpsc::{self, Receiver, Sender, error::TryRecvError};
use futures::Future;

// enum Handle{
//     Msg(String),
//     Output(Vec<CnxOutput>)
// }

fn telnet_handler(mut rx: Receiver<String>) -> impl Future<Output=io::Result<()>> {
    async move {
        println!("Start !");
        //let host = ("edge.xen.prgmr.com",4000);
        //RcSmxqq6&
        //aardwolf.org (23.111.136.202) port 4000
        //let host = "aardwolf.org:4000";
        let host = ("localhost", 9696); //currymud
        //let host = ("localhost", 27733);
        let config = mudnet::MudConfig::default();
        let mut cnx_state = mudnet::CnxState::new();
        let mut telnet: Telnet = Telnet::connect(host, 256).await
            .unwrap_or_else(|_| -> Telnet {
                error!("failed to establish connection with {:?}", host);
                process::exit(1);
            });
        debug!("Connected to the server!");
        loop {
            match rx.try_recv() {
                Ok(msg) => {
                    debug!("sending command {}", msg);
                    telnet.write(msg.as_bytes()).await
                }
                Err(TryRecvError::Empty) => {
                    debug!("try receive empty !");
                    Ok::<usize, io::Error>(0)
                }
                Err(TryRecvError::Closed) => break,
            }?;

            // match rx.recv().await {
            //     Some(msg) =>{
            //         debug!("sending command {}", msg);
            //         telnet.write(msg.as_bytes()).await
            //     },
            //     None => Ok::<usize, io::Error>(0),
            // }?;

            let data = mudnet::next(&mut telnet, &config, &mut cnx_state).await?;

            for d in data.iter() {
                match d {
                    CnxOutput::Data(str) =>
                        println!("{}", str),
                    CnxOutput::Msdp(msdp) =>
                        println!("{:?}", msdp)
                }
            }
            task::yield_now().await;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let (mut tx, mut rx): (Sender<String>, Receiver<String>) = mpsc::channel(100);

    tokio::spawn(telnet_handler(rx));

    let mut input = String::new();
    loop {
        println!("type command:");
        input.clear();
        io::stdin().read_line(&mut input)?;
        debug!("read {:?}", input);
        let trimmed = input.trim();
        if trimmed == ":q" {
            break;
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
        else {
            tx.send(input.clone()).await;
        }
    }
    Ok(())
}