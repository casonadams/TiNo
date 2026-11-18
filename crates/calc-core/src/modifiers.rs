use crate::keymap::Layer;

/// ALPHA's state: off, for the next key, or locked (2nd ALPHA) until ALPHA
/// is pressed again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlphaState {
    #[default]
    Off,
    Once,
    Locked,
}

/// The modifier keys' state, which decides the layer of the next press.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    second: bool,
    alpha: AlphaState,
}

impl Modifiers {
    pub fn layer(self) -> Layer {
        match (self.second, self.alpha) {
            (true, _) => Layer::Second,
            (false, AlphaState::Off) => Layer::Normal,
            (false, _) => Layer::Alpha,
        }
    }

    pub fn alpha(self) -> AlphaState {
        self.alpha
    }

    pub fn press_second(&mut self) {
        self.second = !self.second;
    }

    pub fn press_alpha(&mut self) {
        self.alpha = match (self.second, self.alpha) {
            (true, _) => AlphaState::Locked,
            (false, AlphaState::Off) => AlphaState::Once,
            (false, _) => AlphaState::Off,
        };
        self.second = false;
    }

    /// Locks ALPHA, as screens that take a name do on opening.
    pub fn lock_alpha(&mut self) {
        self.second = false;
        self.alpha = AlphaState::Locked;
    }

    /// After any other key: 2nd and a single ALPHA are used up, a lock stays.
    pub fn release(&mut self) {
        self.second = false;
        if self.alpha == AlphaState::Once {
            self.alpha = AlphaState::Off;
        }
    }
}
