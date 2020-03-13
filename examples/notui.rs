use std::{
    io::{self, stdout, Write},
    time::Duration,
};
use std::process;

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

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let mut app = App::new();

    let (mut command_sender, mut command_receiver): (Sender<String>, Receiver<String>) =
        mpsc::channel(100);
    let (mut cnx_sender, mut cnx_receiver): (Sender<CnxOutput>, Receiver<CnxOutput>) =
        mpsc::channel(100);


    //let host = ("edge.xen.prgmr.com",4000);
    //RcSmxqq6&
    //aardwolf.org (23.111.136.202) port 4000
    //let host = "aardwolf.org:4000";
    let host = ("localhost", 9696); //currymud
    //let host = ("localhost", 27733);
    let mut tcp_stream = Box::new(tokio::net::TcpStream::connect(host).await
        .unwrap_or_else(|_| -> tokio::net::TcpStream {
            error!("failed to establish connection with {:?}", host);
            process::exit(1);
        }));

    tokio::spawn(mudnet::handler(tcp_stream, command_receiver, cnx_sender));

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