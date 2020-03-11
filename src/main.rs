use std::{
   io::{self, stdout, Write},
   time::Duration,
};
use std::process;

use crossterm::{
   event::{self, Event as CEvent, KeyEvent, KeyCode, EventStream},
   execute,
   Command,
   terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use log::{debug, warn, error};
use telnet::{Telnet, TelnetOption};
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



#[tokio::main]
async fn main()  -> Result<(), failure::Error> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let mut app = App::new();

    let (mut command_sender, mut command_receiver): (Sender<String>, Receiver<String>) =
        mpsc::channel(100);
    let (mut cnx_sender, mut cnx_receiver): (Sender<CnxOutput>, Receiver<CnxOutput>) =
        mpsc::channel(100);

    tokio::spawn(telnet_handler(command_receiver, cnx_sender));

    tokio::spawn(
        async move {
            loop {
                match cnx_receiver.try_recv() {
                    Ok(CnxOutput::Data(msg)) => {
                        debug!("receive cnx: {:?}", msg);
                        print!("{}", msg);
                    }
                    Ok(CnxOutput::Msdp(data)) => {
                        println!("receive msdp: {:?}", data);
                    }
                    Err(TryRecvError::Empty) => {
                        //debug!("try receive cnx empty !");
                        ()
                    }
                    Err(TryRecvError::Closed) => break,
                };
                task::yield_now().await;
            }
        }
    );


    let mut input = String::new();
    loop {
        println!("type command:");
        input.clear();
        io::stdin().read_line(&mut input)?;
        if app_events::handle_string(&mut app, &mut command_sender, input.clone()).await {
            break;
        }
    }

    Ok(())
}

// #[tokio::main]
// async fn main()  -> Result<(), failure::Error> {
//     log4rs::init_file("log4rs.yml", Default::default()).unwrap();
//
//     enable_raw_mode()?;
//
//    let mut stdout = stdout();
//    execute!(stdout, EnterAlternateScreen)?;
//
//    let backend = CrosstermBackend::new(stdout);
//    let mut terminal = Terminal::new(backend)?;
//    let mut app = App::new();
//
//     let (mut command_sender, mut command_receiver): (Sender<String>, Receiver<String>) =
//         mpsc::channel(100);
//     let (mut cnx_sender, mut cnx_receiver): (Sender<CnxOutput>, Receiver<CnxOutput>) =
//         mpsc::channel(100);
//
//     let mut events = Events::new(cnx_receiver);
//     tokio::spawn(telnet_handler(command_receiver, cnx_sender));
//
//
//     loop {
//
//        ui::draw_app(&mut terminal, &app)?;
//
//        match events.next().await {
//            Some(Event::Input(CEvent::Key(KeyEvent { code: KeyCode::Esc, modifiers: _ }))) => {
//                break;
//            }
//            Some(Event::Input(CEvent::Key(key_event))) =>
//                if app_events::handle_key_event(&mut app, &mut command_sender, key_event).await {
//                    break;
//                },
//            Some(Event::Input(_)) => {}
//            Some(Event::Tick) => {}
//            Some(Event::Network(msg)) => app.apply_event(msg),
//
//            None => {}
//        }
//    }
//
//    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
//    disable_raw_mode()?;
//    Ok(())
// }



fn telnet_handler(mut command_receiver: Receiver<String>,
                  mut cnx_sender: Sender<CnxOutput>) -> impl Future<Output=io::Result<()>> {
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

            match command_receiver.try_recv() {
                Ok(msg) => {
                    debug!("sending {:?}", msg);
                    telnet.write(msg.as_bytes()).await
                }
                Err(TryRecvError::Empty) => {
                    debug!("try receive empty !");
                    Ok::<usize, io::Error>(0)
                }
                Err(TryRecvError::Closed) => break,
            }?;



            debug!("|telnet_handler 1|----------------------------------");
            let data = mudnet::next(&mut telnet, &config, &mut cnx_state).await?;
            debug!("|telnet_handler 2|----------------------------------");


            for d in data.iter() {
                cnx_sender.send(d.clone()).await;
            }
            task::yield_now().await;
        }
        Ok(())
    }
}

//
// fn telnet_handler(mut command_receiver: Receiver<String>,
//                   mut cnx_sender: Sender<CnxOutput>) -> impl Future<Output=io::Result<()>> {
//     async move {
//         println!("Start !");
//         //let host = ("edge.xen.prgmr.com",4000);
//         //RcSmxqq6&
//         //aardwolf.org (23.111.136.202) port 4000
//         //let host = "aardwolf.org:4000";
//         let host = ("localhost", 9696); //currymud
//         //let host = ("localhost", 27733);
//         let config = mudnet::MudConfig::default();
//         let mut cnx_state = mudnet::CnxState::new();
//         let mut telnet: Telnet = Telnet::connect(host, 256).await
//             .unwrap_or_else(|_| -> Telnet {
//                 error!("failed to establish connection with {:?}", host);
//                 process::exit(1);
//             });
//         debug!("Connected to the server!");
//
//
//         let user_input = tokio::spawn(async {
//             loop {
//                 match command_receiver.recv().await {
//                     Some(msg) => {
//                         debug!("sending {:?}", msg);
//                         match telnet.write(msg.as_bytes()).await {
//                             Ok(_) => (),
//                             Err(_) => ()
//                         }
//                     }
//                     None => (),
//                 };
//
//                 task::yield_now().await;
//             }
//         });
//
//         let network = tokio::spawn(async {
//             loop {
//                 debug!("|telnet_handler 1|----------------------------------");
//                 match mudnet::next(&mut telnet, &config, &mut cnx_state).await {
//                     Ok(data) => {
//                         debug!("|telnet_handler 2|----------------------------------");
//
//                         for d in data.iter() {
//                             cnx_sender.send(d.clone()).await;
//                         }
//                     }
//                     Err(_) => ()
//                 }
//
//                 task::yield_now().await;
//             }
//         });
//
//         loop {
//             tokio::select! {
//                 _ = user_input => {}
//                 _ = network => {}
//             }
//
//             task::yield_now().await;
//         }
//     }
// }