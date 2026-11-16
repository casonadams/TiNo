use calc_core::{Calculator, HomeView, Key};

pub fn pressed(keys: &[Key]) -> Calculator {
    let mut calculator = Calculator::default();
    press(&mut calculator, keys);
    calculator
}

pub fn press(calculator: &mut Calculator, keys: &[Key]) {
    keys.iter().for_each(|&key| calculator.press(key));
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
