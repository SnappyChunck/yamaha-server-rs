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
    // fader commands
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
#[serde(tag = "type")]
pub enum Message {
    Command(Command),
    Config(ConfigCommand),
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