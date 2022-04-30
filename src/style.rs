use iced::*;
use iced::button::Style;

#[derive(Default)]
pub struct ActiveFloorBtnStyle;

impl button::StyleSheet for ActiveFloorBtnStyle {
    fn active(&self) -> Style {
        let mut style = Style::default();
        style.background = Some(Background::Color(Color::from_rgb8(51, 153, 255)));
        // style.text_color = Color::from_rgb8(255,0,0);
        // style.text_color = Color::from_rgb8(51, 102, 255);
        style.text_color = Color::WHITE;
        style
    }
}

