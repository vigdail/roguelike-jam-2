use bevy::prelude::*;
use bevy_ascii_terminal::Tile;

use crate::{
    combat::{Attack, CombatStatsBundle, Health},
    components::{Fov, Layer, Player, Position},
    LAYER_PLAYER,
};

pub fn spawn_player(commands: &mut Commands, position: Position) -> Entity {
    commands
        .spawn()
        .insert(Player)
        .insert(Tile {
            glyph: '@',
            fg_color: Color::WHITE,
            bg_color: Color::rgba(1.0, 1.0, 1.0, 0.0),
        })
        .insert(position)
        .insert(Name::new("Player"))
        .insert(Layer(LAYER_PLAYER))
        .insert(Fov::new(8))
        .insert_bundle(CombatStatsBundle {
            health: Health::new(20),
            attack: Attack::new((1, 6)),
        })
        .id()
}
