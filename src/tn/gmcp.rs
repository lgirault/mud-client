
use telnet::{Telnet, TelnetOption};
use std::io;
use crate::tn::mud::options::GMCP;
//client - IAC   SB GMCP 'MSDP {"LIST" : "COMMANDS"}' IAC SE

pub fn list_command(telnet: &mut Telnet) -> io::Result<()> {

    let msg = "MSDP {\"LIST\" : \"COMMANDS\"}";

    telnet.try_subnegotiate(TelnetOption::UnknownOption(GMCP), &[msg.as_bytes()])?;

    Ok(())
}