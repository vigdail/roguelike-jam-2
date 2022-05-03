use bevy::prelude::*;
use bevy_ascii_terminal::Tile;

use crate::{
    combat::Health,
    components::{Layer, Position, Unrevealable},
    events::{PickupEvent, WantPickup},
    log::GameLog,
    map::Map,
    resources::GameState,
    LAYER_ITEM,
};

#[derive(Component)]
pub struct Item;

#[derive(Component)]
pub struct Potion {
    pub heal_amount: u32,
}

#[derive(Component, Debug, Clone)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct WantUseItem {
    pub item: Entity,
}

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PickupEvent>().add_system_set(
            SystemSet::on_enter(GameState::Turn)
                .with_system(handle_want_pickup)
                .with_system(handle_pickup.after(handle_want_pickup))
                .with_system(handle_use_item),
        );
    }
}

pub fn health_potion(commands: &mut Commands, position: Position) -> Entity {
    commands
        .spawn()
        .insert(Item)
        .insert(Potion { heal_amount: 8 })
        .insert(Tile {
            glyph: '¡',
            fg_color: Color::YELLOW,
            bg_color: Color::NONE,
        })
        .insert(Name::new("Healing potion"))
        .insert(position)
        .insert(Layer(LAYER_ITEM))
        .insert(Unrevealable)
        .id()
}

pub fn handle_want_pickup(
    mut commands: Commands,
    map: Res<Map>,
    actors: Query<(Entity, &Position), With<WantPickup>>,
    mut pickup_events: EventWriter<PickupEvent>,
    items: Query<&Item>,
) {
    for (collector, position) in actors.iter() {
        if let Some(&item) = map
            .at_position(position)
            .iter()
            .find(|&&e| items.contains(e))
        {
            pickup_events.send(PickupEvent {
                collected_by: collector,
                item,
            });
            commands.entity(collector).remove::<WantPickup>();
        }
    }
}

pub fn handle_pickup(
    mut commands: Commands,
    mut game_log: ResMut<GameLog>,
    mut events: EventReader<PickupEvent>,
    names: Query<&Name>,
) {
    for event in events.iter() {
        commands
            .entity(event.item)
            .remove::<Position>()
            .insert(InBackpack {
                owner: event.collected_by,
            });

        let collector_name = names.get(event.collected_by).cloned().unwrap_or_default();
        let item_name = names.get(event.item).cloned().unwrap_or_default();
        game_log.push(format!("{} pickups {}", collector_name, item_name));
    }
}

fn handle_use_item(
    mut commands: Commands,
    mut game_log: ResMut<GameLog>,
    mut to_use: Query<(Entity, &WantUseItem, &Name, Option<&mut Health>)>,
    items: Query<&Potion>,
) {
    for (entity, to_use, name, health) in to_use.iter_mut() {
        if let Some((potion, mut health)) = items.get(to_use.item).ok().zip(health) {
            health.current = health.max.min(health.current + potion.heal_amount);
            commands.entity(to_use.item).remove::<InBackpack>();
            game_log.push(format!(
                "{} drinks potion, restores: {} hp",
                name, potion.heal_amount
            ));
        }

        commands.entity(entity).remove::<WantUseItem>();
    }
}
