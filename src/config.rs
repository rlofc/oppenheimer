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
        Self(" 〉 ".to_string())
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
            header: Style {
                fg: ratatui::style::Color::White,
                bg: ratatui::style::Color::default(),
            },
            active_header: Style {
                fg: ratatui::style::Color::White,
                bg: ratatui::style::Color::default(),
            },
            item: Style::default(),
            tag: Style {
                fg: ratatui::style::Color::Yellow,
                bg: ratatui::style::Color::default(),
            },
            tag_hashsign: Style {
                fg: ratatui::style::Color::DarkGray,
                bg: ratatui::style::Color::default(),
            },
            fringe_on: Style {
                fg: ratatui::style::Color::LightBlue,
                bg: ratatui::style::Color::default(),
            },
            fringe_off: Style {
                fg: ratatui::style::Color::Indexed(239),
                bg: ratatui::style::Color::default(),
            },
            selected: Style {
                fg: ratatui::style::Color::default(),
                bg: ratatui::style::Color::Indexed(235),
            },
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Style {
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub fg: ratatui::style::Color,
    #[serde(
        serialize_with = "serialize_color",
        deserialize_with = "deserialize_color"
    )]
    pub bg: ratatui::style::Color,
}

fn serialize_color<S>(color: &ratatui::style::Color, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&color_to_str(color))
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<ratatui::style::Color, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    if let Some(c) = str_to_color(&s) {
        Ok(c)
    } else {
        Err(serde::de::Error::custom("Unsupported color"))
    }
}

fn str_to_color(color_as_str: &str) -> Option<ratatui::style::Color> {
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

        Some(ratatui::style::Color::Rgb(r, g, b))
    } else {
        match color_as_str {
            "black" => Some(ratatui::style::Color::Black),
            "red" => Some(ratatui::style::Color::Red),
            "green" => Some(ratatui::style::Color::Green),
            "blue" => Some(ratatui::style::Color::Blue),
            "magenta" => Some(ratatui::style::Color::Magenta),
            "cyan" => Some(ratatui::style::Color::Cyan),
            "yellow" => Some(ratatui::style::Color::Yellow),
            "white" => Some(ratatui::style::Color::White),
            "darkgray" => Some(ratatui::style::Color::DarkGray),
            "lightred" => Some(ratatui::style::Color::LightRed),
            "lightgreen" => Some(ratatui::style::Color::LightGreen),
            "lightyellow" => Some(ratatui::style::Color::LightYellow),
            "lightblue" => Some(ratatui::style::Color::LightBlue),
            "lightmagenta" => Some(ratatui::style::Color::LightMagenta),
            "lightcyan" => Some(ratatui::style::Color::LightCyan),
            "lightgray" => Some(ratatui::style::Color::Gray),
            "" => Some(ratatui::style::Color::default()),
            _ => Some(ratatui::style::Color::Indexed(
                color_as_str.parse::<u8>().unwrap(),
            )),
        }
    }
}

fn color_to_str(color: &ratatui::style::Color) -> String {
    match color {
        ratatui::style::Color::Black => "black",
        ratatui::style::Color::Red => "red",
        ratatui::style::Color::Green => "green",
        ratatui::style::Color::Yellow => "yellow",
        ratatui::style::Color::Blue => "blue",
        ratatui::style::Color::Magenta => "magenta",
        ratatui::style::Color::Cyan => "cyan",
        ratatui::style::Color::White => "white",
        ratatui::style::Color::DarkGray => "darkgray",
        ratatui::style::Color::LightRed => "lightred",
        ratatui::style::Color::LightGreen => "lightgreen",
        ratatui::style::Color::LightYellow => "lightyellow",
        ratatui::style::Color::LightBlue => "lightblue",
        ratatui::style::Color::LightMagenta => "lightmagenta",
        ratatui::style::Color::LightCyan => "lightcyan",
        ratatui::style::Color::Gray => "lightgray",
        ratatui::style::Color::Rgb(r, g, b) => return format!("#{:02X}{:02X}{:02X}", r, g, b),
        ratatui::style::Color::Indexed(val) => return format!("{}", val),
        ratatui::style::Color::Reset => return "".to_string(),
    }
    .to_string()
}
