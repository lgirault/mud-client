// Mud Options
pub mod options {
    pub const MSLP: u8 = 68;
    pub const MSDP: u8 = 69;
    pub const MSSP: u8 = 70;
    pub const MCCP1: u8 = 85;
    pub const MCCP2: u8 = 86;
    pub const MCCP3: u8 = 87;
    pub const MSP: u8 = 90;
    pub const MXP: u8 = 91;
    //https://wiki.mudlet.org/w/Manual:Supported_Protocols#Aardwolf.E2.80.99s_102_subchannel
    pub const AARDWOLF102: u8 = 102;
    pub const ATCP: u8 = 200;
    pub const GMCP: u8 = 201;

    pub mod names {
        pub const MSLP: &'static str = "MSLP";
        pub const MSDP: &'static str = "MSDP";
        pub const MSSP: &'static str = "MSSP";
        pub const MCCP1: &'static str = "MCCP1";
        pub const MCCP2: &'static str = "MCCP2";
        pub const MCCP3: &'static str = "MCCP3";
        pub const MSP: &'static str = "MSP";
        pub const MXP: &'static str = "MXP";
        pub const AARDWOLF102: &'static str = "AARDWOLF's 102 channel";
        pub const ATCP: &'static str = "ATCP";
        pub const GMCP: &'static str = "GMCP";
    }
}


pub fn str_of_mud_option(b: u8) -> Result<&'static str, std::io::Error> {
    match b {
        options::MSLP => Ok(options::names::MSLP),
        options::MSDP => Ok(options::names::MSDP),
        options::MSSP => Ok(options::names::MSSP),
        options::MCCP1 => Ok(options::names::MCCP1),
        options::MCCP2 => Ok(options::names::MCCP2),
        options::MCCP3 => Ok(options::names::MCCP3),
        options::MSP => Ok(options::names::MSP),
        options::MXP => Ok(options::names::MXP),
        options::AARDWOLF102 => Ok(options::names::AARDWOLF102),
        options::ATCP => Ok(options::names::ATCP),
        options::GMCP => Ok(options::names::GMCP),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("unknown mud option {}", b),
        )),
    }
}