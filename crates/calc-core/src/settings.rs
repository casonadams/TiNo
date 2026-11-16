use serde::{Deserialize, Serialize};

/// How results are written: positional where they fit, always scientific, or
/// scientific with exponents that are multiples of 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Notation {
    #[default]
    Normal,
    Scientific,
    Engineering,
}

/// How many decimals results show: as many as fit (Float) or exactly `n`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Decimals {
    #[default]
    Float,
    Fix(FixedPlaces),
}

/// A count of fixed decimal places, 0 to 9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct FixedPlaces(u8);

impl FixedPlaces {
    pub const MAX: u8 = 9;

    /// `places` decimal places; `None` beyond 9.
    pub fn new(places: u8) -> Option<Self> {
        (places <= Self::MAX).then_some(FixedPlaces(places))
    }

    pub fn get(self) -> usize {
        usize::from(self.0)
    }
}

impl TryFrom<u8> for FixedPlaces {
    type Error = String;

    fn try_from(places: u8) -> Result<Self, Self::Error> {
        FixedPlaces::new(places).ok_or_else(|| format!("{places} fixed places is more than 9"))
    }
}

impl From<FixedPlaces> for u8 {
    fn from(places: FixedPlaces) -> Self {
        places.0
    }
}

/// The unit trigonometric functions take and inverse ones return.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Angle {
    #[default]
    Radian,
    Degree,
}

/// The MODE settings this calculator honors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Settings {
    pub notation: Notation,
    pub decimals: Decimals,
    pub angle: Angle,
}

impl Settings {
    /// The status bar's words for the settings (`NORMAL FLOAT RADIAN`, `SCI FIX2 DEGREE`).
    pub fn labels(self) -> [String; 3] {
        let notation = match self.notation {
            Notation::Normal => "NORMAL",
            Notation::Scientific => "SCI",
            Notation::Engineering => "ENG",
        };
        let decimals = match self.decimals {
            Decimals::Float => "FLOAT".to_owned(),
            Decimals::Fix(places) => format!("FIX{}", places.get()),
        };
        let angle = match self.angle {
            Angle::Radian => "RADIAN",
            Angle::Degree => "DEGREE",
        };
        [notation.to_owned(), decimals, angle.to_owned()]
    }
}
