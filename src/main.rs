#![allow(clippy::type_complexity)]
mod combat;
mod components;
mod events;
mod map;
mod monster;
mod player;
mod states;
mod utils;

use crate::components::*;
use bevy::prelude::*;
use bevy_ascii_terminal::{Terminal, TerminalBundle, TerminalPlugin, Tile};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_tiled_camera::{TiledCameraBundle, TiledCameraPlugin};
use bracket_lib::prelude::field_of_view;
use combat::{track_dead, Health};
use events::{AttackEvent, MoveEvent};
use itertools::Itertools;
use map::{Map, MapPlugin};
use monster::MonsterPlugin;
use player::PlayerPlugin;
use states::GameState;
use utils::Grayscale;

const LAYER_MAP: u32 = 0;
const LAYER_MONSTER: u32 = 3;
const LAYER_PLAYER: u32 = 4;

fn main() {
    App::new()
        .add_state::<GameState>(GameState::WaitingInput)
        .add_event::<AttackEvent>()
        .add_event::<MoveEvent>()
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
        .add_plugin(PlayerPlugin)
        .add_plugin(MonsterPlugin)
        .add_startup_system(setup_camera)
        .add_system_set(
            SystemSet::on_update(GameState::WaitingInput).with_system(keyboard_handling),
        )
        .add_system(track_dead)
        .add_system(update_fov)
        .add_system(update_visibility.after(update_fov))
        .add_system(render_map.after(update_visibility))
        .run();
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
                tile.grayscale()
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
    mut input: ResMut<Input<KeyCode>>,
    mut states: ResMut<State<GameState>>,
    players: Query<(Entity, &Position), With<Player>>,
) {
    let just_pressed = input.get_just_pressed().next();
    if just_pressed.is_none() {
        return;
    }

    let key = just_pressed.cloned().unwrap();
    let mut delta = Position::new(0, 0);
    match key {
        KeyCode::W | KeyCode::Numpad8 | KeyCode::Up => delta.y += 1,
        KeyCode::S | KeyCode::Numpad2 | KeyCode::Down => delta.y -= 1,
        KeyCode::A | KeyCode::Numpad4 | KeyCode::Left => delta.x -= 1,
        KeyCode::D | KeyCode::Numpad6 | KeyCode::Right => delta.x += 1,
        KeyCode::Numpad7 => {
            delta.x -= 1;
            delta.y += 1;
        }
        KeyCode::Numpad9 => {
            delta.x += 1;
            delta.y += 1;
        }
        KeyCode::Numpad1 => {
            delta.x -= 1;
            delta.y -= 1;
        }
        KeyCode::Numpad3 => {
            delta.x += 1;
            delta.y -= 1;
        }
        _ => {}
    }
    if delta == Position::default() {
        return;
    }

    for (player, position) in players.iter() {
        commands.entity(player).insert(WantToMove {
            position: Position::new(position.x + delta.x, position.y + delta.y),
        });
    }

    input.reset(key);
    states.set(GameState::PlayerTurn).unwrap();
}

pub fn handle_want_to_move(
    mut commands: Commands,
    mut attack_events: EventWriter<AttackEvent>,
    mut move_events: EventWriter<MoveEvent>,
    map: Res<Map>,
    actors: Query<(Entity, &WantToMove)>,
    blocks: Query<Entity, With<BlockMove>>,
    victims: Query<Entity, With<Health>>,
) {
    for (entity, to_move) in actors.iter() {
        let at_position = map.tiles.get(&to_move.position);
        if at_position.is_none() {
            continue;
        }

        let at_position = at_position.unwrap();

        let victim = at_position
            .iter()
            .find(|&&e| victims.get(e).ok().is_some())
            .cloned();

        if let Some(victim) = victim {
            attack_events.send(AttackEvent {
                attacker: entity,
                target: victim,
            });
            commands.entity(entity).remove::<WantToMove>();
            continue;
        }

        let can_move = at_position.iter().all(|&e| blocks.get(e).ok().is_none());

        if can_move {
            move_events.send(MoveEvent {
                entity,
                position: to_move.position,
            });
        }
        commands.entity(entity).remove::<WantToMove>();
    }
}

pub fn movement(mut move_events: EventReader<MoveEvent>, mut actors: Query<&mut Position>) {
    for event in move_events.iter() {
        if let Ok(mut position) = actors.get_mut(event.entity) {
            *position = event.position;
        }
    }
}

pub fn update_visibility(
    mut commands: Commands,
    map: Res<Map>,
    fov: Query<&Fov, (With<Player>, Changed<Fov>)>,
    visible: Query<Entity, With<Visible>>,
    unrevealable: Query<Entity, With<Unrevealable>>,
) {
    if let Ok(fov) = fov.get_single() {
        for entity in visible.iter() {
            commands.entity(entity).remove::<Visible>();
        }
        for visible in fov.visible_tiles.iter() {
            if let Some(entities) = map.tiles.get(visible) {
                for &entity in entities {
                    commands.entity(entity).insert(Visible);
                    if unrevealable.get(entity).is_err() {
                        commands.entity(entity).insert(Revealed);
                    }
                }
            };
        }
    }
}

pub fn update_fov(map: Res<Map>, mut units: Query<(&mut Fov, &Position), Changed<Position>>) {
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
