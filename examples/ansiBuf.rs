use core::str;
use futures::{future::Future, stream::Stream};
use std::io;
use termit_ansi::{
    model::{Ansi as TAnsi, AnsiError, Ctl},
    parser::{AnsiDeviceParser,
             AnsiHostParser,
             AnsiParser,
             DebugWrite},
};
use std::fmt::{self, Write};
use termit_ansi::model::AnsiHandler;


#[derive(Debug)]
pub enum Ansi {
    /// The error and raw bytes that are invalid
    Error(AnsiError, Vec<u8>),
    /// Escape - either as part of a sequence or on it's own
    Esc,
    /// normal or unicode character
    /// * <c>+
    Data(String),
    /// Ansi command
    Command(Ctl, u32, String, Vec<u8>),
}

#[derive(Debug)]
struct AnsiVec {
    buf: Vec<Ansi>
}

impl AnsiVec {
    pub fn new() -> AnsiVec {
        AnsiVec {
            buf: Vec::new()
        }
    }

    fn len(&self) -> usize {
        self.buf.len()
    }

    fn compact(&mut self) {
        let mut init: Option<String> = None;
        let mut new_buf= Vec::new();

        let end = self.buf.drain(..).into_iter().fold(init, |current, a| ->  Option<String> {
            match a {
                Ansi::Data(s) => {
                    let mut cs = current.unwrap_or(String::new());
                    cs.push_str(s.as_str());
                    Some(cs)
                },
                other=> {
                    current.map(| s| -> () {
                        new_buf.push(Ansi::Data(s) )

                    });
                    new_buf.push(other);
                    None
                },
            }
        });

        end.map(| s| -> () {
            new_buf.push(Ansi::Data(s) )
        });

        self.buf = new_buf;
    }

    fn clear(&mut self) {
        self.buf.clear()
    }
}

impl AnsiHandler for AnsiVec {
    fn handle(&mut self, tansi: TAnsi, _raw: &[u8]) {
        let ansi = match tansi {
            TAnsi::Data(str) =>
                Ansi::Data(String::from(str)),
            TAnsi::Esc => Ansi::Esc,
            TAnsi::Command(c, f, p, t) =>
                Ansi::Command(c, f, String::from(p), Vec::from(t)),
            TAnsi::Error(err, raw) =>
                Ansi::Error(err, Vec::from(raw))
        };
        self.buf.push(ansi)
    }
}

// impl<'a> fmt::Write for AnsiBuf<'a> {
//     fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
//         self.buf.push(Ansi::Data(s));
//         Ok(())
//     }
// }

fn main() -> Result<(), std::fmt::Error> {
    let mut handler = AnsiVec::new();
    let mut parser_ansi = AnsiParser::new([0u8; 32]);


    let input = "We\'ll create a new character named \"Lorilan.\" OK? [\u{1b}[36my\u{1b}[0mes/\u{1b}[36mn\u{1b}[0mo]\r\n";


    let buf = input.as_bytes();
    print!("input: {:?}\r\n", str::from_utf8(buf));

    parser_ansi.parse(&mut handler, &buf);
    handler.compact();
    if handler.len() != 0 {
        // handler.write_str("\r\n")?;
        print!("{:?}", handler);
        handler.clear();
    }

    Ok(())
}
