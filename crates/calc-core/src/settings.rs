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

/// Whether results may be complex, and how complex results are written:
/// REAL (a nonreal answer is an error), a+bi, or re^θi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NumberMode {
    #[default]
    Real,
    Rectangular,
    Polar,
}

/// The MODE settings this calculator honors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Settings {
    pub notation: Notation,
    pub decimals: Decimals,
    pub angle: Angle,
    #[serde(default)]
    pub number: NumberMode,
}

impl Settings {
    /// The status bar's words for the settings, in the calculator's order
    /// (`NORMAL FLOAT REAL RADIAN`, `SCI FIX2 a+bi DEGREE`).
    pub fn labels(self) -> [String; 4] {
        let notation = match self.notation {
            Notation::Normal => "NORMAL",
            Notation::Scientific => "SCI",
            Notation::Engineering => "ENG",
        };
        let decimals = match self.decimals {
            Decimals::Float => "FLOAT".to_owned(),
            Decimals::Fix(places) => format!("FIX{}", places.get()),
        };
        let number = match self.number {
            NumberMode::Real => "REAL",
            NumberMode::Rectangular => "a+bi",
            NumberMode::Polar => "re^θi",
        };
        let angle = match self.angle {
            Angle::Radian => "RADIAN",
            Angle::Degree => "DEGREE",
        };
        [
            notation.to_owned(),
            decimals,
            number.to_owned(),
            angle.to_owned(),
        ]
    }
}
