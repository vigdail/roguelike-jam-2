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
mod side_panel;
mod turn;
mod utils;

use crate::components::*;
use bevy::{prelude::*, window::PresentMode};
use bevy_ascii_terminal::{Pivot, StringFormat, Terminal, TerminalBundle, TerminalPlugin, Tile};
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
use resources::GameState;
use side_panel::{render_player_stats, render_visible_entities};
use turn::TurnPlugin;
use utils::{clear_undercursor, cursor_hint, Grayscale, UnderCursor};

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
        .add_state::<GameState>(GameState::Gameplay)
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
        .add_system(update_fov)
        .add_system(update_visibility.after(update_fov))
        .add_system(render_map.after(update_visibility).label("render_map"))
        .add_system(render_player_stats.chain(render_visible_entities))
        .add_system(render_log_panel.chain(render_hint_text))
        .add_system(toggle_inspector)
        .add_system_to_stage(CoreStage::First, clear_undercursor)
        .add_system_to_stage(CoreStage::PreUpdate, cursor_hint)
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
    tiles: Query<
        (
            &Tile,
            &Position,
            Option<&Visible>,
            Option<&Layer>,
            Option<&UnderCursor>,
        ),
        Or<(With<Revealed>, With<Visible>)>,
    >,
    mut terminal: Query<&mut Terminal, With<MapViewTerminal>>,
) {
    let mut terminal = terminal.single_mut();
    terminal.clear();
    let sorted_tiles = tiles
        .iter()
        .filter(|(_, position, _, _, _)| terminal.is_in_bounds([position.x, position.y]))
        .sorted_by(|a, b| a.3.cmp(&b.3))
        .map(|(tile, position, visible, _, under_cursor)| (tile, position, visible, under_cursor))
        .map(|(tile, position, visible, under_cursor)| {
            let tile = if under_cursor.is_some() {
                Tile {
                    glyph: tile.glyph,
                    fg_color: Color::BLACK,
                    bg_color: Color::YELLOW,
                }
            } else if visible.is_some() {
                *tile
            } else {
                tile.grayscale()
            };
            (tile, position)
        });

    for (tile, position) in sorted_tiles {
        terminal.put_tile([position.x, position.y], tile);
    }
}

fn keyboard_handling(
    mut commands: Commands,
    mut input: ResMut<Input<KeyCode>>,
    mut states: ResMut<State<GameState>>,
    mut players: Query<(Entity, &Position, &mut Energy), (With<Player>, With<TakingATurn>)>,
) {
    let (player, &player_pos, mut energy) = match players.get_single_mut() {
        Ok(players) => players,
        Err(_) => return,
    };

    let key = match input.get_just_pressed().next() {
        Some(key) => key,
        None => return,
    };

    let mut delta = Position::new(0, 0);
    match key {
        KeyCode::Numpad8 | KeyCode::Up => delta.y += 1,
        KeyCode::Numpad2 | KeyCode::Down => delta.y -= 1,
        KeyCode::Numpad4 | KeyCode::Left => delta.x -= 1,
        KeyCode::Numpad6 | KeyCode::Right => delta.x += 1,
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
        KeyCode::Numpad5 => {
            energy.0 = 0;
        }
        KeyCode::G => {
            commands.entity(player).insert(WantPickup);
        }
        KeyCode::I => {
            input.clear();
            states.push(GameState::Inventory).unwrap();
            return;
        }
        KeyCode::D => {
            input.clear();
            states.push(GameState::DropItemMenu).unwrap();
            return;
        }
        _ => return,
    }

    if delta != Position::default() {
        energy.0 = 0;
        commands.entity(player).insert(WantToMove {
            position: Position::new(player_pos.x + delta.x, player_pos.y + delta.y),
        });
    }
}

pub fn handle_want_to_move(
    mut commands: Commands,
    mut attack_events: EventWriter<AttackEvent>,
    mut move_events: EventWriter<MoveEvent>,
    map: Res<Map>,
    mut actors: Query<(Entity, &WantToMove)>,
    blocks: Query<Entity, With<Blocker>>,
    victims: Query<Entity, With<Health>>,
) {
    for (entity, to_move) in actors.iter_mut() {
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

pub fn movement(
    mut move_events: EventReader<MoveEvent>,
    mut actors: Query<(&mut Position, &mut Energy)>,
) {
    for event in move_events.iter() {
        if let Ok((mut position, mut energy)) = actors.get_mut(event.entity) {
            *position = event.position;
            energy.0 = 0;
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

fn render_hint_text(
    mut terminal: Query<&mut Terminal, With<LogTerminal>>,
    highlighted: Query<&Name, With<UnderCursor>>,
) {
    if let Some((mut terminal, name)) = terminal
        .get_single_mut()
        .ok()
        .zip(highlighted.get_single().ok())
    {
        let text = format!("You see {}", name);
        let x = 3;
        let y = terminal.height() as i32 - 1;
        let format = StringFormat::default().with_pivot(Pivot::BottomRight);
        terminal.put_string_formatted([x, y], &text, format);
    }
}
