use serde::{Deserialize, Serialize};

use crate::{field_editor::Field, window::Window};

/// A line of the WINDOW screen, also read as a variable from VARS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WindowField {
    XMin,
    XMax,
    XScale,
    YMin,
    YMax,
    YScale,
    XRes,
}

impl Field for WindowField {
    const TITLE: &'static str = "WINDOW";
    const ALL: &'static [Self] = &[
        WindowField::XMin,
        WindowField::XMax,
        WindowField::XScale,
        WindowField::YMin,
        WindowField::YMax,
        WindowField::YScale,
        WindowField::XRes,
    ];

    fn label(self) -> &'static str {
        match self {
            WindowField::XMin => "Xmin",
            WindowField::XMax => "Xmax",
            WindowField::XScale => "Xscl",
            WindowField::YMin => "Ymin",
            WindowField::YMax => "Ymax",
            WindowField::YScale => "Yscl",
            WindowField::XRes => "Xres",
        }
    }
}

impl WindowField {
    pub fn get(self, window: &Window) -> f64 {
        match self {
            WindowField::XMin => window.x_min,
            WindowField::XMax => window.x_max,
            WindowField::XScale => window.x_scale,
            WindowField::YMin => window.y_min,
            WindowField::YMax => window.y_max,
            WindowField::YScale => window.y_scale,
            WindowField::XRes => window.x_res,
        }
    }

    pub fn set(self, window: &mut Window, value: f64) {
        let field = match self {
            WindowField::XMin => &mut window.x_min,
            WindowField::XMax => &mut window.x_max,
            WindowField::XScale => &mut window.x_scale,
            WindowField::YMin => &mut window.y_min,
            WindowField::YMax => &mut window.y_max,
            WindowField::YScale => &mut window.y_scale,
            WindowField::XRes => &mut window.x_res,
        };
        *field = value;
    }
}
