use ratatui::style::Color;
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct BoardConfig {
    #[serde(default)]
    pub dim_tailing_items: bool,
    #[serde(default)]
    pub path_separator: PathSeparator,
    #[serde(default, rename = "Styles")]
    pub styles: Styles,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathSeparator(pub String);
impl Default for PathSeparator {
    fn default() -> Self {
        Self(" 〉 ".to_string())
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(rename = "Board")]
    pub board_config: BoardConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Styles {
    pub header: Style,
    pub active_header: Style,
    pub item: Style,
    pub tag: Style,
    pub tag_hashsign: Style,
    pub fringe_on: Style,
    pub fringe_off: Style,
    pub selected: Style,
}

impl Default for Styles {
    fn default() -> Self {
        Styles {
            header: Style::with_fg(Color::White),
            active_header: Style::with_fg(Color::White),
            item: Style::default(),
            tag: Style::with_fg(Color::Yellow),
            tag_hashsign: Style::with_fg(Color::DarkGray),
            fringe_on: Style::with_fg(Color::LightBlue),
            fringe_off: Style::with_fg(Color::Indexed(239)),
            selected: Style {
                fg: Color::default(),
                bg: Color::Indexed(235),
            },
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Style {
    #[serde(with = "color_serde")]
    pub fg: Color,
    #[serde(with = "color_serde")]
    pub bg: Color,
}

impl Style {
    pub fn with_fg(fg: Color) -> Self {
        Style {
            fg,
            bg: Color::default(),
        }
    }
}

mod color_serde {
    use super::{color_to_str, str_to_color};
    use ratatui::style::Color;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(color: &Color, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&color_to_str(color))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Color, D::Error> {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        str_to_color(&s).ok_or_else(|| serde::de::Error::custom("Unsupported color"))
    }
}

fn str_to_color(color_as_str: &str) -> Option<Color> {
    if color_as_str.starts_with("#") {
        let hex = color_as_str.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 3 {
            return None;
        }
        let hex = if hex.len() == 3 {
            hex.chars()
                .flat_map(|c| std::iter::repeat(c).take(2))
                .collect::<String>()
        } else {
            hex.to_string()
        };

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Color::Rgb(r, g, b))
    } else {
        match color_as_str {
            "black" => Some(Color::Black),
            "red" => Some(Color::Red),
            "green" => Some(Color::Green),
            "blue" => Some(Color::Blue),
            "magenta" => Some(Color::Magenta),
            "cyan" => Some(Color::Cyan),
            "yellow" => Some(Color::Yellow),
            "white" => Some(Color::White),
            "darkgray" => Some(Color::DarkGray),
            "lightred" => Some(Color::LightRed),
            "lightgreen" => Some(Color::LightGreen),
            "lightyellow" => Some(Color::LightYellow),
            "lightblue" => Some(Color::LightBlue),
            "lightmagenta" => Some(Color::LightMagenta),
            "lightcyan" => Some(Color::LightCyan),
            "lightgray" => Some(Color::Gray),
            "" => Some(Color::default()),
            _ => Some(Color::Indexed(color_as_str.parse::<u8>().unwrap())),
        }
    }
}

fn color_to_str(color: &Color) -> String {
    match color {
        Color::Black => "black",
        Color::Red => "red",
        Color::Green => "green",
        Color::Yellow => "yellow",
        Color::Blue => "blue",
        Color::Magenta => "magenta",
        Color::Cyan => "cyan",
        Color::White => "white",
        Color::DarkGray => "darkgray",
        Color::LightRed => "lightred",
        Color::LightGreen => "lightgreen",
        Color::LightYellow => "lightyellow",
        Color::LightBlue => "lightblue",
        Color::LightMagenta => "lightmagenta",
        Color::LightCyan => "lightcyan",
        Color::Gray => "lightgray",
        Color::Rgb(r, g, b) => return format!("#{:02X}{:02X}{:02X}", r, g, b),
        Color::Indexed(val) => return format!("{}", val),
        Color::Reset => return "".to_string(),
    }
    .to_string()
}
