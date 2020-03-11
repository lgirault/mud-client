use im::hashmap::HashMap;
use telnet::{Telnet, TelnetEvent, TelnetOption, NegotiationAction};
use std::io;
use log::{debug, warn};
use std::borrow::Borrow;

mod lexer;
mod msdp;
mod mtts;
pub mod gmcp;
pub mod mud;

use msdp::MsdpData;

pub struct MudConfig {
    pub terminal_type: &'static str,
    pub features: mtts::Features,
}

impl MudConfig {
    pub fn default() -> MudConfig {
        MudConfig {
            terminal_type: mtts::terminal_type::XTERM,
            features: mtts::Features::ANSI | mtts::Features::UTF8,
        }
    }
}


#[derive(Debug, Clone)]
pub struct CnxState {
    negociated_options: HashMap<u8, NegotiationState>,
    mtts_num_call: u8,
}

impl CnxState {
    pub fn new() -> CnxState {
        CnxState {
            negociated_options: HashMap::new(),
            mtts_num_call: 0,
        }
    }

    pub fn add_negociated_option(&mut self, opt: NegotiationState) -> () {
        self.negociated_options.insert(opt.option.to_byte(), opt);
    }


    pub fn negotiation_state(&self, option: &TelnetOption) -> NegotiationState {
        match self.negociated_options.get(&option.to_byte()) {
            Some(state) => state.clone(),
            None => NegotiationState::new(*option)
        }
    }
}

#[derive(Debug)]
pub enum Negotiation {
    Negotiation(NegotiationAction, TelnetOption),
    Subnegotiation(TelnetOption, Box<[u8]>),
}

#[derive(Debug, Clone)]
pub struct NegotiationState {
    pub option: TelnetOption,

    /* WILL (option code)  251  Indicates the desire to begin
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
    */
    pub received_will: bool,
    pub received_wont: bool,
    pub received_do: bool,
    pub received_dont: bool,
    pub send_will: bool,
    pub send_wont: bool,
    pub send_do: bool,
    pub send_dont: bool,
}

const SUPPORTED_OPTIONS: [TelnetOption; 2] =
    [TelnetOption::TTYPE,
        TelnetOption::UnknownOption(mud::options::GMCP)
    ];


impl NegotiationState {
    fn new(option: TelnetOption) -> NegotiationState {
        NegotiationState {
            option,
            received_will: false,
            received_wont: false,
            received_do: false,
            received_dont: false,
            send_will: false,
            send_wont: false,
            send_do: false,
            send_dont: false,
        }
    }

    fn shoud_negotiate(&self) -> bool {
        SUPPORTED_OPTIONS.contains(&self.option) &&
            !self.received_dont &&
            !self.received_wont &&
            !self.send_do
    }

    fn is_active(&self) -> bool {
        (self.received_will || self.received_do)
            && self.send_do
            && !self.received_wont
            && !self.received_dont
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub negotiations: Vec<Negotiation>,
    pub data: String,
}

async fn read_chunk(telnet: &mut Telnet) -> Result<Chunk, io::Error> {
    let mut data = String::new();
    let mut negotiations: Vec<Negotiation> = Vec::new();

    debug!("|read_chunk 1|----------------------------------");
    let event = telnet.read().await?;
    debug!("|read_chunk 2|----------------------------------");

    match event {
        TelnetEvent::Negotiation(act, opt) =>
            negotiations.push(Negotiation::Negotiation(act, opt)),

        TelnetEvent::Subnegotiation(opt, negoData) =>
            negotiations.push(Negotiation::Subnegotiation(opt, negoData)),
        TelnetEvent::Data(buffer) => {
            let d =
                std::str::from_utf8(buffer.borrow()).map_err(|e| -> io::Error {
                    io::Error::new(io::ErrorKind::InvalidInput, e.to_string())
                })?;
            data.push_str(d);
        }
        TelnetEvent::UnknownIAC(code) =>
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                      format!("unknown IAC command {}", code))),
        TelnetEvent::NoData =>
            (),
        TelnetEvent::TimedOut =>
            return Err(io::Error::new(io::ErrorKind::TimedOut, "telnet event timed out")),

        TelnetEvent::Error(msg) => {
            warn!("Telnet error event : {}", msg);
            //    return Err(io::Error::new(io::ErrorKind::Other, msg)),
        }
    }

    Ok(Chunk {
        data,
        negotiations,
    })
}


