use super::{Error, FaderUpdate, util, ShowMode};
use crate::osc::{Buffer, Message};

/// CUE record
#[derive(Debug, PartialEq, PartialOrd)]
pub struct CueUpdate {
    /// index in list
    pub index : usize,
    /// Displayed cue number
    pub cue_number : String,
    /// Cue name
    pub name : String,
    /// associated snippet (or None)
    pub snippet : Option<usize>,
    /// associated scene (or None)
    pub scene : Option<usize>,
}

/// Snippet record
#[derive(Debug, PartialEq, PartialOrd)]
pub struct SnippetUpdate {
    /// index
    pub index : usize,
    /// display name
    pub name : String,
}

/// Scene record
#[derive(Debug, PartialEq, PartialOrd)]
pub struct SceneUpdate {
    /// index
    pub index : usize,
    /// display name
    pub name : String,
}

#[derive(Debug, PartialEq, PartialOrd)]
/// Messages received from the X32 console
pub enum ConsoleMessage {
    /// Fader updates
    Fader(FaderUpdate),
    /// Cue listing
    Cue(CueUpdate),
    /// Snippet listing
    Snippet(SnippetUpdate),
    /// Scene listing
    Scene(SceneUpdate),
    /// Current cue index
    CurrentCue(i16),
    /// Current control mode (Cues, Scenes or Snippets)
    ShowMode(ShowMode)
}

impl TryFrom<Buffer> for ConsoleMessage {
    type Error = Error;

    fn try_from(value: Buffer) -> Result<Self, Self::Error> {
        let msg:Message = value.try_into().map_err(|_| Error::MalformedPacket )?;
        msg.try_into()
    }
}

impl TryFrom<Message> for ConsoleMessage {
    type Error = Error;

    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        match msg.address.as_str() {
            "node" => {
                let node_arg:String = msg.args
                    .first()
                    .unwrap_or_default()
                    .clone()
                    .try_into()
                    .map_err(|_| Error::MalformedPacket)?;

                match_incoming_node(node_arg.as_str())
            },
            _ => match_incoming_standard(&msg)
        }
    }
}

/// Match a standard OSC message from the console
fn match_incoming_standard(msg : &Message) -> Result<ConsoleMessage, Error> {
    let parts = util::split_address(&msg.address);
    let parts = (parts.0.as_str(), parts.1.as_str(), parts.2.as_str(), parts.3.as_str());

    match parts {
        (_, _, "mix", "fader") | ("dca", _, "fader", "") => {
            let fader_update:FaderUpdate = (
                parts.0.to_owned(),
                parts.1.to_owned(),
                msg.args
                    .first()
                    .unwrap_or_default()
                    .clone()
                    .try_into()
                    .unwrap_or(0_f32)
            ).try_into()?;
            
            Ok(ConsoleMessage::Fader(fader_update))
        },

        (_, _, "mix", "on") | ("dca", _, "on", "") => {
            let fader_update:FaderUpdate = (
                parts.0.to_owned(),
                parts.1.to_owned(),
                msg.args
                    .first()
                    .unwrap_or_default()
                    .clone()
                    .try_into()
                    .unwrap_or(0_i32)
            ).try_into()?;

            Ok(ConsoleMessage::Fader(fader_update))
        },

        (_, _, "config", "name") => {
            let fader_update:FaderUpdate = (
                parts.0.to_owned(),
                parts.1.to_owned(),
                msg.args
                    .first()
                    .unwrap_or_default()
                    .clone()
                    .try_into()
                    .unwrap_or(String::new())
            ).try_into()?;

            Ok(ConsoleMessage::Fader(fader_update))
        },

        #[expect(clippy::cast_possible_truncation)]
        ("-show", "prepos", "current", "") => Ok(ConsoleMessage::CurrentCue(msg.args
            .first()
            .unwrap_or_default()
            .clone()
            .try_into()
            .unwrap_or(-1_i32) as i16
        )),

        ("-prefs", "show_control", "", "") => {
            let show_mode_int = msg.args
                .first()
                .unwrap_or_default()
                .clone()
                .try_into()
                .unwrap_or(-1_i32);

            Ok(ConsoleMessage::ShowMode(match show_mode_int {
                1 => ShowMode::Scenes,
                2 => ShowMode::Snippets,
                _ => ShowMode::Cues
            }))
        },
        _ => Err(Error::UnimplementedPacket)
    }
}



/// Match a standard OSC message from the console
fn match_incoming_node(arg: &str) -> Result<ConsoleMessage, Error> {
    let (address, args) = util::split_node_msg(arg);

    let arg_len = args.len();

    let parts = util::split_address(&address);
    let parts = (parts.0.as_str(), parts.1.as_str(), parts.2.as_str(), parts.3.as_str());

    match parts {
        (_, _, "mix", "") if arg_len >= 2 => {
            let fader_update:FaderUpdate = (
                parts.0.to_owned(),
                parts.1.to_owned(),
                args[0].clone(),
                args[1].clone()
            ).try_into()?;
            
            Ok(ConsoleMessage::Fader(fader_update))
        },

        (_, _, "config", "") if arg_len >= 1 => {
            let fader_update:FaderUpdate = (
                parts.0.to_owned(),
                parts.1.to_owned(),
                args[0].clone()
            ).try_into()?;

            Ok(ConsoleMessage::Fader(fader_update))
        },

        #[expect(clippy::cast_possible_truncation)]
        ("-show", "prepos", "current", "") => Ok(ConsoleMessage::CurrentCue(args[0]
            .parse::<i32>()
            .unwrap_or(-1_i32) as i16
        )),

        ("-prefs", "show_control", "", "") => {
            Ok(ConsoleMessage::ShowMode(match args[0].as_str() {
                "SCENES" => ShowMode::Scenes,
                "SNIPPETS" => ShowMode::Snippets,
                _ => ShowMode::Cues
            }))
        },

        ("-show", "showfile", "cue", _) => {
            let mut cue_number = args[0].clone();
            cue_number.insert(cue_number.len()-2, '.');
            cue_number.insert(cue_number.len()-1, '.');

            #[expect(clippy::cast_sign_loss)]
            let scene = match args[3].parse::<i32>() {
                Ok(d) if d >= 0 => Some(d as usize),
                _ => None
            };

            #[expect(clippy::cast_sign_loss)]
            let snippet = match args[4].parse::<i32>() {
                Ok(d) if d >= 0 => Some(d as usize),
                _ => None,
            };

            Ok(ConsoleMessage::Cue(CueUpdate {
                cue_number, scene, snippet,
                index: parts.3.parse::<usize>().unwrap_or(0),
                name: args[1].clone(),
            }))
        }

        ("-show", "showfile", "scene", _) => Ok(ConsoleMessage::Scene(SceneUpdate {
            index: parts.3.parse::<usize>().unwrap_or(0),
            name: args[0].clone(),
        })),

        ("-show", "showfile", "snippet", _) => Ok(ConsoleMessage::Snippet(SnippetUpdate {
            index: parts.3.parse::<usize>().unwrap_or(0),
            name: args[0].clone(),
        })),

        _ => Err(Error::UnimplementedPacket)
    }
}
