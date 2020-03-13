use futures::Future;
use im::hashmap::HashMap;
use log::{debug, warn};
use std::borrow::Borrow;
use std::io;
use telnet::{NegotiationAction, Telnet, TelnetEvent, TelnetOption, TelnetWriter};
use tokio::sync::mpsc::{self, error::TryRecvError, Receiver, Sender};
use tokio::task;

pub mod gmcp;
mod lexer;
mod msdp;
mod mtts;
pub mod mud;

use msdp::MsdpData;

pub struct MudConfig {
    pub client_name: String,
    pub terminal_type: &'static str,
    pub features: mtts::Features,
}

impl MudConfig {
    pub fn default() -> MudConfig {
        MudConfig {
            client_name: String::from("mudnet"),
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
            None => NegotiationState::new(*option),
        }
    }
}

#[derive(Debug, Clone)]
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

const SUPPORTED_OPTIONS: [TelnetOption; 2] = [
    TelnetOption::TTYPE,
    TelnetOption::UnknownOption(mud::options::GMCP),
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
        SUPPORTED_OPTIONS.contains(&self.option)
            && !self.received_dont
            && !self.received_wont
            && !self.send_do
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

pub async fn read_chunk(telnet: &mut Telnet<'_>) -> Result<Chunk, io::Error> {
    let mut data = String::new();
    let mut negotiations: Vec<Negotiation> = Vec::new();

    let event = telnet.read().await?;

    match event {
        TelnetEvent::Negotiation(act, opt) => negotiations.push(Negotiation::Negotiation(act, opt)),

        TelnetEvent::Subnegotiation(opt, negoData) => {
            negotiations.push(Negotiation::Subnegotiation(opt, negoData))
        }
        TelnetEvent::Data(buffer) => {
            let d = std::str::from_utf8(buffer.borrow()).map_err(|e| -> io::Error {
                io::Error::new(io::ErrorKind::InvalidInput, e.to_string())
            })?;
            data.push_str(d);
        }
        TelnetEvent::UnknownIAC(code) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown IAC command {}", code),
            ))
        }
        TelnetEvent::NoData => (),
        TelnetEvent::TimedOut => {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "telnet event timed out",
            ))
        }

        TelnetEvent::Error(msg) => {
            warn!("Telnet error event : {}", msg);
            //    return Err(io::Error::new(io::ErrorKind::Other, msg)),
        }
    }

    Ok(Chunk { data, negotiations })
}

async fn handle_sub_negotiations(
    telnet: &mut TelnetWriter<'_>,
    config: &MudConfig,
    cnx_state: &mut CnxState,
    opt: &TelnetOption,
    data: &Box<[u8]>,
) -> io::Result<Option<CnxOutput>> {
    match opt {
        TelnetOption::TTYPE => {
            debug!("handling sub negotiations for TTYPE");
            mtts::handle_sub_negotiations(telnet, config, cnx_state).await;
            Ok(None)
        }
        TelnetOption::UnknownOption(mud::options::MSDP) => {
            let msdp_data = msdp::parse_msdp(data.borrow())?;
            Ok(Some(CnxOutput::Msdp(msdp_data)))
        }
        _ => {
            warn!("ignoring subnegotiation for {:?}", (opt, data));
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{:?}", *opt),
            ))
        }
    }
}

pub async fn handle_negotiation(
    telnet: &mut TelnetWriter<'_>,
    config: &MudConfig,
    state: &mut CnxState,
    n: &Negotiation,
) -> io::Result<Option<CnxOutput>> {
    match n {
        Negotiation::Negotiation(action, opt)
            if *action == NegotiationAction::Do || *action == NegotiationAction::Will =>
        {
            negotiate_answer(telnet, state, action, opt).await;
            Ok(None)
        }
        Negotiation::Subnegotiation(option, data) => {
            handle_sub_negotiations(telnet, config, state, option, data).await
        }
        _ => Ok(None),
    }
}

// pub struct MudNet {
//     telnet: Telnet,
//     config: MudConfig,
//     state: CnxState,
// }
//
// impl MudNet {
//     pub async fn next(&mut self) -> io::Result<Vec<CnxOutput>> {
//         next(&mut self.telnet, &self.config, &mut self.state).await
//     }
//
//     pub async fn negotiate(&mut self,
//                            state: &mut CnxState,
//                            opt: &TelnetOption) -> io::Result<()> {
//         negotiate(&mut self.telnet, state, opt).await
//     }
// }

