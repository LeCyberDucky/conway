use iced::{Background, Color, button, container, slider, text_input};

#[allow(clippy::eq_op)]

// Cell grid colors
pub const LIVE_CELL: Color = Color::from_rgba(255.0 / 255.0, 0.0 / 255.0, 128.0 / 255.0, 1.0);
pub const DEAD_CELL: Color = Color::from_rgba(36.0 / 255.0, 36.0 / 255.0, 36.0 / 255.0, 1.0);
pub const GRID_LINE: Color = Color::from_rgba(125.0 / 255.0, 0.0 / 255.0, 175.0 / 255.0, 1.0);

// Control colors
pub const ACTIVE: Color = Color::from_rgba(230.0 / 255.0, 0.0 / 255.0, 100.0 / 255.0, 1.0);
pub const HOVERED: Color = Color::from_rgba(250.0 / 255.0, 0.0 / 255.0, 115.0 / 255.0, 1.0);
pub const ACCENT: Color = Color::from_rgba(0.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0, 1.0);
pub const BACKGROUND: Color = DEAD_CELL;
pub const TEXT: Color = Color::WHITE;
pub const BORDER: Color = TEXT;

// Settings
pub const BORDER_WIDTH: f32 = 2.0;
pub const BORDER_RADIUS: f32 = 5.0;

pub struct InputField;

impl  text_input::StyleSheet for InputField {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Background::Color(ACCENT),
            border_radius: BORDER_RADIUS,
            ..text_input::Style::default()
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_width: BORDER_WIDTH,
            border_color: BORDER,
            background: Background::Color(Color {
                a: 0.5,
                ..ACCENT
            }),
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> Color {
        Color {
            a: 0.5,
            ..self.value_color()
        }
    }

    fn value_color(&self) -> Color {
        TEXT
    }

    fn selection_color(&self) -> Color {
        Color {
            a: 0.3,
            ..ACTIVE
        }
    }
}

pub struct Button;

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(ACCENT)),
            text_color: TEXT,
            border_radius: BORDER_RADIUS,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(Color {
                a: 0.5,
                ..ACCENT
            })),
            text_color: TEXT,
            border_color: BORDER,
            border_width: BORDER_WIDTH,
            ..self.active()
        }
    }

    fn pressed(&self) -> button::Style {
        button::Style {
            border_width: BORDER_WIDTH/2.0,
            background: Some(Background::Color(ACTIVE)),
            ..self.hovered()
        }
    }
}

pub struct Container;

impl container::StyleSheet for Container {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(BACKGROUND)),
            text_color: Some(ACCENT),
            ..container::Style::default()
        }
    }
}

pub struct TextSnippet;

impl container::StyleSheet for TextSnippet {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(ACCENT)),
            text_color: Some(TEXT),
            border_radius: BORDER_RADIUS,
            // border_width: 2.0,
            ..container::Style::default()
        }
    }
}

pub struct Slider;

impl slider::StyleSheet for Slider {
    fn active(&self) -> slider::Style {
        slider::Style {
            rail_colors: (ACTIVE, Color { a: 0.1, ..ACTIVE }),
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 9.0 },
                color: ACTIVE,
                border_width: BORDER_WIDTH,
                border_color: ACCENT,
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
