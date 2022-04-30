#![allow(clippy::type_complexity)]
mod map;

use std::collections::HashSet;

use bevy::prelude::*;
use bevy_ascii_terminal::{Terminal, TerminalBundle, TerminalPlugin, Tile};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_tiled_camera::{TiledCameraBundle, TiledCameraPlugin};
use bracket_lib::prelude::{field_of_view, Point, BLACK, RGBA};
use itertools::Itertools;
use map::{Map, MapPlugin};

const LAYER_PLAYER: u32 = 4;
const LAYER_MAP: u32 = 0;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1280.0,
            height: 720.0,
            title: "Roguelike".to_string(),
            resizable: false,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(TerminalPlugin)
        .add_plugin(TiledCameraPlugin)
        .add_plugin(MapPlugin)
        .add_startup_system(setup_camera)
        .add_system(keyboard_handling)
        .add_system(movement)
        .add_system(update_fov)
        .add_system(update_visibility.after(update_fov))
        .add_system(render_map.after(update_visibility))
        .run();
}

#[derive(Component)]
pub struct MapViewTerminal;

#[derive(Component, Default, PartialEq, Eq, Clone, Copy, Ord, PartialOrd)]
pub struct Layer(pub u32);

#[derive(Component, Clone, Copy)]
pub struct BlockMove;

#[derive(Component, Clone, Copy)]
pub struct Opaque;

#[derive(Component, Clone, Copy)]
pub struct Revealed;

#[derive(Component, Clone, Copy)]
pub struct Visible;

#[derive(Component, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    fn new<T>(x: T, y: T) -> Self
    where
        T: TryInto<i32>,
    {
        Self {
            x: x.try_into().unwrap_or(0),
            y: y.try_into().unwrap_or(0),
        }
    }
}

impl From<Point> for Position {
    fn from(point: Point) -> Self {
        Self::from(&point)
    }
}

