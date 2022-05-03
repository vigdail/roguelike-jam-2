use bevy::prelude::*;
use bevy_ascii_terminal::Tile;

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
