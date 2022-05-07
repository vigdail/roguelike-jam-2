use bevy::prelude::*;
use bevy_ascii_terminal::Tile;

use crate::{
    combat::{Attack, CombatStatsBundle, Health},
    components::{Fov, Layer, MovingEntityBundle, Player, Position},
    LAYER_PLAYER,
};

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub name: Name,
    pub fov: Fov,
    pub tile: Tile,
    pub layer: Layer,
    #[bundle]
    pub combat_stats: CombatStatsBundle,
    #[bundle]
    pub moving: MovingEntityBundle,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            name: "Player".into(),
            fov: Fov::new(8),
            tile: Tile {
                glyph: '@',
                fg_color: Color::WHITE,
                bg_color: Color::rgba(1.0, 1.0, 1.0, 0.0),
            },
            layer: Layer(LAYER_PLAYER),
            combat_stats: CombatStatsBundle {
                health: Health::new(20),
                attack: Attack::new((1, 6)),
            },
            moving: MovingEntityBundle::new(50),
        }
    }
}

pub fn spawn_player(commands: &mut Commands, position: Position) -> Entity {
    let player = PlayerBundle {
        moving: MovingEntityBundle::new(50).with_position(position),
        ..default()
    };

    commands.spawn_bundle(player).id()
}
