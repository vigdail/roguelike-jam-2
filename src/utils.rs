use bevy::prelude::*;
use bevy_ascii_terminal::{CharFormat, Terminal, Tile};

pub trait Grayscale {
    fn grayscale(&self) -> Self;
}

impl Grayscale for Color {
    fn grayscale(&self) -> Self {
        let [r, g, b, _]: [f32; 4] = (*self).into();
        let grey = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let grey = grey / 6.0;
        Color::rgb(grey, grey, grey)
    }
}

impl Grayscale for Tile {
    fn grayscale(&self) -> Self {
        Self {
            glyph: self.glyph,
            fg_color: self.fg_color.grayscale(),
            bg_color: self.bg_color.grayscale(),
        }
    }
}

pub struct TitleBarStyle {
    pub width: usize,
    pub filled: CharFormat,
    pub empty: CharFormat,
}

pub trait UiUtils {
    fn draw_titled_bar(
        &mut self,
        position: [i32; 2],
        text: &str,
        value: i32,
        max: i32,
        style: TitleBarStyle,
    );
}

impl UiUtils for Terminal {
    fn draw_titled_bar(
        &mut self,
        position: [i32; 2],
        text: &str,
        value: i32,
        max: i32,
        style: TitleBarStyle,
    ) {
        let [x, y] = position;
        let normalized = match max {
            0 => 0.0,
            _ => value as f32 / max as f32,
        };

        let v = f32::ceil(normalized * style.width as f32) as usize;

        let text_start = (style.width - text.len()) / 2;

        for i in 0..text_start {
            let format = if i < v { style.filled } else { style.empty };
            self.put_char_formatted([x + i as i32, y], ' ', format);
        }

        for (i, c) in text.chars().enumerate() {
            let format = if i + text_start < v {
                style.filled
            } else {
                style.empty
            };
            self.put_char_formatted([x + (i + text_start) as i32, y], c, format);
        }
        for i in (text_start + text.len())..style.width {
            let format = if i < v { style.filled } else { style.empty };
            self.put_char_formatted([x + i as i32, y], ' ', format);
        }
    }
}
