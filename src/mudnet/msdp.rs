use telnet::{Telnet, TelnetOption};
use im::{HashSet, hashset};
use std::io;

use crate::mudnet::mud::options::MSDP;
use super::lexer::{tokenize, Token};

const MSDP_VAR: u8 = 1;
const MSDP_VAL: u8 = 2;
const MSDP_TABLE_OPEN: u8 = 3;
const MSDP_TABLE_CLOSE: u8 = 4;
const MSDP_ARRAY_OPEN: u8 = 5;
const MSDP_ARRAY_CLOSE: u8 = 6;

#[derive(Debug, Clone)]
pub struct MsdpData {
    key: String,
    value: MsdpVal,
}

#[derive(Debug, Clone)]
pub enum MsdpVal {
    Value(String),
    Array(Vec<MsdpVal>),
    Table(Vec<(String, MsdpVal)>),
}

pub async fn send_key_val(telnet: &mut Telnet,
                    k: &String, v: &String) -> io::Result<()> {
    let data: [&[u8]; 4] =
        [&[MSDP_VAR],
            k.as_bytes(),
            &[MSDP_VAL],
            v.as_bytes()
        ];

    telnet.try_subnegotiate(TelnetOption::UnknownOption(MSDP), &data).await?;
    Ok(())
}

enum ParsingState {
    Key,
    Value,
}

enum Shape {
    KeyVal,
    Array,
    Table,
}

/*
 Variables and Values
Variables are send as a typical telnet sub-negotiation having the format: IAC SB MSDP MSDP_VAR <VARIABLE> MSDP_VAL <VALUE> IAC SE. For ease of parsing, variables and values cannot contain the NUL, MSDP_VAL, MSDP_VAR, MSDP_TABLE_OPEN, MSDP_TABLE_CLOSE, MSDP_ARRAY_OPEN, MSDP_ARRAY_CLOSE, or IAC byte. For example:
IAC SB MSDP MSDP_VAR "SEND" MSDP_VAL "HEALTH" IAC SE
The quote characters mean that the encased word is a string, the quotes themselves should not be send.

Tables
Sometimes it's useful to send data as a table, which can be done in MSDP using MSDP_TABLE_OPEN and MSDP_TABLE_CLOSE after MSDP_VAL, and nest variables and values inside the MSDP_TABLE_OPEN and MSDP_TABLE_CLOSE arguments. Tables are called objects in the JSON standard, they're also known as maps, dictionaries, and associative arrays.
A ROOM data table in MSDP would look like:

IAC SB MSDP MSDP_VAR "ROOM" MSDP_VAL MSDP_TABLE_OPEN MSDP_VAR "VNUM" MSDP_VAL "6008" MSDP_VAR "NAME" MSDP_VAL "The forest clearing" MSDP_VAR "AREA" MSDP_VAL "Haon Dor" MSDP_VAR "TERRAIN" MSDP_VAL "forest" MSDP_VAR "EXITS" MSDP_VAL MSDP_TABLE_OPEN MSDP_VAR "n" MSDP_VAL "6011" MSDP_VAR "e" MSDP_VAL "6007" MSDP_TABLE_CLOSE MSDP_TABLE_CLOSE IAC SE


Arrays
Sometimes it's useful to send data as an array, which can be done in MSDP using MSDP_ARRAY_OPEN and MSDP_ARRAY_CLOSE after MSDP_VAL, and nest values inside the MSDP_ARRAY_OPEN and MSDP_ARRAY_CLOSE arguments, with each value preceded by an MSDP_VAL argument.
An array in MSDP would look like:

IAC SB MSDP MSDP_VAR "REPORTABLE_VARIABLES" MSDP_VAL MSDP_ARRAY_OPEN MSDP_VAL "HEALTH" MSDP_VAL "HEALTH_MAX" MSDP_VAL "MANA" MSDP_VAL "MANA_MAX" MSDP_ARRAY_CLOSE IAC SE

The quote characters mean that the encased word is a string, the quotes themselves should not be send.
*/

pub fn parse_msdp(data: &[u8]) -> io::Result<MsdpData> {
    let delims: HashSet<u8> = hashset![MSDP_VAR, MSDP_VAL, MSDP_TABLE_OPEN, MSDP_TABLE_CLOSE, MSDP_ARRAY_OPEN, MSDP_ARRAY_CLOSE];
    let tokens: Vec<Token> = tokenize(data, &delims);
    parse_tokens(&tokens)
}

fn parse_tokens(tokens: &Vec<Token>) -> io::Result<MsdpData> {
    let (key, _) = parse_var(tokens, 0)?;


    let (value, _) = parse_value(tokens, 2)?;

    Ok(
        MsdpData {
            key,
            value,
        })
}


