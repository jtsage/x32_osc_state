use crate::x32::{FaderType, FaderUpdate};
use crate::x32::util;

/// Full tracked fader banks
#[derive(Debug, Clone)]
pub struct FaderBank {
    /// main and mono
    main : [Fader;2],
    /// matrix (6)
    matrix : [Fader;6],
    /// aux in (8)
    aux : [Fader;8],
    /// DCA (8)
    dca : [Fader;8],
    /// mix bus (16)
    bus : [Fader;16],
    /// channels (32)
    channel : [Fader;32],
}

impl FaderBank {
    /// create new fader bank
    pub fn new() -> Self {
        FaderBank {
            main    : core::array::from_fn(|i| Fader::new(i, FaderType::Main)),
            matrix  : core::array::from_fn(|i| Fader::new(i, FaderType::Matrix)),
            bus     : core::array::from_fn(|i| Fader::new(i, FaderType::Bus)),
            channel : core::array::from_fn(|i| Fader::new(i, FaderType::Channel)),
            aux     : core::array::from_fn(|i| Fader::new(i, FaderType::Aux)),
            dca     : core::array::from_fn(|i| Fader::new(i, FaderType::Dca)),
        }
    }

    /// Update a fader
    pub fn update(&mut self, update : FaderUpdate) {
        if let Some(fader) = self.get_mut(&update.source, update.index) {
            fader.update(update);
        }
    }

    /// Get a mutable fader, zero based index
    pub fn get_mut(&mut self, f_type: &FaderType, index : usize) -> Option<&mut Fader> {
        match f_type {
            FaderType::Aux => self.aux.get_mut(index),
            FaderType::Matrix => self.matrix.get_mut(index),
            FaderType::Main => self.main.get_mut(index),
            FaderType::Channel => self.channel.get_mut(index),
            FaderType::Dca => self.dca.get_mut(index),
            FaderType::Bus => self.bus.get_mut(index),
            FaderType::Unknown => None,
        }
    }

    /// Get a fader, zero based index
    pub fn get(&self, f_type: &FaderType, index : usize) -> Option<&Fader> {
        match f_type {
            FaderType::Aux => self.aux.get(index),
            FaderType::Matrix => self.matrix.get(index),
            FaderType::Main => self.main.get(index),
            FaderType::Channel => self.channel.get(index),
            FaderType::Dca => self.dca.get(index),
            FaderType::Bus => self.bus.get(index),
            FaderType::Unknown => None,
        }
    }
}

impl Default for FaderBank {
    fn default() -> Self { Self::new() }
}

/// Named fader for console
#[derive(Debug, Clone)]
pub struct Fader {
    /// zero based index of fader
    index : usize,
    /// type of fader
    f_type : FaderType,
    /// scribble strip label
    label : String,
    /// level of fader, as number
    level_state : f32,
    /// mute status, as bool
    is_on_state : bool
}

impl Fader {
    /// create new fader
    #[must_use]
    pub fn new(index:usize, f_type:FaderType) -> Self {
        Fader {
            index, f_type,
            label : String::new(),
            level_state : 0_f32,
            is_on_state : false
        }
    }

    /// get fader label or default name
    #[must_use]
    pub fn name(&self) -> String {
        if self.label.is_empty() {
            format!("{}{:02}", self.f_type, self.index+1)
        } else {
            self.label.clone()
        }
    }

    /// get fader level
    #[must_use]
    pub fn level(&self) -> (f32, String) {
        ( self.level_state, util::level_to_string(self.level_state) )
    }

    /// get fader mute status
    #[must_use]
    pub fn is_on(&self) -> (bool, String) {
        ( self.is_on_state, String::from(if self.is_on_state { "ON" } else { "OFF" }) )
    }

    /// update fader from OSC data
    pub fn update(&mut self, update : FaderUpdate) {
        if let Some(new_level) = update.level {
            self.level_state = new_level;
        }

        if let Some(new_is_on) = update.is_on {
            self.is_on_state = new_is_on;
        }

        if let Some(new_label) = update.label {
            self.label = new_label;
        }
    }
}