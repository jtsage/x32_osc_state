#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(clippy::allow_attributes)]
#![warn(clippy::default_trait_access)]
#![warn(clippy::derive_partial_eq_without_eq)]
#![warn(clippy::equatable_if_let)]
#![warn(clippy::from_iter_instead_of_collect)]
#![warn(clippy::if_not_else)]
#![warn(clippy::if_then_some_else_none)]
#![warn(clippy::implicit_clone)]
#![warn(clippy::inefficient_to_string)]
#![warn(clippy::manual_is_variant_and)]
#![warn(clippy::manual_let_else)]
#![warn(clippy::manual_ok_or)]
#![warn(clippy::map_unwrap_or)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::needless_collect)]
#![warn(clippy::needless_pass_by_ref_mut)]
#![warn(clippy::option_if_let_else)]
#![warn(clippy::or_fun_call)]
#![warn(clippy::partial_pub_fields)]
// #![warn(clippy::pub_use)]
#![warn(clippy::redundant_type_annotations)]
#![warn(clippy::renamed_function_params)]
#![warn(clippy::return_self_not_must_use)]
#![warn(clippy::single_call_fn)]
#![warn(clippy::str_to_string)]
#![warn(clippy::string_to_string)]
#![warn(clippy::suspicious_operation_groupings)]
#![warn(clippy::unseparated_literal_suffix)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::use_self)]
// #![warn(clippy::non_std_lazy_statics)]

/// Enums and static data
pub mod enums;
/// Low-level OSC message handling
pub mod osc;
/// X32 Types and OSC Reflections
pub mod x32;



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

    /// Get a fader, 1 based index
    #[must_use]
    pub fn fader(&self, f_type:&enums::FaderIndex) -> Option<enums::Fader> {
        self.faders.get(f_type)
    }

    /// Get active cue, scene, or snippet
    #[must_use]
    pub fn active_cue(&self) -> String {
        match self.show_mode {
            enums::ShowMode::Cues => format!("Cue: {}" ,self.cue_name(self.current_cue)),
            enums::ShowMode::Scenes => format!("Scene: {}" ,self.scene_name(self.current_cue)),
            enums::ShowMode::Snippets => format!("Snippet: {}" ,self.snip_name(self.current_cue)),
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

    /// Process a `Buffer` or `Message` from x32 OSC data
    pub fn process<T: TryInto<x32::ConsoleMessage>>(&mut self, v : T) {
        if let Ok(v) = v.try_into() {
            self.update(v);
        }
    }

    /// Update the state machine from processed OSC data
    pub fn update(&mut self, update :x32::ConsoleMessage ) {
        match update {
            x32::ConsoleMessage::Fader(update) => self.faders.update(update),
            #[expect(clippy::cast_sign_loss)]
            x32::ConsoleMessage::CurrentCue(v) => self.current_cue = if v < 0 { None } else { Some(v as usize) },
            x32::ConsoleMessage::ShowMode(v) => self.show_mode = v,
            x32::ConsoleMessage::Cue(v) => {
                if v.index <= 500 {
                    self.cues[v.index] = Some(enums::ShowCue{
                        cue_number: v.cue_number,
                        name: v.name,
                        snippet: v.snippet,
                        scene: v.scene,
                    });
                }
            },
            x32::ConsoleMessage::Snippet(v) => {
                if v.index <= 500 {
                    self.snippets[v.index] = Some(v.name.clone());
                }
            },
            x32::ConsoleMessage::Scene(v) => {
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

