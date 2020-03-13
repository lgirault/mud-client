
use telnet::{TelnetOption, TelnetWriter};
use std::io;
use crate::mudnet::mud::options::GMCP;
//client - IAC   SB GMCP 'MSDP {"LIST" : "COMMANDS"}' IAC SE

pub async fn list_command(telnet: &mut TelnetWriter<'_>) -> io::Result<()> {

    let msg = "MSDP {\"LIST\" : \"COMMANDS\"}";

    telnet.try_subnegotiate(TelnetOption::UnknownOption(GMCP), &[msg.as_bytes()]).await?;

    Ok(())
}