use bevy::{prelude::*, utils::HashMap};
use bevy_ascii_terminal::{CharFormat, StringFormat, Terminal};

use crate::{
    components::{MapViewTerminal, Player},
    items::{InBackpack, WantDropItem, WantUseItem},
    resources::GameState,
};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Inventory).with_system(
                render_inventory
                    .chain(handle_inventory_input)
                    .after("render_map"),
            ),
        )
        .add_system_set(
            SystemSet::on_update(GameState::DropItemMenu).with_system(
                render_drop_menu
                    .chain(handle_drop_input)
                    .after("render_map"),
            ),
        );
    }
}

fn handle_inventory_input(
    backpack: In<HashMap<char, Entity>>,
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut char_evr: EventReader<ReceivedCharacter>,
    mut states: ResMut<State<GameState>>,
    player: Query<Entity, With<Player>>,
) {
    let player = match player.get_single() {
        Ok(player) => player,
        Err(_) => return,
    };

    let key = match input.get_just_pressed().next() {
        Some(key) => key,
        None => return,
    };

    if key == &KeyCode::Escape {
        states.pop().unwrap()
    };

    if let Some(&entity) = char_evr
        .iter()
        .next()
        .map(|r| r.char)
        .and_then(|c| backpack.0.get(&c))
    {
        commands.entity(player).insert(WantUseItem { item: entity });
        states.pop().unwrap();
    }
}

fn handle_drop_input(
    backpack: In<HashMap<char, Entity>>,
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut char_evr: EventReader<ReceivedCharacter>,
    mut states: ResMut<State<GameState>>,
    player: Query<Entity, With<Player>>,
) {
    let player = match player.get_single() {
        Ok(player) => player,
        Err(_) => return,
    };

    let key = match input.get_just_pressed().next() {
        Some(key) => key,
        None => return,
    };

    if key == &KeyCode::Escape {
        states.pop().unwrap()
    };

    if let Some(&entity) = char_evr
        .iter()
        .next()
        .map(|r| r.char)
        .and_then(|c| backpack.0.get(&c))
    {
        commands
            .entity(player)
            .insert(WantDropItem { item: entity });
        states.pop().unwrap();
    }
}

fn render_inventory(
    mut terminal: Query<&mut Terminal, With<MapViewTerminal>>,
    player: Query<Entity, With<Player>>,
    backpack: Query<(Entity, &Name, &InBackpack)>,
) -> HashMap<char, Entity> {
    let terminal = terminal.single_mut();
    let player = player.get_single().ok();
    let backpack = backpack
        .iter()
        .filter_map(|(entity, name, in_backpack)| {
            let player = player?;
            if player == in_backpack.owner {
                Some((entity, name))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    draw_items_menu("Inventory", &backpack, terminal);

    ('a'..'z')
        .zip(backpack.into_iter().map(|(entity, _)| entity))
        .collect()
}

fn render_drop_menu(
    mut terminal: Query<&mut Terminal, With<MapViewTerminal>>,
    player: Query<Entity, With<Player>>,
    backpack: Query<(Entity, &Name, &InBackpack)>,
) -> HashMap<char, Entity> {
    let terminal = terminal.single_mut();
    let player = player.get_single().ok();
    let backpack = backpack
        .iter()
        .filter_map(|(entity, name, in_backpack)| {
            let player = player?;
            if player == in_backpack.owner {
                Some((entity, name))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    draw_items_menu("Drop item", &backpack, terminal);

    ('a'..'z')
        .zip(backpack.into_iter().map(|(entity, _)| entity))
        .collect()
}

fn draw_items_menu(title: &str, backpack: &[(Entity, &Name)], mut terminal: Mut<Terminal>) {
    let item_count = backpack.len() as i32;
    let width = 31;
    let height = 4 + item_count;
    let x = (80 - width) / 2;
    let y = (45 - height) / 2;
    terminal.clear_box([x, y], [width as u32, height as u32]);
    terminal.draw_box_double([x, y], [width as u32, height as u32]);
    terminal.put_string_formatted(
        [x + 3, y + height - 1],
        title,
        StringFormat::colors(Color::YELLOW, Color::NONE),
    );
    terminal.put_string_formatted(
        [x + 3, y],
        "ESCAPE to cancel",
        StringFormat::colors(Color::YELLOW, Color::NONE),
    );
    for (i, (letter, name)) in ('a'..'z')
        .zip(backpack.iter().map(|(_, name)| name))
        .enumerate()
    {
        let x = x + 2;
        let y = y + height - i as i32 - 3;
        terminal.put_char([x, y], '(');
        terminal.put_char_formatted(
            [x + 1, y],
            letter,
            CharFormat::new(Color::YELLOW, Color::NONE),
        );
        terminal.put_string([x + 2, y], &format!("): {}", name));
    }
}
