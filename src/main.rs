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

use std::net::TcpStream;
use std::io::{Write, Read};


mod telnet {
    //https://tools.ietf.org/html/rfc854  [Page 14]
    /*
    NAME               CODE              MEANING

      SE                  240    End of subnegotiation parameters.
      NOP                 241    No operation.
      Data Mark           242    The data stream portion of a Synch.
                                 This should always be accompanied
                                 by a TCP Urgent notification.
      Break               243    NVT character BRK.
      Interrupt Process   244    The function IP.
      Abort output        245    The function AO.
      Are You There       246    The function AYT.
      Erase character     247    The function EC.
      Erase Line          248    The function EL.
      Go ahead            249    The GA signal.
      SB                  250    Indicates that what follows is
                                 subnegotiation of the indicated
                                 option.
      WILL (option code)  251    Indicates the desire to begin
                                 performing, or confirmation that
                                 you are now performing, the
                                 indicated option.
      WON'T (option code) 252    Indicates the refusal to perform,
                                 or continue performing, the
                                 indicated option.
      DO (option code)    253    Indicates the request that the
                                 other party perform, or
                                 confirmation that you are expecting
                                 the other party to perform, the
                                 indicated option.
      DON'T (option code) 254    Indicates the demand that the
                                 other party stop performing,
                                 or confirmation that you are no
                                 longer expecting the other party
                                 to perform, the indicated option.
      IAC                 255    Data Byte 255.
    */
    pub const SE: u8 = 0xF0;    // 240 - End of subnegotiation parameters.
    pub const NOP: u8 = 0xF1;   // 241 - No operation.
    pub const SB: u8 = 0xFA;    // 250 - start of subnegotiation
    pub const WILL: u8 = 0xFB;  // 251
    pub const WONT: u8 = 0xFC;  // 252
    pub const DO: u8 = 0xFD;    // 253
    pub const DONT: u8 = 0xFE;  // 254
    pub const IAC: u8 = 0xFF;   // 255 - Interpret As Command
    pub const GMCP: u8 = 0xC9;  // 201 - GMCP sequence
}


fn main() -> Result<(), failure::Error> {

    let mut buffer:[u8;32] = [0; 32];
    println!("{:X?}", buffer);

    //aardwolf.org (23.111.136.202) port 4000
    let mut stream = TcpStream::connect("aardwolf.org:4000")?;


    println!("Connected to the server!");
    stream.read(&mut buffer)?;
    println!("{:X?}", buffer);
    //[FF, FB, 56,
    // FF, FB, 55,
    // FF, FB, 66,
    // FF, FB, C8,
    // FF, FB, C9,
    // FF, FD, 66,
    // FF, FD, 18,
    // FF, FD, 1F,
    // 23, 23, 23, 23, 23, 23, 23, 23]
    //IAC WILL 56
    //IAC WILL 55
    //IAC WILL 66
    //IAC WILL C8
    //IAC WILL GMCP
    //IAC DO   66
    //IAC DO   18
    //IAC DO   1F
    Ok(())
}