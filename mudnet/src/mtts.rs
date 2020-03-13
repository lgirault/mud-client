/*
    Mud Terminal Type standard:

    https://mudhalla.net/tintin/protocols/mtts/
*/
use super::{CnxState, MudConfig};
use bitflags::bitflags;
use std::io;
use telnet::{Telnet, TelnetOption, TelnetWriter};

bitflags! {
    pub struct Features: u16 {
        const ANSI              = 0b0000_0000_0001;
        const VT100             = 0b0000_0000_0010;
        const UTF8              = 0b0000_0000_0100;
        const _256COLORS        = 0b0000_0000_1000;
        const MOUSE_TRACKING    = 0b0000_0001_0000;
        const OSC_COLOR_PALETTE = 0b0000_0010_0000;
        const SCREEN_READER     = 0b0000_0100_0000;
        const PROXY             = 0b0000_1000_0000;
        const TRUECOLOR         = 0b0001_0000_0000;
        const MNES              = 0b0010_0000_0000;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_vector_bit() {
        let feat: Features = Features::ANSI | Features::UTF8 | Features::_256COLORS;
        assert_eq!(format!("{}", feat.bits), "13");
    }
}

pub mod terminal_type {
    pub const DUMB: &'static str = "DUMB";
    //	Terminal has no ANSI color or VT100 support.
    pub const ANSI: &'static str = "ANSI";
    // Terminal supports the common ANSI color codes. Supporting blink and underline is optional.
    pub const VT100: &'static str = "VT100";
    //	Terminal supports most VT100 codes and ANSI color codes.
    pub const XTERM: &'static str = "XTERM"; //	Terminal supports all VT100 and ANSI color codes, 256 colors, mouse tracking, and all commonly used xterm console codes.
}

const IS: u8 = 0;

pub async fn handle_sub_negotiations(
    telnet: &mut TelnetWriter<'_>,
    config: &MudConfig,
    cnx_state: &mut CnxState,
) -> io::Result<()> {
    let feature_msg: String;

    let msg = (if cnx_state.mtts_num_call == 0 {
        Ok(config.client_name.as_bytes())
    } else if cnx_state.mtts_num_call == 1 {
        Ok(config.terminal_type.as_bytes())
    } else if cnx_state.mtts_num_call == 2 {
        feature_msg = format!("MTTS {}", config.features.bits);
        Ok(feature_msg.as_bytes())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no more than 3",
        ))
    })?;

    telnet
        .try_subnegotiate(TelnetOption::TTYPE, &[&[IS], msg])
        .await?;

    cnx_state.mtts_num_call = cnx_state.mtts_num_call + 1;
    Ok(())
}
