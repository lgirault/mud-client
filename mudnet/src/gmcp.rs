use crate::mud::options::GMCP;
use std::io;
use telnet::{TelnetOption, TelnetWriter};
//client - IAC   SB GMCP 'MSDP {"LIST" : "COMMANDS"}' IAC SE

pub async fn list_command(telnet: &mut TelnetWriter<'_>) -> io::Result<()> {
    let msg = "MSDP {\"LIST\" : \"COMMANDS\"}";

    telnet
        .try_subnegotiate(TelnetOption::UnknownOption(GMCP), &[msg.as_bytes()])
        .await?;

    Ok(())
}
