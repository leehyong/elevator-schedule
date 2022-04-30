use iced::*;

const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../assets/font/iconfont.ttf"),
};

fn icon(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .font(ICONS)
        .width(Length::Units(20))
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .size(20)
}


pub fn loading_icon() -> Text {
    icon('\u{e64a}')
}

pub fn plus_icon() -> Text {
    icon('\u{e8fe}')
}

pub fn subtract_icon() -> Text {
    icon('\u{e6fe}')
}

pub fn up_icon() -> Text {
    icon('\u{e688}')
}

pub fn down_icon() -> Text {
    icon('\u{e6d3}')
}