impl From<&Point> for Position {
    fn from(point: &Point) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<&Position> for Point {
    fn from(position: &Position) -> Self {
        Self {
            x: position.x,
            y: position.y,
        }
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct WantToMove {
    pub position: Position,
}

#[derive(Component)]
pub struct Fov {
    pub visible_tiles: HashSet<Position>,
    pub range: u32,
}

trait AsBracket<T> {
    fn as_bracket(&self) -> T;
}

impl AsBracket<RGBA> for Color {
    fn as_bracket(&self) -> RGBA {
        RGBA::from_f32(self.r(), self.g(), self.b(), self.a())
    }
}

trait AsBevy<T> {
    fn as_bevy(&self) -> T;
}

impl AsBevy<Color> for RGBA {
    fn as_bevy(&self) -> Color {
        Color::rgba(self.r, self.g, self.b, self.a)
    }
}

fn setup_camera(mut commands: Commands) {
    let size = [80, 45];
    let mut term_bundle = TerminalBundle::new().with_size(size);

    term_bundle.terminal.draw_border_single();

    commands.spawn_bundle(term_bundle).insert(MapViewTerminal);

    commands.spawn_bundle(
        TiledCameraBundle::new()
            .with_centered(true)
            .with_pixels_per_tile(8)
            .with_tile_count(size),
    );
}

fn render_map(
    tiles: Query<(
        &Tile,
        &Position,
        Option<&Revealed>,
        Option<&Visible>,
        Option<&Layer>,
    )>,
    mut terminal: Query<&mut Terminal, With<MapViewTerminal>>,
) {
    let mut terminal = terminal.single_mut();
    terminal.clear();
    let sorted_tiles = tiles
        .iter()
        .filter(|(_, _, revealed, visible, _)| visible.is_some() || revealed.is_some())
        .sorted_by(|a, b| a.4.cmp(&b.4))
        .map(|(tile, position, _, visible, _)| (tile, position, visible))
        .map(|(tile, position, visible)| {
            let tile = if visible.is_some() {
                *tile
            } else {
                // TODO: implement lerp Bevy colors insted of using this mess
                let fg_color = tile
                    .fg_color
                    .as_bracket()
                    .lerp(RGBA::named(BLACK), 0.6)
                    .as_bevy();
                let bg_color = tile
                    .bg_color
                    .as_bracket()
                    .lerp(RGBA::named(BLACK), 0.6)
                    .as_bevy();
                Tile {
                    glyph: tile.glyph,
                    fg_color,
                    bg_color,
                }
            };
            (tile, position)
        });

    for (tile, position) in sorted_tiles {
        if terminal.is_in_bounds([position.x, position.y]) {
            terminal.put_tile([position.x, position.y], tile);
        }
    }
}
fn keyboard_handling(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    players: Query<(Entity, &Position), With<Player>>,
) {
    let mut delta = Position::new(0, 0);
    if input.just_pressed(KeyCode::W) || input.just_pressed(KeyCode::Numpad8) {
        delta.y += 1;
    }
    if input.just_pressed(KeyCode::S) || input.just_pressed(KeyCode::Numpad2) {
        delta.y -= 1;
    }
    if input.just_pressed(KeyCode::A) || input.just_pressed(KeyCode::Numpad4) {
        delta.x -= 1;
    }
    if input.just_pressed(KeyCode::D) || input.just_pressed(KeyCode::Numpad6) {
        delta.x += 1;
    }

    if input.just_pressed(KeyCode::Numpad7) {
        delta.x -= 1;
        delta.y += 1;
    }
    if input.just_pressed(KeyCode::Numpad9) {
        delta.x += 1;
        delta.y += 1;
    }
    if input.just_pressed(KeyCode::Numpad1) {
        delta.x -= 1;
        delta.y -= 1;
    }
    if input.just_pressed(KeyCode::Numpad3) {
        delta.x += 1;
        delta.y -= 1;
    }

    if delta == Position::default() {
        return;
    }

    for (player, position) in players.iter() {
        commands.entity(player).insert(WantToMove {
            position: Position::new(position.x + delta.x, position.y + delta.y),
        });
    }
}

fn movement(
    mut commands: Commands,
    map: Res<Map>,
    mut units: Query<(Entity, &mut Position, &WantToMove)>,
    blocks: Query<&BlockMove>,
) {
    for (entity, mut position, target_position) in units.iter_mut() {
        let targets = map.at_position(&target_position.position);
        if targets.into_iter().any(|e| blocks.get(e).is_ok()) {
            info!(
                "Unable to move to: {}, {}",
                target_position.position.x, target_position.position.y
            );
        } else {
            *position = target_position.position;
        }
        commands.entity(entity).remove::<WantToMove>();
    }
}

fn update_visibility(
    mut commands: Commands,
    map: Res<Map>,
    fov: Query<&Fov, With<Player>>,
    visible: Query<Entity, With<Visible>>,
) {
    for entity in visible.iter() {
        commands.entity(entity).remove::<Visible>();
    }

    if let Ok(fov) = fov.get_single() {
        for visible in fov.visible_tiles.iter() {
            if let Some(entities) = map.tiles.get(visible) {
                for &entity in entities {
                    commands.entity(entity).insert(Visible);
                    commands.entity(entity).insert(Revealed);
                }
            };
        }
    }
}

fn update_fov(map: Res<Map>, mut units: Query<(&mut Fov, &Position), Changed<Position>>) {
    for (mut fov, position) in units.iter_mut() {
        fov.visible_tiles.clear();
        fov.visible_tiles =
            field_of_view(position.into(), fov.range.try_into().unwrap_or(0), &*map)
                .iter()
                .filter_map(|p| {
                    let pos = p.into();
                    if map.is_in_bounds(&pos) {
                        Some(pos)
                    } else {
                        None
                    }
                })
                .collect();
    }
}
