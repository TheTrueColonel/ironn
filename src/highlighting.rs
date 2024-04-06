use crossterm::style::Color;

#[derive(PartialEq, Copy, Clone)]
pub enum Type {
    None,
    Number,
    Match,
    String,
    Character,
    Comment,
    MultilineComment,
    PrimaryKeywords,
    SecondaryKeywords,
}

impl Type {
    pub const fn to_color(self) -> Color {
        match self { 
            Self::Number => Color::Rgb { r: 220, g: 163, b: 163 },
            Self::Match => Color::Rgb { r: 38, g: 139, b: 210 },
            Self::String => Color::Rgb { r: 211, g: 54, b: 190 },
            Self::Character => Color::Rgb { r: 108, g: 113, b: 196 },
            Self::Comment | Self::MultilineComment => Color::Rgb { r: 133, g: 153, b: 0 },
            Self::PrimaryKeywords => Color::Rgb { r: 181, g: 137, b: 0 },
            Self::SecondaryKeywords => Color::Rgb { r: 42, g: 161, b: 152 },
            Self::None => Color::Rgb { r: 255, g: 255, b: 255 },
        }
    }
}