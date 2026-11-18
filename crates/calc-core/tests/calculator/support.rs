use calc_core::{Calculator, HomeView, Key, ScreenView};

pub fn pressed(keys: &[Key]) -> Calculator {
    let mut calculator = Calculator::default();
    press(&mut calculator, keys);
    calculator
}

pub fn press(calculator: &mut Calculator, keys: &[Key]) {
    keys.iter().for_each(|&key| calculator.press(key));
}

/// Opens MODE, moves `down` rows and `right` options, chooses, and quits.
pub fn choose(calculator: &mut Calculator, down: usize, right: usize) {
    press(calculator, &[Key::Mode]);
    (0..down).for_each(|_| calculator.press(Key::Down));
    (0..right).for_each(|_| calculator.press(Key::Right));
    press(calculator, &[Key::Enter, Key::Second, Key::Mode]);
}

pub fn home<'a>(calculator: &'a Calculator) -> HomeView<'a> {
    *calculator
        .screen()
        .home()
        .expect("the home screen is shown")
}

pub fn entry_text(calculator: &Calculator) -> String {
    home(calculator).entry.iter().map(|t| t.text()).collect()
}

pub fn results(calculator: &Calculator) -> Vec<String> {
    home(calculator)
        .history
        .iter()
        .map(|line| line.output.result.clone())
        .collect()
}

/// The value a settings screen (WINDOW, TBLSET) shows on the row labeled `label`.
pub fn field_value(calculator: &Calculator, label: &str) -> String {
    let ScreenView::Fields(fields) = calculator.screen().view else {
        panic!("expected a settings screen");
    };
    let row = fields.rows.iter().find(|row| row.label == label);
    row.expect("the row is shown").value.clone()
}
