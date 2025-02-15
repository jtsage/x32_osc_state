use crate::x32::updates::{CueUpdate, SnippetUpdate, SceneUpdate, FaderUpdate, FaderUpdateParse, FaderName, FaderIdx};
use crate::enums::{Error, X32Error, ShowMode, NODE_STRING};
use crate::osc::{Type, Buffer, Message};

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
    ShowMode(ShowMode),
    /// Meters (see notes on [`crate::X32ProcessResult`])
    Meters((usize, Vec<f32>))
}

impl TryFrom<Buffer> for ConsoleMessage {
    type Error = Error;

    fn try_from(value: Buffer) -> Result<Self, Self::Error> {
        let msg:Message = value.try_into()?;
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
                    .try_into()?;
                Self::try_from_node(node_arg.as_str())
            },
            _ => Self::try_from_standard_osc(&msg)
        }
    }
}

impl ConsoleMessage {
    /// Split address on slashes, return as a tuple
    #[must_use]
    pub fn split_address(s : &str) -> (&str, &str, &str, &str) {
        let s = s.strip_prefix('/').map_or(s, |s| s);

        let mut sp = s.split('/');
        (
            sp.next().unwrap_or(""),
            sp.next().unwrap_or(""),
            sp.next().unwrap_or(""),
            sp.next().unwrap_or(""),
        )
    }

    /// Split an node message string argument into it's parts
    #[must_use]
    pub fn split_node_msg(s : &str) -> (String, Vec<String>) {
        let mut address = String::new();
        let mut args:Vec<String> = vec![];

        for (i, cap) in NODE_STRING.captures_iter(s).enumerate() {
            if let Some(v) = cap.get(1) {
                args.push(v.as_str().to_owned());
            } else if let Some(v) = cap.get(0) {
                if i == 0 {
                    v.as_str().clone_into(&mut address);
                } else {
                    args.push(v.as_str().to_owned());
                }
            }
        }
        (address, args)
    }

    /// Match a standard OSC message from the console
    #[expect(clippy::single_call_fn)]
    fn try_from_standard_osc(msg : &Message) -> Result<Self, Error> {
        let parts = Self::split_address(&msg.address);
        // let parts = (parts.0.as_str(), parts.1.as_str(), parts.2.as_str(), parts.3.as_str());

        match parts {
            (_, _, "mix", "fader") | ("dca", _, "fader", "") => {
                let fader_update = FaderUpdate::try_from(FaderUpdateParse::StdFader(
                    FaderName(parts.0.to_owned()),
                    FaderIdx(parts.1.to_owned()),
                    msg.first_default(0_f32)
                ))?;
                
                Ok(Self::Fader(fader_update))
            },

            (_, _, "mix", "on") | ("dca", _, "on", "") => {
                let fader_update = FaderUpdate::try_from(FaderUpdateParse::StdMute(
                    FaderName(parts.0.to_owned()),
                    FaderIdx(parts.1.to_owned()),
                    msg.first_default(0_i32)
                ))?;

                Ok(Self::Fader(fader_update))
            },

            (_, _, "config", "name") => {
                let fader_update = FaderUpdate::try_from(FaderUpdateParse::StdName(
                    FaderName(parts.0.to_owned()),
                    FaderIdx(parts.1.to_owned()),
                    msg.first_default(String::new())
                ))?;

                Ok(Self::Fader(fader_update))
            },

            (_, _, "config", "color") => {
                let fader_update = FaderUpdate::try_from(FaderUpdateParse::StdColor(
                    FaderName(parts.0.to_owned()),
                    FaderIdx(parts.1.to_owned()),
                    msg.first_default(1_i32)
                ))?;

                Ok(Self::Fader(fader_update))
            },

            #[expect(clippy::cast_possible_truncation)]
            ("-show", "prepos", "current", "") => 
                Ok(Self::CurrentCue(msg.first_default(-1_i32) as i16)),

            ("-prefs", "show_control", "", "") =>
                Ok(Self::ShowMode(ShowMode::from_int(msg.first_default(-1_i32)))),

            ("meters", _, "", "") => {
                parts.1.parse::<usize>().map_or(Err(Error::X32(X32Error::UnimplementedPacket)), |t| {
                    if let Some(Type::Blob(v)) = msg.args.first() {
                        let float_vec:Vec<f32> = v.chunks_exact(4)
                            .map(|f| {
                                f32::from_le_bytes([f[0], f[1], f[2], f[3]])
                            }).collect();

                        Ok(Self::Meters((t, float_vec)))
                    } else {
                        Err(Error::X32(X32Error::UnimplementedPacket))
                    }
                })
            },

            _ => Err(Error::X32(X32Error::UnimplementedPacket))
        }
    }

    

    /// Match a standard OSC message from the console
    #[expect(clippy::single_call_fn)]
    fn try_from_node(arg: &str) -> Result<Self, Error> {
        let (address, args) = Self::split_node_msg(arg);

        let arg_len = args.len();

        let parts = Self::split_address(&address);
        // let parts = (parts.0.as_str(), parts.1.as_str(), parts.2.as_str(), parts.3.as_str());

        match parts {
            (_, _, "mix", "") | ("dca", _, "", "") if arg_len >= 2 => {
                let fader_update = FaderUpdate::try_from(FaderUpdateParse::NodeMix(
                    FaderName(parts.0.to_owned()),
                    FaderIdx(parts.1.to_owned()),
                    args[0].clone(),
                    args[1].clone()
                ))?;
                
                Ok(Self::Fader(fader_update))
            },

            (_, _, "config", "") if arg_len >= 1 => {
                let fader_update = FaderUpdate::try_from(FaderUpdateParse::NodeConfig(
                    FaderName(parts.0.to_owned()),
                    FaderIdx(parts.1.to_owned()),
                    args[0].clone(),
                    args[2].clone(),
                ))?;

                Ok(Self::Fader(fader_update))
            },

            #[expect(clippy::cast_possible_truncation)]
            ("-show", "prepos", "current", "") => Ok(Self::CurrentCue(args[0]
                .parse::<i32>()
                .unwrap_or(-1_i32) as i16
            )),

            ("-prefs", "show_control", "", "") =>
                Ok(Self::ShowMode(ShowMode::from_const(args[0].as_str()))),

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

                Ok(Self::Cue(CueUpdate {
                    cue_number, scene, snippet,
                    index: parts.3.parse::<usize>().unwrap_or(0),
                    name: args[1].clone(),
                }))
            }

            ("-show", "showfile", "scene", _) => Ok(Self::Scene(SceneUpdate {
                index: parts.3.parse::<usize>().unwrap_or(0),
                name: args[0].clone(),
            })),

            ("-show", "showfile", "snippet", _) => Ok(Self::Snippet(SnippetUpdate {
                index: parts.3.parse::<usize>().unwrap_or(0),
                name: args[0].clone(),
            })),

            _ => Err(Error::X32(X32Error::UnimplementedPacket))
        }
    }
}



