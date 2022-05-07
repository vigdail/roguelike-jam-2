use bevy::prelude::*;
use bevy_ascii_terminal::{CharFormat, Terminal, Tile};
use itertools::Itertools;

use crate::{
    combat::Health,
    components::{Fov, Player, Position, StatusTerminal},
    items::Item,
    map::Map,
    monster::Monster,
    utils::{TitleBarStyle, UiUtils},
    STATUS_PANEL_SIZE,
};

pub fn render_player_stats(
    mut terminal: Query<&mut Terminal, With<StatusTerminal>>,
    player: Query<&Health, With<Player>>,
) {
    if let Ok(mut terminal) = terminal.get_single_mut() {
        terminal.clear();
        terminal.draw_box_single([0, 0], STATUS_PANEL_SIZE);
        terminal.draw_box_single(
            [0, STATUS_PANEL_SIZE[1] as i32 - 5],
            [STATUS_PANEL_SIZE[0], 5],
        );
        if let Ok(health) = player.get_single() {
            terminal.draw_titled_bar(
                [1, STATUS_PANEL_SIZE[1] as i32 - 4],
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

pub fn render_visible_entities(
    map: Res<Map>,
    player: Query<(&Fov, &Position), With<Player>>,
    monsters: Query<(&Name, &Health, &Tile), With<Monster>>,
    items: Query<(&Name, &Tile), With<Item>>,
    mut terminal: Query<&mut Terminal, With<StatusTerminal>>,
) {
    let (fov, player_pos) = match player.get_single() {
        Ok(player) => player,
        Err(_) => return,
    };

    let y_offset = 7;
    let mut y = STATUS_PANEL_SIZE[1] as i32 - y_offset;
    let max_monsters = STATUS_PANEL_SIZE[1] - y as u32 / 3;

    let visible_entities = fov
        .visible_tiles
        .iter()
        .sorted_by(|a, b| {
            player_pos
                .distance_squared(a)
                .cmp(&player_pos.distance_squared(b))
        })
        .map(|p| map.at_position(p))
        .filter(|v| !v.is_empty())
        .flat_map(|v| v.into_iter())
        .collect::<Vec<_>>();

    let monsters = visible_entities
        .iter()
        .filter_map(|entity| monsters.get(*entity).ok())
        .take(max_monsters as usize)
        .collect::<Vec<_>>();

    let max_items = (STATUS_PANEL_SIZE[1] as usize - monsters.len() * 3 - y_offset as usize) / 2;
    let items = visible_entities
        .iter()
        .filter_map(|entity| items.get(*entity).ok())
        .take(max_items);

    if let Ok(mut terminal) = terminal.get_single_mut() {
        for (name, health, tile) in monsters {
            terminal.put_tile([1, y], *tile);
            terminal.put_string([3, y], name);
            terminal.draw_titled_bar(
                [1, y - 1],
                "Health",
                health.current as i32,
                health.max as i32,
                TitleBarStyle {
                    width: STATUS_PANEL_SIZE[0] as usize - 2,
                    filled: CharFormat::new(Color::WHITE, Color::BLUE),
                    empty: CharFormat::new(Color::WHITE, Color::PURPLE),
                },
            );
            y -= 3;
        }

        for (name, tile) in items {
            terminal.put_tile([1, y], *tile);
            terminal.put_string([3, y], name);
            y -= 2;
        }
    }
}