fn string_from_u8(data: &[u8]) -> io::Result<String> {
    let d = std::str::from_utf8(data)
        .map_err(|e| -> io::Error {
            io::Error::new(io::ErrorKind::InvalidInput, e.to_string())
        })?;
    Ok(String::from(d))
}

fn parse_var(tokens: &Vec<Token>, pos: usize) -> io::Result<(String, usize)> {
    match tokens.get(pos) {
        Some(Token::Delim(MSDP_VAR)) => Ok(()),
        _ =>
            Err(io::Error::new(io::ErrorKind::InvalidInput, format!("expected MSDP_VAR ({}), found {:?}", MSDP_VAR, tokens.get(pos))))
    }?;

    let key = match tokens.get(pos + 1) {
        Some(Token::Data(d)) => string_from_u8(*d),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("expected data found {:?}", tokens.get(pos + 1))))
    }?;

    Ok((key, pos + 2))
}

fn parse_value(tokens: &Vec<Token>, pos: usize) -> io::Result<(MsdpVal, usize)> {
    match tokens.get(pos) {
        Some(Token::Delim(MSDP_VAL)) => Ok(()),
        _ =>
            Err(io::Error::new(io::ErrorKind::InvalidInput, format!("expected MSDP_VAL ({}), found {:?}", MSDP_VAL, tokens.get(pos))))
    }?;

    match tokens.get(pos + 1) {
        Some(Token::Delim(MSDP_TABLE_OPEN)) =>
            parse_table(tokens, pos + 2),
        Some(Token::Delim(MSDP_ARRAY_OPEN)) =>
            parse_array(tokens, pos + 2),
        Some(Token::Data(d)) =>
            string_from_u8(*d).map(|data| -> (MsdpVal, usize) { (MsdpVal::Value(data), pos + 2) }),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "malformed paylod"))
    }
}

fn is_delim(value: &Option<&Token>, expected: u8) -> bool {
    match value {
        Some(Token::Delim(found)) => *found == expected,
        _ => false
    }
}
/*
IAC SB MSDP
   MSDP_VAR "REPORTABLE_VARIABLES"
   MSDP_VAL MSDP_ARRAY_OPEN
               MSDP_VAL "HEALTH"
               MSDP_VAL "HEALTH_MAX"
               MSDP_VAL "MANA"
               MSDP_VAL "MANA_MAX"
   MSDP_ARRAY_CLOSE IAC SE

*/
fn parse_array(tokens: &Vec<Token>, pos: usize) -> io::Result<(MsdpVal, usize)> {
    let mut values: Vec<MsdpVal> = Vec::new();

    let mut i = pos;

    while i < tokens.len() && !(is_delim(&tokens.get(i), MSDP_ARRAY_CLOSE)) {
        let (val, next_pos) = parse_value(tokens, i)?;
        values.push(val);
        i = next_pos;
    }


    match tokens.get(i) {
        Some(Token::Delim(MSDP_ARRAY_CLOSE)) => Ok(()),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "reach end of tokens without founding MSDP_ARRAY_CLOSE"))
    }?;


    Ok((MsdpVal::Array(values), i + 1))
}

/*
IAC SB MSDP
   MSDP_VAR "ROOM"
   MSDP_VAL MSDP_TABLE_OPEN
       MSDP_VAR "VNUM" MSDP_VAL "6008"
       MSDP_VAR "NAME" MSDP_VAL "The forest clearing"
       MSDP_VAR "AREA" MSDP_VAL "Haon Dor"
       MSDP_VAR "TERRAIN" MSDP_VAL "forest"
       MSDP_VAR "EXITS" MSDP_VAL
           MSDP_TABLE_OPEN
               MSDP_VAR "n" MSDP_VAL "6011"
               MSDP_VAR "e" MSDP_VAL "6007"
           MSDP_TABLE_CLOSE
   MSDP_TABLE_CLOSE IAC SE
*/

