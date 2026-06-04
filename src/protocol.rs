use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Copy, Deserialize, Serialize)]
pub enum DeviceMode {
    Tio,
    Tf1,
}

#[derive(Clone)]
pub struct Config {
    pub host: Option<String>,
    pub mode: DeviceMode,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Command {
    Gain    { ch: u32, value: f64 },
    Mute    { ch: u32, on: bool },
    Phantom { ch: u32, on: bool },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConfigCommand {
    SetDeviceMode { mode: DeviceMode },
    SetHost { host: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "target", rename_all = "camelCase")]
pub enum GetCommand {
    Config,
    Gain { ch: u32 },
    Scan,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum StateMessage {
    ConfigState { host: Option<String>, mode: DeviceMode },
    GainState { ch: u32, value: f64 },
    DeviceList { devices: Vec<crate::discovery::DiscoveredDevice> },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Message {
    Command(Command),
    Config(ConfigCommand),
    Get(GetCommand),
    State(StateMessage),
}


pub fn build_rpc_command(cmd: &Command, device_mode: DeviceMode) -> String {
    match device_mode {
            DeviceMode::Tio => match cmd {
                Command::Gain { ch, value } => {
                    format!("set IO:Current/InCh/HAGain {} 0 {}\n", ch, value)
                }
                Command::Mute { ch: _, on: _ } => {
                    String::new()
                }
                Command::Phantom { ch, on } => {
                    format!("set IO:Current/InCh/48VOn {} 0 {}\n", ch, if *on { 1 } else { 0 })
                }
            }

            DeviceMode::Tf1 => match cmd {
                Command::Gain { ch, value } => {
                    format!("set MIXER:Current/InCh/Fader/Level {} 0 {}\n", ch, value)
                }
                Command::Mute { ch, on } => {
                    format!("set MIXER:Current/InCh/Fader/On {} 0 {}\n", ch, if *on { 0 } else { 1 })
                }
                Command::Phantom { ch, on } => {
                    format!("set MIXER:Current/InCh/HA/48V {} 0 {}\n", ch, if *on { 1 } else { 0 })
                }
            }
    }
}

pub fn build_rpc_get(cmd: &GetCommand, device_mode: DeviceMode) -> Option<String> {
    match device_mode {
        DeviceMode::Tio => match cmd {
            GetCommand::Gain { ch } => Some(format!("get IO:Current/InCh/HAGain {} 0\n", ch)),
            _ => None,
        },
        DeviceMode::Tf1 => match cmd {
            GetCommand::Gain { ch } => Some(format!("get MIXER:Current/InCh/Fader/Level {} 0\n", ch)),
            _ => None,
        },
    }
}

pub fn parse_yamaha_response(line: &str) -> Option<StateMessage> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 5 || (parts[0] != "OK" && parts[0] != "NOTIFY") {
        return None;
    }

    let path = parts[1];
    let ch_str = parts[2];
    let val_str = parts[4];

    let Ok(ch) = ch_str.parse::<u32>() else { return None; };

    match path {
        "MIXER:Current/InCh/Fader/Level" => {
            if let Ok(val) = val_str.parse::<f64>() {
                return Some(StateMessage::GainState { ch, value: val });
            }
        }
        "IO:Current/InCh/HAGain" => {
            if let Ok(val) = val_str.parse::<f64>() {
                return Some(StateMessage::GainState { ch, value: val });
            }
        }
        _ => {
        }
    }
    
    None
}