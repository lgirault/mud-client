use im::hashmap::HashMap;
use telnet::{Telnet, TelnetEvent, TelnetOption, NegotiationAction};
use std::io::{self, Write, Read};
use log::{debug, warn};

mod mtts;
pub mod gmcp;
pub mod mud;


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


}

#[derive(Debug)]
pub struct Chunk {
    pub negotiations: Vec<Negotiation>,
    pub data: String,
}

fn read_chunk(telnet: &mut Telnet) -> Result<Chunk, io::Error> {
    let mut data = String::new();
    let mut negotiations: Vec<Negotiation> = Vec::new();

    loop {
        let event = telnet.read_nonblocking()?;

        match event {
            TelnetEvent::Negotiation(act, opt) =>
                negotiations.push(Negotiation::Negotiation(act, opt)),

            TelnetEvent::Subnegotiation(opt, negoData) =>
                negotiations.push(Negotiation::Subnegotiation(opt, negoData)),
            TelnetEvent::Data(buffer) => {
                for b in buffer.iter() {
                    if b.is_ascii() {
                        data.push(*b as char)
                    } else {
                        warn!("ignoring character '{}'", b);
                    }
                }
            }
            TelnetEvent::UnknownIAC(code) =>
                return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                          format!("unknown IAC command {}", code))),
            TelnetEvent::NoData =>
                break,
            TelnetEvent::TimedOut =>
                return Err(io::Error::new(io::ErrorKind::TimedOut, "telnet event timed out")),

            TelnetEvent::Error(msg) => {
                warn!("Telnet error event : {}", msg);
                //    return Err(io::Error::new(io::ErrorKind::Other, msg)),
                break;
            }
        }
    }

    Ok(Chunk {
        data,
        negotiations,
    })
}


fn handle_sub_negotiations(telnet: &mut Telnet,
                           config: &MudConfig,
                           cnx_state: &mut CnxState,
                           opt: &TelnetOption,
                           data: &Box<[u8]>) -> io::Result<()> {
    match opt {
        TelnetOption::TTYPE =>
            {
                debug!("handling sub negotiations for TTYPE");
                mtts::handle_sub_negotiations(telnet, config, cnx_state)
            }
        _ => {
            warn!("ignoring subnegotiation for {:?}", (opt, data));
            Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{:?}", *opt)))
        }
    }
}

fn handle_negotiations(telnet: &mut Telnet,
                       config: &MudConfig,
                       cnx_state: &CnxState,
                       negotiations: &Vec<Negotiation>) -> io::Result<CnxState> {
    let mut state = cnx_state.clone();

    for n in negotiations.iter() {
        match n {
            Negotiation::Negotiation(action, opt)
            if *action == NegotiationAction::Do ||
                *action == NegotiationAction::Will =>
                negotiate_answer(telnet, &mut state, action, opt),
            Negotiation::Subnegotiation(option, data) =>
                handle_sub_negotiations(telnet, config, &mut state, option, data),
            _ => Ok(())
        }?;
    }

    Ok(state)
}


pub struct MudNet {
    telnet: Telnet,
    config: MudConfig,
}

impl MudNet {
    pub fn next(&mut self,
                cnx_state: &CnxState) -> io::Result<(CnxState, String)> {
        next(&mut self.telnet, &self.config, cnx_state)
    }
}


pub fn negotiate(telnet: &mut Telnet,
                 state: &mut CnxState,
                 opt: &TelnetOption) -> io::Result<()> {
    let mut nego_state = state.negotiation_state(opt);
    do_negotiate(telnet, &mut nego_state)?;
    state.add_negociated_option(nego_state);
    Ok(())
}

pub fn negotiate_answer(telnet: &mut Telnet,
                        state: &mut CnxState,
                        action: &NegotiationAction,
                        opt: &TelnetOption) -> io::Result<()> {
    let mut nego_state = state.negotiation_state(opt);
    do_negotiate(telnet, &mut nego_state)?;

    nego_state.received_do = nego_state.received_do || *action == NegotiationAction::Do;
    nego_state.received_will = nego_state.received_will || *action == NegotiationAction::Will;

    state.add_negociated_option(nego_state);
    Ok(())
}

fn do_negotiate(telnet: &mut Telnet,
                nego_state: &mut NegotiationState) -> io::Result<()> {
    if nego_state.shoud_negotiate() {
        debug!("negotiating Do for supported option {:?}", nego_state.option);
        telnet.try_negotiate(NegotiationAction::Will, nego_state.option)?;
        telnet.try_negotiate(NegotiationAction::Do, nego_state.option)?;
        nego_state.send_will = true;
        nego_state.send_do = true;
    } else {
        warn!("negotiating Wont for unsupported {:?}", nego_state.option);
        telnet.try_negotiate(NegotiationAction::Wont, nego_state.option)?;
        nego_state.send_wont = true;
    }

    Ok(())
}

pub fn next(telnet: &mut Telnet,
            config: &MudConfig,
            cnx_state: &CnxState) -> io::Result<(CnxState, String)> {
    let chunk = read_chunk(telnet)?;

    debug!("{} negociations to process: ", chunk.negotiations.len());
    for n in chunk.negotiations.iter() {
        debug!("- {:?}", n);
    }

    let new_cnx_state = handle_negotiations(telnet, &config, cnx_state, &chunk.negotiations)?;

    Ok((new_cnx_state, chunk.data))
}

