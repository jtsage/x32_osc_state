use super::x32::{ConsoleMessage, FaderType, ShowMode};

/// Fader state
pub mod faders;
use faders::{Fader, FaderBank};
pub use faders::Fader as X32Fader;

/// Cue record
#[derive(Debug, Clone)]
pub struct ShowCue {
    /// Displayed cue number
    pub cue_number : String,
    /// Cue name
    pub name : String,
    /// associated snippet (or None)
    pub snippet : Option<usize>,
    /// associated scene (or None)
    pub scene : Option<usize>,
}

/// X32 State
#[derive(Debug, Clone)]
pub struct X32Console {
    /// Faders
    pub faders : faders::FaderBank,

    /// Full Cue List
    pub cues : [Option<ShowCue>; 500],
    /// Full Snippet List
    pub snippets : [Option<String>; 100],
    /// Full Scene List
    pub scenes : [Option<String>; 100],

    /// Board tracking method
    pub show_mode : ShowMode,
    /// Current Cue
    pub current_cue : Option<usize>,
}

impl X32Console {
    /// create new X32 state machine
    #[must_use]
    pub fn new() -> Self {
        X32Console {
            faders: FaderBank::default(),
            cues: [(); 500].map(|()| None),
            snippets: [(); 100].map(|()| None),
            scenes: [(); 100].map(|()| None),
            show_mode: ShowMode::Cues,
            current_cue: None,
        }
    }

    /// Get a fader, 1 based index
    #[must_use]
    pub fn fader(&self, f_type:&FaderType, index: usize) -> Option<Fader> {
        self.faders.get(f_type, index-1)
    }

    /// Get active cue, scene, or snippet
    #[must_use]
    pub fn active_cue(&self) -> String {
        match self.show_mode {
            ShowMode::Cues => format!("Cue: {}" ,self.cue_name(self.current_cue)),
            ShowMode::Scenes => format!("Scene: {}" ,self.scene_name(self.current_cue)),
            ShowMode::Snippets => format!("Snippet: {}" ,self.snip_name(self.current_cue)),
        }
    }

    /// Count cues
    #[must_use]
    pub fn cue_list_size(&self) -> (usize, usize, usize) {
        (
            self.cues.iter().filter(|v| v.is_some()).count(),
            self.scenes.iter().filter(|v| v.is_some()).count(),
            self.snippets.iter().filter(|v| v.is_some()).count(),
        )
    }

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

    /// get formatted cue name from index (includes scene and snippet)
    fn cue_name(&self, index: Option<usize> ) -> String {
        let default = String::from("0.0.0 :: -- [--] [--]");

        match index {
            Some(d) if d < 500 => {
                match &self.cues[d] {
                    Some(t) => format!("{} :: {} [{}] [{}]",
                        t.cue_number,
                        t.name,
                        self.scene_name(t.scene),
                        self.snip_name(t.snippet)
                    ),
                    None => default
                }
            },
            _ => default
        }
    }

    /// get scene name from index
    fn scene_name(&self, index: Option<usize> ) -> String {
        let default = String::from("--");

        match index {
            Some(d) if d < 100 => {
                match &self.scenes[d] {
                    Some(t) => format!("{d:02}:{t}"),
                    None => default
                }
            },
            _ => default
        }
    }

    /// get snippet name from index
    fn snip_name(&self, index: Option<usize> ) -> String {
        let default = String::from("--");

        match index {
            Some(d) if d < 100 => {
                match &self.snippets[d] {
                    Some(t) => format!("{d:02}:{t}"),
                    None => default
                }
            },
            _ => default
        }
    }

    /// Process a `Buffer` or `Message` from x32 OSC data
    pub fn process<T: TryInto<ConsoleMessage>>(&mut self, v : T) {
        if let Ok(v) = v.try_into() {
            self.update(v);
        }
    }

    /// Update the state machine from processed OSC data
    pub fn update(&mut self, update :ConsoleMessage ) {
        match update {
            ConsoleMessage::Fader(update) => self.faders.update(update),
            #[expect(clippy::cast_sign_loss)]
            ConsoleMessage::CurrentCue(v) => self.current_cue = if v < 0 { None } else { Some(v as usize) },
            ConsoleMessage::ShowMode(v) => self.show_mode = v,
            ConsoleMessage::Cue(v) => {
                if v.index <= 500 {
                    self.cues[v.index] = Some(ShowCue{
                        cue_number: v.cue_number,
                        name: v.name,
                        snippet: v.snippet,
                        scene: v.scene,
                    });
                }
            },
            ConsoleMessage::Snippet(v) => {
                if v.index <= 500 {
                    self.snippets[v.index] = Some(v.name.clone());
                }
            },
            ConsoleMessage::Scene(v) => {
                if v.index <= 500 {
                    self.scenes[v.index] = Some(v.name.clone());
                }
            },
        }
    }
}

impl Default for X32Console {
    fn default() -> Self { Self::new() }
}
