use super::*;
use crate::parse::tests::tokens;

/// A line: `keyword` (if any) followed by tokens spelled as `parse` tests spell them.
fn line(keyword: Option<Keyword>, spelling: &str) -> Vec<Token> {
    keyword
        .map(Token::Keyword)
        .into_iter()
        .chain(tokens(spelling))
        .collect()
}

/// Each step's jump target, `None` for steps that do not jump.
fn jumps(lines: &[Vec<Token>]) -> Vec<Option<usize>> {
    let target = |step: Step| match step.op {
        Op::Unless(_, to) | Op::Jump(to) => Some(to),
        Op::ForStart { exit, .. } => Some(exit),
        Op::ForNext { body, .. } => Some(body),
        _ => None,
    };
    compile(lines).unwrap().into_iter().map(target).collect()
}

#[test]
fn if_then_else_jumps_around_each_part() {
    let program = [
        line(Some(Keyword::If), "A"),
        line(Some(Keyword::Then), ""),
        line(None, "1"),
        line(Some(Keyword::Else), ""),
        line(None, "2"),
        line(Some(Keyword::End), ""),
    ];
    assert_eq!(jumps(&program), [Some(3), None, Some(4), None]);
}

#[test]
fn a_one_line_if_skips_only_the_next_instruction() {
    let mut guarded = line(Some(Keyword::If), "A");
    guarded.push(Token::Colon);
    guarded.extend(line(Some(Keyword::Disp), "1"));
    let program = [guarded, line(None, "2")];
    assert_eq!(jumps(&program), [Some(2), None, None]);
}

#[test]
fn loops_jump_back_to_their_start() {
    let program = [
        line(Some(Keyword::For), "X,1,3"),
        line(Some(Keyword::While), "A"),
        line(Some(Keyword::End), ""),
        line(Some(Keyword::End), ""),
    ];
    assert_eq!(jumps(&program), [Some(4), Some(3), Some(1), Some(1)]);
}

#[test]
fn unmatched_blocks_are_syntax_errors_on_their_line() {
    let unclosed = [line(Some(Keyword::While), "1"), line(None, "2")];
    assert_eq!(compile(&unclosed).unwrap_err().1, 1);
    let stray = [line(None, "1"), line(Some(Keyword::End), "")];
    let (error, at) = compile(&stray).unwrap_err();
    assert_eq!((error.kind, at), (ErrorKind::Syntax, 1));
}

#[test]
fn names_start_with_a_letter_and_run_up_to_eight() {
    assert_eq!(name(&tokens("AB1")), Some("AB1".to_owned()));
    assert_eq!(name(&tokens("1A")), None);
    assert_eq!(name(&tokens("ABCDABCDA")), None);
}
