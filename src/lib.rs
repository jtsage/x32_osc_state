#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Enums and static data
pub mod enums;
/// Low-level OSC message handling
pub mod osc;
/// X32 Types and OSC Reflections
pub mod x32;

/// [`X32Console::process`] results
/// 
/// Note that a lot of understood messages still return [`X32ProcessResult::NoOperation`],
/// particularly cue type messages
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum X32ProcessResult {
    /// No operation should be taken
    NoOperation,
    /// A fader was changed
    Fader(enums::Fader),
    /// The current cue was changed
    CurrentCue(String),
    /// Meter info
    /// the first item of the tuple is the meter message index.
    /// note that the first element in the Vec is nonsense - it *should*
    /// be an integer equal to the size of the vector, but that would
    /// complicate working with the data - it is left intact so that
    /// the vector indexes line up better with the data.
    Meters((usize, Vec<f32>))
}

// MARK: X32State
/// X32 State
#[derive(Debug, Clone)]
pub struct X32Console {
    /// Faders
    pub faders : enums::FaderBank,

    /// Full Cue List
    pub cues : [Option<enums::ShowCue>; 500],
    /// Full Snippet List
    pub snippets : [Option<String>; 100],
    /// Full Scene List
    pub scenes : [Option<String>; 100],

    /// Board tracking method
    pub show_mode : enums::ShowMode,
    /// Current Cue
    pub current_cue : Option<usize>,
}

impl X32Console {
    /// create new X32 state machine
    #[must_use]
    pub fn new() -> Self {
        Self {
            faders: enums::FaderBank::default(),
            cues: [(); 500].map(|()| None),
            snippets: [(); 100].map(|()| None),
            scenes: [(); 100].map(|()| None),
            show_mode: enums::ShowMode::Cues,
            current_cue: None,
        }
    }

    // MARK: ~fader
    /// Get a fader, 1 based index
    #[must_use]
    pub fn fader(&self, f_type:&enums::FaderIndex) -> Option<enums::Fader> {
        self.faders.get(f_type)
    }

    // MARK: ~active_cue
    /// Get active cue, scene, or snippet
    #[must_use]
    pub fn active_cue(&self) -> String {
        match self.show_mode {
            enums::ShowMode::Cues => format!("Cue: {}" ,self.cue_name(self.current_cue)),
            enums::ShowMode::Scenes => format!("Scene: {}" ,self.scene_name(self.current_cue)),
            enums::ShowMode::Snippets => format!("Snippet: {}" ,self.snip_name(self.current_cue)),
        }
    }

    // MARK: ~cue_list_size
    /// Count cues
    #[must_use]
    pub fn cue_list_size(&self) -> (usize, usize, usize) {
        (
            self.cues.iter().filter(|v| v.is_some()).count(),
            self.scenes.iter().filter(|v| v.is_some()).count(),
            self.snippets.iter().filter(|v| v.is_some()).count(),
        )
    }

    // MARK: ~reset
    /// Reset the state machine
    pub fn reset(&mut self) {
        self.clear_cues();
        self.faders.reset();
    }

    /// Clear cue list.
    pub fn clear_cues(&mut self) {
        self.cues = [(); 500].map(|()| None);
        self.snippets = [(); 100].map(|()| None);
        self.scenes = [(); 100].map(|()| None);
    }

    // MARK: ~cue_name
    /// get formatted cue name from index (includes scene and snippet)
    fn cue_name(&self, index: Option<usize> ) -> String {
        let default = String::from("0.0.0 :: -- [--] [--]");

        match index {
            Some(d) if d < 500 => {
                self.cues[d].as_ref().map_or(default, |t| format!("{} :: {} [{}] [{}]",
                    t.cue_number,
                    t.name,
                    self.scene_name(t.scene),
                    self.snip_name(t.snippet)
                ))
            },
            _ => default
        }
    }

    /// get scene name from index
    fn scene_name(&self, index: Option<usize> ) -> String {
        let default = String::from("--");

        match index {
            Some(d) if d < 100 =>
                self.scenes[d].as_ref().map_or(default, |t| format!("{d:02}:{t}")),
            _ => default
        }
    }

    /// get snippet name from index
    fn snip_name(&self, index: Option<usize> ) -> String {
        let default = String::from("--");

        match index {
            Some(d) if d < 100 =>
                self.snippets[d].as_ref().map_or(default, |t| format!("{d:02}:{t}")),
            _ => default
        }
    }

    // MARK: ~process
    /// Process OSC data from the X32
    /// 
    /// This takes a well formed [`osc::Buffer`] or [`osc::Message`]
    /// 
    /// Returns [`X32ProcessResult`]
    pub fn process<T: TryInto<x32::ConsoleMessage>>(&mut self, v : T) -> X32ProcessResult {
        v.try_into().map_or(X32ProcessResult::NoOperation, |v| self.update(v))
    }

    /// Update the state machine from processed OSC data
    pub fn update(&mut self, update :x32::ConsoleMessage ) -> X32ProcessResult {
        match update {
            x32::ConsoleMessage::Meters(v) => X32ProcessResult::Meters(v),
            x32::ConsoleMessage::Fader(update) => self.faders.update(update),

            #[expect(clippy::cast_sign_loss)]
            x32::ConsoleMessage::CurrentCue(v) => {
                self.current_cue = if v < 0 { None } else { Some(v as usize) };
                X32ProcessResult::CurrentCue(self.active_cue())
            },

            x32::ConsoleMessage::ShowMode(v) => {
                self.show_mode = v;
                X32ProcessResult::CurrentCue(self.active_cue())
            },
    
            x32::ConsoleMessage::Cue(v) => {
                if v.index <= 500 {
                    self.cues[v.index] = Some(enums::ShowCue{
                        cue_number: v.cue_number,
                        name: v.name,
                        snippet: v.snippet,
                        scene: v.scene,
                    });
                }
                X32ProcessResult::NoOperation
            },

            x32::ConsoleMessage::Snippet(v) => {
                if v.index <= 500 {
                    self.snippets[v.index] = Some(v.name.clone());
                }
                X32ProcessResult::NoOperation
            },

            x32::ConsoleMessage::Scene(v) => {
                if v.index <= 500 {
                    self.scenes[v.index] = Some(v.name.clone());
                }
                X32ProcessResult::NoOperation
            },
        }
    }
}

impl Default for X32Console {
    fn default() -> Self { Self::new() }
}

