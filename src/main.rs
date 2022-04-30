mod map;

use bevy::prelude::*;
use bevy_ascii_terminal::{Terminal, TerminalBundle, TerminalPlugin, Tile};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_tiled_camera::{TiledCameraBundle, TiledCameraPlugin};
use bracket_lib::prelude::Point;
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
        .add_system(render_map)
        .add_system(keyboard_handling)
        .add_system(movement)
        .run();
}

#[derive(Component)]
pub struct MapViewTerminal;

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

#[derive(Component, Default, PartialEq, Eq, Clone, Copy, Ord, PartialOrd)]
pub struct Layer(pub u32);

#[derive(Component, Clone, Copy)]
pub struct BlockMove;

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
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

#[derive(Component)]
pub struct Player;

fn render_map(
    tiles: Query<(&Tile, &Position, Option<&Layer>)>,
    mut terminal: Query<&mut Terminal, With<MapViewTerminal>>,
) {
    let mut terminal = terminal.single_mut();
    terminal.clear();
    let sorted_tiles = tiles
        .iter()
        .sorted_by(|a, b| a.2.cmp(&b.2))
        .map(|(tile, position, _)| (tile, position));

    for (tile, position) in sorted_tiles {
        if terminal.is_in_bounds([position.x, position.y]) {
            terminal.put_tile([position.x, position.y], *tile);
        }
    }
}

#[derive(Component)]
pub struct WantToMove {
    pub position: Position,
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
