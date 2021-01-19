use iced::{Color,
slider};

#[allow(clippy::eq_op)]

// Cell grid colors
pub const LIVE_CELL: Color = Color::from_rgba(255.0 / 255.0, 0.0 / 255.0, 128.0 / 255.0, 1.0);
pub const DEAD_CELL: Color = Color::from_rgba(36.0 / 255.0, 36.0 / 255.0, 36.0 / 255.0, 1.0);
pub const GRID_LINE: Color = Color::from_rgba(125.0 / 255.0, 0.0 / 255.0, 175.0 / 255.0, 1.0);

// Control colors
pub const ACTIVE: Color = Color::from_rgba(230.0/255.0, 0.0/255.0, 100.0/255.0, 1.0);
pub const HOVERED: Color = Color::from_rgba(250.0/255.0, 0.0/255.0, 115.0/255.0, 1.0);

pub struct Slider;

impl slider::StyleSheet for Slider {
    fn active(&self) -> slider::Style {
        slider::Style {
            rail_colors: (ACTIVE, Color { a: 0.1, ..ACTIVE }),
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 9.0 },
                color: ACTIVE,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self) -> slider::Style {
        let active = self.active();

        slider::Style {
            handle: slider::Handle {
                color: HOVERED,
                ..active.handle
            },
            ..active
        }
    }

    fn dragging(&self) -> slider::Style {
        let active = self.active();

        slider::Style {
            handle: slider::Handle {
                color: Color::from_rgb(0.85, 0.85, 0.85),
                ..active.handle
            },
            ..active
        }
    }
}