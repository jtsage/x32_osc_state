use super::super::enums::{Error, FaderIndex, Fader, FaderColor, FaderIndexParse};


/// CUE record
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
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
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct SnippetUpdate {
    /// index
    pub index : usize,
    /// display name
    pub name : String,
}

/// Scene record
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct SceneUpdate {
    /// index
    pub index : usize,
    /// display name
    pub name : String,
}

/// Fader update processed
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct FaderUpdate {
    /// Type of fader
    pub source : FaderIndex,
    /// scribble strip label
    pub label : Option<String>,
    /// level of fader, as number
    pub level : Option<f32>,
    /// mute status, as bool
    pub is_on : Option<bool>,
    /// color
    pub color : Option<FaderColor>
}

impl Default for FaderUpdate {
    fn default() -> Self { Self {
        source : FaderIndex::Unknown,
        label : None,
        level : None,
        is_on : None,
        color : None
    } }
}


/// Fader bank name
pub struct FaderName(pub String);
/// Fader index (1-based)
pub struct FaderIdx(pub String);

/// Fader update parsing
/// - first element is always the fader bank
/// - second element is always the index (1-based)
pub enum FaderUpdateParse {
    /// node Mix message - [ON/OFF], level (str)
    NodeMix(FaderName, FaderIdx, String, String),
    /// node config - name, color (str)
    NodeConfig(FaderName, FaderIdx, String, String),
    /// /fader - level
    StdFader(FaderName, FaderIdx, f32),
    /// /fader/on - i32
    StdMute(FaderName, FaderIdx, i32),
    /// /fader/name - name
    StdName(FaderName, FaderIdx, String),
    /// /fader/config/color - color (i32)
    StdColor(FaderName, FaderIdx, i32),
}

impl TryFrom<FaderUpdateParse> for FaderUpdate {
    type Error = Error;

    fn try_from(value: FaderUpdateParse) -> Result<Self, Self::Error> {
        let source = match &value {
            FaderUpdateParse::NodeMix(b, i, _, _) |
            FaderUpdateParse::NodeConfig(b, i, _, _) |
            FaderUpdateParse::StdFader(b, i, _) |
            FaderUpdateParse::StdMute(b, i, _) |
            FaderUpdateParse::StdName(b, i, _) |
            FaderUpdateParse::StdColor(b, i, _) =>
                FaderIndex::try_from(FaderIndexParse::String(b.0.clone(), i.0.clone()))?,
        };

        let is_on = match &value {
            FaderUpdateParse::NodeMix(_, _, t, _) => Some(Fader::is_on_from_string(t)),
            FaderUpdateParse::StdMute(_, _, i) => Some(*i == 1),
            _ => None
        };

        let level = match &value {
            FaderUpdateParse::NodeMix(_, _, _, t) => Some(Fader::level_from_string(t)),
            FaderUpdateParse::StdFader(_, _, f) => Some(*f),
            _ => None
        };

        let label = match &value {
            FaderUpdateParse::NodeConfig(_, _, t, _) |
            FaderUpdateParse::StdName(_, _, t) => Some(t.clone()),
            _ => None
        };

        let color = match &value {
            FaderUpdateParse::NodeConfig(_, _, _, t) => Some(FaderColor::parse_str(t)),
            FaderUpdateParse::StdColor(_, _, i) => Some(FaderColor::parse_int(*i)),
            _ => None
        };


        Ok(Self { source, label, level, is_on, color })
    }
}
