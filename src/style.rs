use iced::*;

#[derive(Default)]
pub struct ActiveFloorBtnStyle;

impl button::StyleSheet for ActiveFloorBtnStyle {
    fn active(&self) -> button::Style {
        let mut style = button::Style::default();
        style.background = Some(Background::Color(Color::from_rgb8(51, 153, 255)));
        // style.text_color = Color::from_rgb8(255,0,0);
        // style.text_color = Color::from_rgb8(51, 102, 255);
        style.text_color = Color::WHITE;
        style
    }
}

#[derive(Default)]
pub struct ActiveFloorTxtStyle;

impl container::StyleSheet for ActiveFloorTxtStyle {
    fn style(&self) -> container::Style {
        use iced::button::StyleSheet;
        let btn_style = ActiveFloorBtnStyle::default();
        container::Style {
            text_color: Some(btn_style.active().text_color),
            background: btn_style.active().background,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}
