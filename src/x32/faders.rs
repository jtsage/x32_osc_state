use super::super::enums::{Error, FaderIndex, Fader};




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
    pub is_on : Option<bool>
}

impl Default for FaderUpdate {
    fn default() -> Self { FaderUpdate {
        source : FaderIndex::Unknown,
        label : None,
        level : None,
        is_on : None
    } }
}

/// mix message from node s~... osc message
type NodeMixMessage = (String, String, String, String);
/// fader level from /fader/... message
type StdFaderMessage = (String, String, f32);
/// mute from /fader/... message
type StdMuteMessage = (String, String, i32);
/// name from /fader/ or node s~ message
type NameMessage = (String, String, String);

impl TryFrom<StdMuteMessage> for FaderUpdate {
    type Error = Error;

    fn try_from(v: StdMuteMessage) -> Result<Self, Self::Error> {
        let source = FaderIndex::try_from((v.0, v.1))?;

        Ok(Self {
            source,
            is_on : Some(v.2 == 1),
            ..Default::default()
        })
    }
}

impl TryFrom<StdFaderMessage> for FaderUpdate {
    type Error = Error;

    fn try_from(v: StdFaderMessage) -> Result<Self, Self::Error> {
        let source = FaderIndex::try_from((v.0, v.1))?;
        
        Ok(Self {
            source,
            level : Some(v.2),
            ..Default::default()
        })
    }
}

impl TryFrom<NodeMixMessage> for FaderUpdate {
    type Error = Error;

    fn try_from(v: NodeMixMessage) -> Result<Self, Self::Error> {
        let source = FaderIndex::try_from((v.0, v.1))?;

        Ok(Self {
            source,
            level : Some(Fader::level_from_string(&v.3)),
            is_on : Some(Fader::is_on_from_string(&v.2)),
            ..Default::default()
        })
    }
}

impl TryFrom<NameMessage> for FaderUpdate {
    type Error = Error;

    fn try_from(v: NameMessage) -> Result<Self, Self::Error> {
        let source = FaderIndex::try_from((v.0, v.1))?;

        Ok(Self {
            source,
            label : Some(v.2.clone()),
            ..Default::default()
        })
    }
}
