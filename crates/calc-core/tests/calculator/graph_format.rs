use calc_core::{Calculator, GraphView, Key, ScreenView};

use crate::support::{field_value, press, pressed};

const FORMAT: [Key; 2] = [Key::Second, Key::Zoom];

fn graph(calculator: &Calculator) -> GraphView {
    match calculator.screen().view {
        ScreenView::Graph(graph) => *graph,
        other => panic!("expected the graph, got {other:?}"),
    }
}

#[test]
fn format_choices_change_how_the_graph_is_drawn_and_are_kept() {
    let mut calculator = pressed(&FORMAT);
    press(
        &mut calculator,
        &[Key::Down, Key::Right, Key::Right, Key::Enter],
    );
    press(
        &mut calculator,
        &[Key::Down, Key::Right, Key::Enter, Key::Graph],
    );
    let format = graph(&calculator).format;
    assert_eq!((format.grid as u8, format.axes), (2, false));
    let reloaded = Calculator::load(&calculator.save()).unwrap();
    assert_eq!(reloaded.save(), calculator.save());
}

#[test]
fn arrows_on_the_graph_move_a_free_cursor_a_pixel_at_a_time() {
    let mut calculator = pressed(&[Key::Graph, Key::Right]);
    let free = graph(&calculator).free.expect("the free cursor is out");
    assert!(free.x > 0.0 && free.x < 0.1);
    assert_eq!(free.y_text, "Y=0");
    press(&mut calculator, &[Key::Up, Key::Left]);
    let free = graph(&calculator).free.expect("the free cursor is out");
    assert_eq!(free.x, 0.0);
    assert!(free.y > 0.0);
}

#[test]
fn zbox_makes_the_box_between_two_marked_corners_the_window() {
    let mut calculator = pressed(&[Key::Zoom, Key::Digit1]);
    press(&mut calculator, &[Key::Enter]);
    press(&mut calculator, &[Key::Right; 66]);
    press(&mut calculator, &[Key::Up; 41]);
    let corner = graph(&calculator).free.and_then(|free| free.corner);
    assert_eq!(corner, Some((0.0, 0.0)));
    press(&mut calculator, &[Key::Enter, Key::Window]);
    assert_eq!(field_value(&calculator, "Xmin"), "0");
    assert_eq!(field_value(&calculator, "Xmax"), "5");
    assert_eq!(field_value(&calculator, "Ymax"), "5");
}
