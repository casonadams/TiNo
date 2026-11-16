use crate::{
    editor::Editor,
    error::{CalcError, ErrorKind},
    expr::Expr,
    parse::parse,
};

/// How many functions Y= holds: `Y1` to `Y9`, then `Y0`.
pub const FUNCTION_COUNT: usize = 10;

/// One function of Y=: its entry and whether it is graphed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FunctionSlot {
    pub editor: Editor,
    pub enabled: bool,
}

/// The Y= functions, in screen order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Functions {
    slots: [FunctionSlot; FUNCTION_COUNT],
}

impl Functions {
    pub fn slots(&self) -> &[FunctionSlot; FUNCTION_COUNT] {
        &self.slots
    }

    pub fn slot_mut(&mut self, index: usize) -> &mut FunctionSlot {
        &mut self.slots[index]
    }

    /// The name of slot `index`: `Y1` for the first, `Y0` for the tenth.
    pub fn label(index: usize) -> String {
        format!("Y{}", (index + 1) % FUNCTION_COUNT)
    }

    /// The slots that are graphed: enabled and not empty.
    pub fn graphed(&self) -> impl Iterator<Item = usize> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.enabled && !slot.editor.tokens().is_empty())
            .map(|(index, _)| index)
    }

    /// Every graphed function parsed, or the first that does not parse with
    /// its slot.
    pub fn parse_graphed(&self) -> Result<Vec<(usize, Expr)>, (usize, CalcError)> {
        self.graphed()
            .map(|index| {
                parse_function(self.slots[index].editor.tokens())
                    .map(|expr| (index, expr))
                    .map_err(|e| (index, e))
            })
            .collect()
    }

    /// Every slot parsed, as Y-vars read them: an empty slot is undefined.
    pub fn parse_all(&self) -> Vec<Result<Expr, CalcError>> {
        self.slots
            .iter()
            .map(|slot| match slot.editor.tokens() {
                [] => Err(CalcError::new(ErrorKind::Undefined)),
                tokens => parse_function(tokens),
            })
            .collect()
    }
}

/// A function's entry as an expression; storing (`→`) is not allowed in one.
fn parse_function(tokens: &[crate::token::Token]) -> Result<Expr, CalcError> {
    let statement = parse(tokens)?;
    match statement.store {
        None => Ok(statement.expr),
        Some(_) => {
            let arrow = tokens.iter().position(|t| *t == crate::token::Token::Store);
            Err(CalcError {
                kind: ErrorKind::Syntax,
                position: arrow,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::tests::tokens;

    fn functions(entries: &[(usize, &str, bool)]) -> Functions {
        let mut functions = Functions::default();
        for &(index, spelling, enabled) in entries {
            let slot = functions.slot_mut(index);
            slot.editor.replace(tokens(spelling));
            slot.enabled = enabled;
        }
        functions
    }

    #[test]
    fn labels_run_y1_to_y9_then_y0() {
        assert_eq!(Functions::label(0), "Y1");
        assert_eq!(Functions::label(9), "Y0");
    }

    #[test]
    fn only_enabled_non_empty_functions_are_graphed() {
        let functions = functions(&[
            (0, "A", true),
            (1, "B", false),
            (2, "", true),
            (3, "C", true),
        ]);
        assert_eq!(functions.graphed().collect::<Vec<_>>(), [0, 3]);
    }

    #[test]
    fn a_function_that_does_not_parse_names_its_slot() {
        let functions = functions(&[(0, "A", true), (4, "2+", true)]);
        let (slot, error) = functions.parse_graphed().unwrap_err();
        assert_eq!(
            (slot, error.kind, error.position),
            (4, ErrorKind::Syntax, Some(2))
        );
    }

    #[test]
    fn storing_is_a_syntax_error_in_a_function() {
        let functions = functions(&[(0, "5→A", true)]);
        let (_, error) = functions.parse_graphed().unwrap_err();
        assert_eq!((error.kind, error.position), (ErrorKind::Syntax, Some(1)));
    }
}
