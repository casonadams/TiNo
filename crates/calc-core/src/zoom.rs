use std::f64::consts::PI;

use crate::{settings::Angle, window::Window};

/// How much Zoom In narrows and Zoom Out widens each range.
const ZOOM_FACTOR: f64 = 4.0;

/// The ZOOM menu's commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomItem {
    ZBox,
    ZoomIn,
    ZoomOut,
    ZDecimal,
    ZSquare,
    ZStandard,
    ZTrig,
    ZoomFit,
}

impl ZoomItem {
    pub const ALL: [ZoomItem; 8] = [
        ZoomItem::ZBox,
        ZoomItem::ZoomIn,
        ZoomItem::ZoomOut,
        ZoomItem::ZDecimal,
        ZoomItem::ZSquare,
        ZoomItem::ZStandard,
        ZoomItem::ZTrig,
        ZoomItem::ZoomFit,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ZoomItem::ZBox => "ZBox",
            ZoomItem::ZoomIn => "Zoom In",
            ZoomItem::ZoomOut => "Zoom Out",
            ZoomItem::ZDecimal => "ZDecimal",
            ZoomItem::ZSquare => "ZSquare",
            ZoomItem::ZStandard => "ZStandard",
            ZoomItem::ZTrig => "ZTrig",
            ZoomItem::ZoomFit => "ZoomFit",
        }
    }

    /// The digit that picks the item, as numbered on the calculator's ZOOM
    /// menu.
    pub fn digit(self) -> u8 {
        match self {
            ZoomItem::ZBox => 1,
            ZoomItem::ZoomIn => 2,
            ZoomItem::ZoomOut => 3,
            ZoomItem::ZDecimal => 4,
            ZoomItem::ZSquare => 5,
            ZoomItem::ZStandard => 6,
            ZoomItem::ZTrig => 7,
            ZoomItem::ZoomFit => 0,
        }
    }

    /// The item `digit` picks, if any.
    pub fn numbered(digit: u8) -> Option<ZoomItem> {
        ZoomItem::ALL.into_iter().find(|item| item.digit() == digit)
    }
}

/// What some zooms need besides the window: the angle mode (ZTrig) and the
/// y extent of the graphed functions over the current x range (ZoomFit),
/// when they have values there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomContext {
    pub angle: Angle,
    pub fit: Option<(f64, f64)>,
}

/// The window `item` makes from `window`, Zoom In and Out keeping its
/// center; `None` for ZBox, whose window comes from two corners marked on
/// the graph.
pub fn apply(item: ZoomItem, window: Window, context: ZoomContext) -> Option<Window> {
    Some(match item {
        ZoomItem::ZBox => return None,
        ZoomItem::ZoomIn => window.zoomed(ZOOM_FACTOR, (0.5, 0.5)),
        ZoomItem::ZoomOut => window.zoomed(1.0 / ZOOM_FACTOR, (0.5, 0.5)),
        ZoomItem::ZDecimal => decimal(window),
        ZoomItem::ZSquare => square(window),
        ZoomItem::ZStandard => Window::standard(),
        ZoomItem::ZTrig => trig(context.angle),
        ZoomItem::ZoomFit => context.fit.map_or(window, |extent| fitted(window, extent)),
    })
}

/// x from -6.6 to 6.6 (.05 per column of the color screen's graph) and y
/// from -4.1 to 4.1.
fn decimal(window: Window) -> Window {
    Window {
        x_min: -6.6,
        x_max: 6.6,
        x_scale: 1.0,
        y_min: -4.1,
        y_max: 4.1,
        y_scale: 1.0,
        ..window
    }
}

/// Widens the axis whose pixels are smaller so pixels are square, keeping
/// the center.
fn square(window: Window) -> Window {
    let (dx, dy) = (window.delta_x(), window.delta_y());
    if dx > dy {
        let center = (window.y_min + window.y_max) / 2.0;
        let half = (window.y_max - window.y_min) / 2.0 * dx / dy;
        Window {
            y_min: center - half,
            y_max: center + half,
            ..window
        }
    } else {
        let center = (window.x_min + window.x_max) / 2.0;
        let half = (window.x_max - window.x_min) / 2.0 * dy / dx;
        Window {
            x_min: center - half,
            x_max: center + half,
            ..window
        }
    }
}

/// About two turns of x either side of 0, ticks every quarter turn, y from -4 to 4.
fn trig(angle: Angle) -> Window {
    let (turn, tick) = match angle {
        Angle::Radian => (2.0 * PI, PI / 2.0),
        Angle::Degree => (360.0, 90.0),
    };
    let reach = turn * 47.0 / 48.0;
    Window {
        x_min: -reach,
        x_max: reach,
        x_scale: tick,
        y_min: -4.0,
        y_max: 4.0,
        y_scale: 1.0,
        ..Window::standard()
    }
}

fn fitted(window: Window, (low, high): (f64, f64)) -> Window {
    let (y_min, y_max) = if low < high {
        (low, high)
    } else {
        (low - 1.0, high + 1.0)
    };
    Window {
        y_min,
        y_max,
        ..window
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(angle: Angle, fit: Option<(f64, f64)>) -> ZoomContext {
        ZoomContext { angle, fit }
    }

    const RADIAN: ZoomContext = ZoomContext {
        angle: Angle::Radian,
        fit: None,
    };

    #[test]
    fn zoom_in_then_out_restores_the_window() {
        let window = Window::standard();
        let zoomed_in = apply(ZoomItem::ZoomIn, window, RADIAN).unwrap();
        assert_eq!(apply(ZoomItem::ZoomOut, zoomed_in, RADIAN), Some(window));
        assert_eq!(apply(ZoomItem::ZBox, window, RADIAN), None);
    }

    #[test]
    fn zsquare_makes_pixels_square() {
        let squared = apply(ZoomItem::ZSquare, Window::standard(), RADIAN).unwrap();
        assert!((squared.delta_x() - squared.delta_y()).abs() < 1e-12);
        assert_eq!((squared.y_min, squared.y_max), (-10.0, 10.0));
    }

    #[test]
    fn ztrig_follows_the_angle_mode() {
        let degrees = apply(
            ZoomItem::ZTrig,
            Window::standard(),
            context(Angle::Degree, None),
        )
        .unwrap();
        assert_eq!(
            (degrees.x_min, degrees.x_max, degrees.x_scale),
            (-352.5, 352.5, 90.0)
        );
    }

    #[test]
    fn zoom_fit_takes_the_extent_and_widens_a_flat_one() {
        let fit = |extent| {
            apply(
                ZoomItem::ZoomFit,
                Window::standard(),
                context(Angle::Radian, Some(extent)),
            )
            .unwrap()
        };
        let fitted = fit((-3.0, 8.0));
        assert_eq!((fitted.y_min, fitted.y_max), (-3.0, 8.0));
        let flat = fit((2.0, 2.0));
        assert_eq!((flat.y_min, flat.y_max), (1.0, 3.0));
    }
}