pub async fn negotiate(
    telnet: &mut TelnetWriter<'_>,
    state: &mut CnxState,
    opt: &TelnetOption,
) -> io::Result<()> {
    let mut nego_state = state.negotiation_state(opt);
    do_negotiate(telnet, &mut nego_state).await?;
    state.add_negociated_option(nego_state);
    Ok(())
}

pub async fn negotiate_answer(
    telnet: &mut TelnetWriter<'_>,
    state: &mut CnxState,
    action: &NegotiationAction,
    opt: &TelnetOption,
) -> io::Result<()> {
    let mut nego_state = state.negotiation_state(opt);
    do_negotiate(telnet, &mut nego_state).await?;

    nego_state.received_do = nego_state.received_do || *action == NegotiationAction::Do;
    nego_state.received_will = nego_state.received_will || *action == NegotiationAction::Will;

    state.add_negociated_option(nego_state);
    Ok(())
}

async fn do_negotiate(
    telnet: &mut TelnetWriter<'_>,
    nego_state: &mut NegotiationState,
) -> io::Result<()> {
    if nego_state.shoud_negotiate() {
        debug!(
            "negotiating Do for supported option {:?}",
            nego_state.option
        );
        telnet
            .try_negotiate(NegotiationAction::Will, nego_state.option)
            .await?;
        telnet
            .try_negotiate(NegotiationAction::Do, nego_state.option)
            .await?;
        nego_state.send_will = true;
        nego_state.send_do = true;
    } else {
        warn!("negotiating Wont for unsupported {:?}", nego_state.option);
        telnet
            .try_negotiate(NegotiationAction::Wont, nego_state.option)
            .await?;
        nego_state.send_wont = true;
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum CnxOutput {
    Data(String),
    Msdp(MsdpData),
}

pub fn handler(
    mut tcp_stream: Box<tokio::net::TcpStream>,
    mut command_receiver: Receiver<String>,
    mut cnx_sender: Sender<CnxOutput>,
) -> impl Future<Output = io::Result<()>> {
    async move {
        let config = MudConfig::default();
        let mut cnx_state = CnxState::new();

        let (mut telnet, mut writer): (Telnet, TelnetWriter) =
            Telnet::from_stream(tcp_stream.as_mut(), 256);

        debug!("Connected to the server!");

        let (mut nego_sender, mut nego_receiver): (Sender<Negotiation>, Receiver<Negotiation>) =
            mpsc::channel(100);

        let mut data_sender = cnx_sender.clone();

        let user_input = async move {
            loop {
                match nego_receiver.try_recv() {
                    Ok(nego) => {
                        debug!("negotiating {:?}", nego);

                        if let Some(output) =
                            handle_negotiation(&mut writer, &config, &mut cnx_state, &nego).await?
                        {
                            cnx_sender
                                .send(output)
                                .await
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                        } else {
                            Ok(())
                        }
                    }
                    Err(TryRecvError::Empty) => {
                        //                        debug!("try receive empty !");
                        //                        Ok::<usize, io::Error>(0)
                        Ok(())
                    }
                    Err(TryRecvError::Closed) => break,
                };

                match command_receiver.try_recv() {
                    Ok(msg) => {
                        debug!("sending {:?}", msg);
                        writer.write(msg.as_bytes()).await?;
                        Ok(())
                    }
                    Err(TryRecvError::Empty) => {
                        //                      debug!("try receive empty !");
                        //Ok::<usize, io::Error>(0)
                        Ok::<(), io::Error>(())
                    }
                    Err(TryRecvError::Closed) => break,
                }?;
                task::yield_now().await;
            }
            Ok::<(), io::Error>(())
        };

        let network = async move {
            loop {
                let chunk = read_chunk(&mut telnet).await?;

                data_sender.send(CnxOutput::Data(chunk.data)).await;

                for n in chunk.negotiations.into_iter() {
                    nego_sender.send(n.clone());
                }

                task::yield_now().await;
            }
            Ok::<(), io::Error>(())
        };

        tokio::join!(user_input, network);
        Ok::<(), io::Error>(())
    }
}
