use bevy::prelude::*;
use bevy_ascii_terminal::{CharFormat, Terminal, Tile};
use bevy_tiled_camera::TiledProjection;

use crate::{
    components::{Layer, Position, Revealed, Visible},
    map::Map,
    LOG_PANEL_SIZE, STATUS_PANEL_SIZE, WINDOW_SIZE,
};

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

pub fn cursor_hint(
    input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    map: Res<Map>,
    q_camera: Query<(&Camera, &GlobalTransform, &TiledProjection)>,
    names: Query<(&Name, &Layer), Or<(With<Visible>, With<Revealed>)>>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(pos) = window.cursor_position() {
        for (cam, cam_transform, proj) in q_camera.iter() {
            if let Some(p) = proj.screen_to_world(cam, &windows, cam_transform, pos) {
                if let Some(mut p) = proj.world_to_tile(cam_transform, p) {
                    p.x += WINDOW_SIZE[0] as i32 / 2 - STATUS_PANEL_SIZE[0] as i32;
                    p.y += WINDOW_SIZE[1] as i32 / 2 - LOG_PANEL_SIZE[1] as i32;
                    let position = Position::new(p.x, p.y);
                    if input.just_pressed(MouseButton::Left) {
                        if !map.is_in_bounds(&position) {
                            return;
                        }
                        if let Some(name) = map
                            .at_position(&position)
                            .into_iter()
                            .filter_map(|entity| names.get(entity).ok())
                            .max_by(|a, b| a.1.cmp(b.1))
                            .map(|(name, _)| name)
                        {
                            info!("You see: {}", name);
                        }
                    }

                    return;
                }
            }
        }
    }
}
