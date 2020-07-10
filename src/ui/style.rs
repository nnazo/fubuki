use iced::{button, Background, Color, container};

// const CONTAINER_BACKGROUND: Color = Color::from_rgb8(11u8, 22u8, 34u8);

pub enum Button {
    Nav { selected: bool },
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        match self {
            Button::Nav { selected } => {
                if *selected {
                    button::Style {
                        background: Some(
                            Background::Color(Color::from_rgb8(21u8, 31u8, 46u8))
                        ),
                        border_radius: 0,
                        text_color: Color::from_rgb8(144u8, 168u8, 191u8),
                        ..button::Style::default()
                    }
                } else {
                    button::Style {
                        background: Some(
                            Background::Color(Color::from_rgb8(21u8, 31u8, 46u8))
                        ),
                        border_radius: 0,
                        text_color: Color::from_rgb8(114u8, 138u8, 161u8),
                        ..button::Style::default()
                    }
                }
            },
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();
        button::Style {
            text_color: match self {
                Button::Nav { selected } => {
                    if !selected {
                        Color::from_rgb8(144u8, 168u8, 191u8)
                    } else {
                        active.text_color
                    }
                },
            },
            ..active
        }
    }

    // fn pressed(&self) -> button::Style {}

    // fn disabled(&self) -> button::Style {}
}

pub enum Container {
    Background,
}

impl container::StyleSheet for Container {
    fn style(&self) -> container::Style {
        match self {
            Container::Background => container::Style {
                background: Some(Background::Color(Color::from_rgb8(11u8, 22u8, 34u8))),
                text_color: Some(Color::from_rgb8(159u8, 173u8, 189u8)),
                border_radius: 0,
                border_width: 0,
                border_color: Color::from_rgba(0.0, 0.0, 0.0, 0.0)
            }
        }
    }
}