fn parse_table(tokens: &Vec<Token>, pos: usize) -> io::Result<(MsdpVal, usize)> {
    let mut values: Vec<(String, MsdpVal)> = Vec::new();

    let mut i = pos;

    while i < tokens.len() && !(is_delim(&tokens.get(i), MSDP_TABLE_CLOSE)) {
        let (key, next_pos) = parse_var(tokens, i)?;

        let (val, next_pos2) = parse_value(tokens, next_pos)?;
        values.push((key, val));
        i = next_pos2;
    }


    match tokens.get(i) {
        Some(Token::Delim(MSDP_TABLE_CLOSE)) => Ok(()),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "reach end of tokens without founding MSDP_TABLE_CLOSE"))
    }?;


    Ok((MsdpVal::Table(values), i + 1))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io;

    #[test]
    fn key_val() -> io::Result<()> {
        let tokens =
            vec![Token::Delim(MSDP_VAR),
                 Token::Data("SEND".as_bytes()),
                 Token::Delim(MSDP_VAL),
                 Token::Data("HEALTH".as_bytes())
            ];

        match parse_tokens(&tokens)? {
            MsdpData { key: k, value: MsdpVal::Value(v) } =>
                {
                    assert_eq!(k, String::from("SEND"));
                    assert_eq!(v, String::from("HEALTH"));
                    Ok(())
                }
            _ => panic!("fail")
        }
    }

    fn is_val(found: &Option<&MsdpVal>, expected: &str) {
        match found {
            Some(MsdpVal::Value(v)) => assert_eq!(*v, String::from(expected)),
            _ => panic!("fail")
        }
    }

    #[test]
    fn array() -> io::Result<()> {
        let tokens =
            vec![Token::Delim(MSDP_VAR),
                 Token::Data("REPORTABLE_VARIABLES".as_bytes()),
                 Token::Delim(MSDP_VAL), Token::Delim(MSDP_ARRAY_OPEN),
                 Token::Delim(MSDP_VAL), Token::Data("HEALTH".as_bytes()),
                 Token::Delim(MSDP_VAL), Token::Data("HEALTH_MAX".as_bytes()),
                 Token::Delim(MSDP_VAL), Token::Data("MANA".as_bytes()),
                 Token::Delim(MSDP_VAL), Token::Data("MANA_MAX".as_bytes()),
                 Token::Delim(MSDP_ARRAY_CLOSE)
            ];

        match parse_tokens(&tokens)? {
            MsdpData { key: k, value: MsdpVal::Array(vec) } =>
                {
                    assert_eq!(k, String::from("REPORTABLE_VARIABLES"));
                    is_val(&vec.get(0), "HEALTH");
                    is_val(&vec.get(1), "HEALTH_MAX");
                    is_val(&vec.get(2), "MANA");
                    is_val(&vec.get(3), "MANA_MAX");
                    Ok(())
                }
            _ => panic!("fail")
        }
    }

    fn is_key_val(found: &Option<&(String, MsdpVal)>, expected_key: &str, expected_val: &str) {
        match found {
            Some((k, MsdpVal::Value(v))) => {
                assert_eq!(*k, String::from(expected_key));
                assert_eq!(*v, String::from(expected_val))
            }
            _ => panic!("fail")
        }
    }

    #[test]
    fn table() -> io::Result<()> {
        let tokens =
            vec![Token::Delim(MSDP_VAR),
                 Token::Data("ROOM".as_bytes()),
                 Token::Delim(MSDP_VAL), Token::Delim(MSDP_TABLE_OPEN),
                 Token::Delim(MSDP_VAR), Token::Data("VNUM".as_bytes()), Token::Delim(MSDP_VAL), Token::Data("6008".as_bytes()),
                 Token::Delim(MSDP_VAR), Token::Data("NAME".as_bytes()), Token::Delim(MSDP_VAL), Token::Data("The forest clearing".as_bytes()),
                 Token::Delim(MSDP_VAR), Token::Data("AREA".as_bytes()), Token::Delim(MSDP_VAL), Token::Data("Haon Dor".as_bytes()),
                 Token::Delim(MSDP_VAR), Token::Data("TERRAIN".as_bytes()), Token::Delim(MSDP_VAL), Token::Data("forest".as_bytes()),
                 Token::Delim(MSDP_VAR), Token::Data("EXITS".as_bytes()), Token::Delim(MSDP_VAL),
                 Token::Delim(MSDP_TABLE_OPEN),
                 Token::Delim(MSDP_VAR), Token::Data("n".as_bytes()), Token::Delim(MSDP_VAL), Token::Data("6011".as_bytes()),
                 Token::Delim(MSDP_VAR), Token::Data("e".as_bytes()), Token::Delim(MSDP_VAL), Token::Data("6007".as_bytes()),
                 Token::Delim(MSDP_TABLE_CLOSE),
                 Token::Delim(MSDP_TABLE_CLOSE)
            ];

        match parse_tokens(&tokens)? {
            MsdpData { key: k, value: MsdpVal::Table(vec) } =>
                {
                    assert_eq!(k, String::from("ROOM"));
                    is_key_val(&vec.get(0), "VNUM", "6008");
                    is_key_val(&vec.get(1), "NAME", "The forest clearing");
                    is_key_val(&vec.get(2), "AREA", "Haon Dor");
                    is_key_val(&vec.get(3), "TERRAIN", "forest");

                    match vec.get(4) {
                    Some((k, MsdpVal::Table(inner))) => {
                        assert_eq!(*k, String::from("EXITS"));
                        is_key_val(&inner.get(0), "n", "6011");
                        is_key_val(&inner.get(1), "e", "6007");
                        Ok(())
                    },
                    _ => Ok(())
                }
                },
            _ => panic!("fail")
        }
    }
}

