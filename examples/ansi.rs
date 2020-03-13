use core::str;
use futures::{future::Future, stream::Stream};
use std::fmt::Write;
use std::io;
use termit_ansi::model::AnsiHandler;
use termit_ansi::{
    model::Ansi,
    parser::{AnsiDeviceParser, AnsiHostParser, AnsiParser, DebugWrite},
};

fn main() -> Result<(), std::fmt::Error> {
    let mut data = [0u8; 1024];
    let mut handler = DebugWrite::new(&mut data);
    let mut parser_ansi = AnsiParser::new([0u8; 32]);
    let mut parser = AnsiDeviceParser::new([0u8; 32]);

    let input = "We\'ll create a new character named \"Lorilan.\" OK? [\u{1b}[36my\u{1b}[0mes/\u{1b}[36mn\u{1b}[0mo]\r\n";

    let buf = input.as_bytes();
    print!("input: {:?}\r\n", str::from_utf8(buf));

    //let mut pa = (parser_ansi.take().unwrap().write(buf).2).unwrap();
    parser_ansi.parse(&mut handler, &buf);
    if handler.len() != 0 {
        handler.write_str("\r\n")?;
        print!("{}", handler.as_str());
        handler.clear();
    }

    parser.parse(&mut handler, &buf);
    if handler.len() != 0 {
        handler.write_str("\r\n")?;
        print!("{}", handler.as_str());

        handler.clear();
    }

    Ok(())
}
