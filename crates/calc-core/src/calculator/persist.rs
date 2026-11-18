use std::fmt;

use serde::{Deserialize, Serialize};

use super::Calculator;
use crate::{
    eval::Memory, functions::FUNCTION_COUNT, graph_format::GraphFormat, home::Output,
    program::Program, settings::Settings, table::TableSettings, token::Token, window::Window,
};

/// The snapshot format written by this version; a snapshot of another
/// version is not read.
const SNAPSHOT_VERSION: u32 = 2;

/// What a calculator keeps across sessions: everything but the screen
/// being shown and the modifier keys.
#[derive(Debug, Serialize, Deserialize)]
struct Snapshot {
    version: u32,
    settings: Settings,
    memory: Memory,
    functions: Vec<FunctionState>,
    window: Window,
    table: TableSettings,
    #[serde(default)]
    format: GraphFormat,
    history: Vec<Output>,
    entries: Vec<Vec<Token>>,
    #[serde(default)]
    programs: Vec<Program>,
}

/// A Y= function as kept: its entry and whether it is graphed.
#[derive(Debug, Serialize, Deserialize)]
struct FunctionState {
    entry: Vec<Token>,
    enabled: bool,
}

/// Why a saved calculator could not be read.
#[derive(Debug)]
pub enum LoadError {
    /// Not a snapshot this calculator understands.
    Malformed(serde_json::Error),
    /// A snapshot of another format version.
    Version(u32),
    /// More Y= functions than the calculator has.
    TooManyFunctions(usize),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Malformed(error) => write!(f, "malformed calculator snapshot: {error}"),
            LoadError::Version(version) => write!(
                f,
                "calculator snapshot version {version} is not {SNAPSHOT_VERSION}"
            ),
            LoadError::TooManyFunctions(count) => {
                write!(f, "calculator snapshot has {count} functions")
            }
        }
    }
}

impl std::error::Error for LoadError {}

impl Calculator {
    /// The calculator's memory, settings, functions, window, table and graph
    /// format settings, home screen, entries and programs, as text `load`
    /// reads back.
    pub fn save(&self) -> String {
        let functions = self.functions.slots().iter().map(|slot| FunctionState {
            entry: slot.editor.tokens().to_vec(),
            enabled: slot.enabled,
        });
        let snapshot = Snapshot {
            version: SNAPSHOT_VERSION,
            settings: self.settings,
            memory: self.memory.clone(),
            functions: functions.collect(),
            window: self.window,
            table: self.table,
            format: self.format,
            history: self
                .history
                .lines()
                .iter()
                .map(|line| line.output.clone())
                .collect(),
            entries: self.entries.entries().iter().cloned().collect(),
            programs: self.programs.clone(),
        };
        serde_json::to_string(&snapshot).expect("a snapshot holds only strings, numbers and lists")
    }

    /// A calculator as `save` wrote it, on the home screen.
    pub fn load(text: &str) -> Result<Self, LoadError> {
        let snapshot: Snapshot = serde_json::from_str(text).map_err(LoadError::Malformed)?;
        if snapshot.version != SNAPSHOT_VERSION {
            return Err(LoadError::Version(snapshot.version));
        }
        if snapshot.functions.len() > FUNCTION_COUNT {
            return Err(LoadError::TooManyFunctions(snapshot.functions.len()));
        }
        let mut calculator = Calculator {
            settings: snapshot.settings,
            memory: snapshot.memory,
            window: snapshot.window,
            table: snapshot.table,
            format: snapshot.format,
            programs: snapshot.programs,
            ..Calculator::default()
        };
        for (index, function) in snapshot.functions.into_iter().enumerate() {
            let slot = calculator.functions.slot_mut(index);
            slot.editor.replace(function.entry);
            slot.enabled = function.enabled;
        }
        snapshot
            .history
            .into_iter()
            .for_each(|output| calculator.history.push(output));
        snapshot
            .entries
            .into_iter()
            .for_each(|entry| calculator.entries.push(entry));
        Ok(calculator)
    }
}
