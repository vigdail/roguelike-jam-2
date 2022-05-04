#![allow(clippy::type_complexity)]
mod combat;
mod components;
mod events;
mod inventory;
mod items;
mod log;
mod map;
mod map_tile;
mod monster;
mod player;
mod resources;
mod turn;
mod utils;

use crate::components::*;
use bevy::{prelude::*, window::PresentMode};
use bevy_ascii_terminal::{CharFormat, Terminal, TerminalBundle, TerminalPlugin, Tile};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_tiled_camera::{TiledCameraBundle, TiledCameraPlugin};
use bracket_lib::prelude::field_of_view_set;
use combat::{CombatPlugin, Health};
use events::{AttackEvent, MoveEvent, WantPickup};
use inventory::InventoryPlugin;
use items::ItemPlugin;
use itertools::Itertools;
use log::GameLog;
use map::{Map, MapPlugin};
use monster::MonsterPlugin;
use resources::{CurrentTurn, GameState};
use turn::TurnPlugin;
use utils::{Grayscale, TitleBarStyle, UiUtils};

const LAYER_MAP: u32 = 0;
const LAYER_ITEM: u32 = 2;
const LAYER_MONSTER: u32 = 3;
const LAYER_PLAYER: u32 = 4;

const WINDOW_SIZE: [u32; 2] = [80, 45];
const LOG_PANEL_SIZE: [u32; 2] = [80, 6];
const STATUS_PANEL_SIZE: [u32; 2] = [22, WINDOW_SIZE[1] - LOG_PANEL_SIZE[1]];
const MAP_SIZE: [u32; 2] = [
    WINDOW_SIZE[0] - STATUS_PANEL_SIZE[0],
    WINDOW_SIZE[1] - LOG_PANEL_SIZE[1],
];

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
            present_mode: PresentMode::Mailbox,
            ..default()
        })
        .init_resource::<GameLog>()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(CurrentTurn::Player)
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(TerminalPlugin)
        .add_plugin(TiledCameraPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(ItemPlugin)
        .add_plugin(TurnPlugin)
        .add_plugin(MonsterPlugin)
        .add_plugin(CombatPlugin)
        .add_plugin(InventoryPlugin)
        .add_startup_system(setup_camera)
        .add_system_set(
            SystemSet::on_update(GameState::WaitingInput).with_system(keyboard_handling),
        )
        .add_system(update_fov)
        .add_system(update_visibility.after(update_fov))
        .add_system(render_map.after(update_visibility).label("render_map"))
        .add_system(render_status_panel)
        .add_system(render_log_panel)
        .add_system(toggle_inspector)
        .run();
}

fn toggle_inspector(
    input: ResMut<Input<KeyCode>>,
    mut inspector_params: ResMut<WorldInspectorParams>,
) {
    if input.just_pressed(KeyCode::Space) {
        inspector_params.enabled = !inspector_params.enabled;
    }
}

fn setup_camera(mut commands: Commands) {
    let mut map_terminal = TerminalBundle::new().with_size(MAP_SIZE);
    map_terminal.renderer.terminal_pivot.0 = Vec2::new(1.0, 1.0);
    map_terminal.transform.translation = Vec3::new(
        WINDOW_SIZE[0] as f32 / 2.0,
        WINDOW_SIZE[1] as f32 / 2.0,
        0.0,
    );
    commands.spawn_bundle(map_terminal).insert(MapViewTerminal);

    let mut status_terminal = TerminalBundle::new().with_size(STATUS_PANEL_SIZE);
    status_terminal.renderer.terminal_pivot.0 = Vec2::new(0.0, 1.0);
    status_terminal.transform.translation = Vec3::new(
        WINDOW_SIZE[0] as f32 / -2.0,
        WINDOW_SIZE[1] as f32 / 2.0,
        0.0,
    );
    commands
        .spawn_bundle(status_terminal)
        .insert(StatusTerminal);

    let mut logs_terminal = TerminalBundle::new().with_size(LOG_PANEL_SIZE);
    logs_terminal.renderer.terminal_pivot.0 = Vec2::new(0.0, 0.0);
    logs_terminal.transform.translation = Vec3::new(
        WINDOW_SIZE[0] as f32 / -2.0,
        WINDOW_SIZE[1] as f32 / -2.0,
        0.0,
    );
    commands.spawn_bundle(logs_terminal).insert(LogTerminal);

    commands.spawn_bundle(
        TiledCameraBundle::new()
            .with_centered(true)
            .with_pixels_per_tile(8)
            .with_tile_count(WINDOW_SIZE),
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
    let players = players.get_single();
    if players.is_err() {
        return;
    }

    let (player, &player_pos) = players.unwrap();

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
        KeyCode::Numpad5 => {}
        KeyCode::G => {
            commands.entity(player).insert(WantPickup);
        }
        KeyCode::I => {
            input.clear();
            states.push(GameState::Inventory).unwrap();
            return;
        }
        _ => return,
    }

    if delta != Position::default() {
        commands.entity(player).insert(WantToMove {
            position: Position::new(player_pos.x + delta.x, player_pos.y + delta.y),
        });
    }

    input.clear();
    states.set(GameState::Turn).unwrap();
}

pub fn handle_want_to_move(
    mut commands: Commands,
    mut attack_events: EventWriter<AttackEvent>,
    mut move_events: EventWriter<MoveEvent>,
    map: Res<Map>,
    actors: Query<(Entity, &WantToMove)>,
    blocks: Query<Entity, With<Blocker>>,
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
            field_of_view_set(position.into(), fov.range.try_into().unwrap_or(0), &*map)
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

fn render_status_panel(
    mut terminal: Query<&mut Terminal, With<StatusTerminal>>,
    player: Query<&Health, With<Player>>,
) {
    if let Ok(mut terminal) = terminal.get_single_mut() {
        terminal.clear();
        terminal.draw_box_single([0, 0], STATUS_PANEL_SIZE);
        if let Ok(health) = player.get_single() {
            let x = 0;
            terminal.draw_titled_bar(
                [x + 1, STATUS_PANEL_SIZE[1] as i32 - 4],
                &format!("HP: {}/{}", health.current, health.max),
                health.current as i32,
                health.max as i32,
                TitleBarStyle {
                    width: STATUS_PANEL_SIZE[0] as usize - 2,
                    filled: CharFormat::new(Color::WHITE, Color::RED),
                    empty: CharFormat::new(Color::WHITE, Color::MAROON),
                },
            );
        }
    }
}

fn render_log_panel(mut terminal: Query<&mut Terminal, With<LogTerminal>>, game_log: Res<GameLog>) {
    if let Ok(mut terminal) = terminal.get_single_mut() {
        terminal.clear();
        terminal.draw_box_single([0, 0], LOG_PANEL_SIZE);

        let count = (LOG_PANEL_SIZE[1] - 2) as usize;
        game_log
            .entries()
            .iter()
            .rev()
            .take(count)
            .rev()
            .enumerate()
            .for_each(|(i, log)| {
                terminal.put_string([2, (count - i) as i32], log);
            });
    }
}
