#[derive(clap::ValueEnum, Clone)]
pub enum DeviceMode {
    Tio,
    Tf1,
}

pub enum TioCommand {
    Gain { ch: u32, value: i32 },
    Phantom { ch: u32, on: bool },
}

pub enum MixerCommand {
    FaderLevel { ch: u32, value: i32 },
    Mute { ch: u32, on: bool },
    Phantom { ch: u32, on: bool },
}

pub fn build_tio_command(cmd: &TioCommand) -> String {
    match cmd {
        TioCommand::Gain { ch, value } => {
            format!("set IO:Current/InCh/HAGain {} 0 {}\n", ch, value)
        }
        TioCommand::Phantom { ch, on } => {
            format!("set IO:Current/InCh/48VOn {} 0 {}\n", ch, if *on { 1 } else { 0 })
        }
    }
}

pub fn build_mixer_command(cmd: &MixerCommand) -> String {
    match cmd {
        MixerCommand::FaderLevel { ch, value } => {
            format!("set MIXER:Current/InCh/Fader/Level {} 0 {}\n", ch, value)
        }
        MixerCommand::Mute { ch, on } => {
            format!("set MIXER:Current/InCh/Fader/On {} 0 {}\n", ch, if *on { 0 } else { 1 })
        }
        MixerCommand::Phantom { ch, on } => {
            format!("set MIXER:Current/InCh/HA/48V {} 0 {}\n", ch, if *on { 1 } else { 0 })
        }
    }
}