async fn handle_sub_negotiations(telnet: &mut Telnet,
                                 config: &MudConfig,
                                 cnx_state: &mut CnxState,
                                 output: &mut Vec<CnxOutput>,
                                 opt: &TelnetOption,
                                 data: &Box<[u8]>) -> io::Result<()> {
    match opt {
        TelnetOption::TTYPE =>
            {
                debug!("handling sub negotiations for TTYPE");
                mtts::handle_sub_negotiations(telnet, config, cnx_state).await
            }
        TelnetOption::UnknownOption(mud::options::MSDP) => {
            let msdp_data = msdp::parse_msdp(data.borrow())?;
            output.push(CnxOutput::Msdp(msdp_data));
            Ok(())
        }
        _ => {
            warn!("ignoring subnegotiation for {:?}", (opt, data));
            Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{:?}", *opt)))
        }
    }
}

async fn handle_negotiations(telnet: &mut Telnet,
                             config: &MudConfig,
                             state: &mut CnxState,
                             output: &mut Vec<CnxOutput>,
                             negotiations: &Vec<Negotiation>) -> io::Result<()> {
    for n in negotiations.iter() {
        match n {
            Negotiation::Negotiation(action, opt)
            if *action == NegotiationAction::Do ||
                *action == NegotiationAction::Will =>
                negotiate_answer(telnet, state, action, opt).await,
            Negotiation::Subnegotiation(option, data) =>
                handle_sub_negotiations(telnet, config, state, output, option, data).await,
            _ => Ok(())
        }?;
    }

    Ok(())
}


pub struct MudNet {
    telnet: Telnet,
    config: MudConfig,
    state: CnxState,
}

impl MudNet {
    pub async fn next(&mut self) -> io::Result<Vec<CnxOutput>> {
        next(&mut self.telnet, &self.config, &mut self.state).await
    }

    pub async fn negotiate(&mut self,
                           state: &mut CnxState,
                           opt: &TelnetOption) -> io::Result<()> {
        negotiate(&mut self.telnet, state, opt).await
    }
}


pub async fn negotiate(telnet: &mut Telnet,
                       state: &mut CnxState,
                       opt: &TelnetOption) -> io::Result<()> {
    let mut nego_state = state.negotiation_state(opt);
    do_negotiate(telnet, &mut nego_state).await?;
    state.add_negociated_option(nego_state);
    Ok(())
}

pub async fn negotiate_answer(telnet: &mut Telnet,
                              state: &mut CnxState,
                              action: &NegotiationAction,
                              opt: &TelnetOption) -> io::Result<()> {
    let mut nego_state = state.negotiation_state(opt);
    do_negotiate(telnet, &mut nego_state).await?;

    nego_state.received_do = nego_state.received_do || *action == NegotiationAction::Do;
    nego_state.received_will = nego_state.received_will || *action == NegotiationAction::Will;

    state.add_negociated_option(nego_state);
    Ok(())
}

async fn do_negotiate(telnet: &mut Telnet,
                      nego_state: &mut NegotiationState) -> io::Result<()> {
    if nego_state.shoud_negotiate() {
        debug!("negotiating Do for supported option {:?}", nego_state.option);
        telnet.try_negotiate(NegotiationAction::Will, nego_state.option).await?;
        telnet.try_negotiate(NegotiationAction::Do, nego_state.option).await?;
        nego_state.send_will = true;
        nego_state.send_do = true;
    } else {
        warn!("negotiating Wont for unsupported {:?}", nego_state.option);
        telnet.try_negotiate(NegotiationAction::Wont, nego_state.option).await?;
        nego_state.send_wont = true;
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum CnxOutput {
    Data(String),
    Msdp(MsdpData),
}

pub async fn next(telnet: &mut Telnet,
                  config: &MudConfig,
                  cnx_state: &mut CnxState) -> io::Result<Vec<CnxOutput>> {
    let chunk = read_chunk(telnet).await?;

    let mut output: Vec<CnxOutput> = Vec::new();

    debug!("{} negociations to process: ", chunk.negotiations.len());
    for n in chunk.negotiations.iter() {
        debug!("- {:?}", n);
    }

    handle_negotiations(telnet, &config, cnx_state, &mut output, &chunk.negotiations).await?;

    output.push(CnxOutput::Data(chunk.data));

    Ok(output)
}

