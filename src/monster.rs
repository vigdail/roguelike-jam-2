use bevy::prelude::*;
use bevy_ascii_terminal::Tile;
use bracket_lib::prelude::{
    a_star_search, Algorithm2D, Bresenham, DistanceAlg, RandomNumberGenerator,
};

use crate::{
    combat::{Attack, CombatStatsBundle, Health},
    components::{Blocker, Energy, MovingEntityBundle, Player, TakingATurn, WantToMove},
    map::Map,
    Layer, Position, Unrevealable, LAYER_MONSTER,
};

#[derive(Component)]
pub struct Monster;

#[derive(Bundle)]
pub struct MonsterBundle {
    pub monster: Monster,
    pub name: Name,
    pub unrevealable: Unrevealable,
    pub blocker: Blocker,
    pub tile: Tile,
    pub layer: Layer,
    #[bundle]
    pub combat_stats: CombatStatsBundle,
    #[bundle]
    pub moving: MovingEntityBundle,
}

impl Default for MonsterBundle {
    fn default() -> Self {
        Self {
            monster: Monster,
            name: "Goblin".into(),
            unrevealable: Unrevealable,
            blocker: Blocker,
            tile: Tile {
                glyph: 'g',
                fg_color: Color::RED,
                bg_color: Color::BLACK,
            },
            layer: Layer(LAYER_MONSTER),
            combat_stats: CombatStatsBundle {
                health: Health::new(10),
                attack: Attack::new((1, 4)),
            },
            moving: MovingEntityBundle::new(40),
        }
    }
}

pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(monster_ai);
    }
}

pub fn spawn_monster(commands: &mut Commands, position: Position) -> Entity {
    let mut rng = RandomNumberGenerator::new();
    let roll = rng.roll_dice(1, 6);
    let (glyph, name, attack, speed) = match roll {
        1 => ('o', "Orc", Attack::new((1, 6)), 30),
        _ => ('g', "Goblin", Attack::new((1, 4)), 45),
    };
    let monster = MonsterBundle {
        monster: Monster,
        name: name.into(),
        tile: Tile {
            glyph,
            fg_color: Color::RED,
            bg_color: Color::BLACK,
        },
        combat_stats: CombatStatsBundle {
            health: Health::new(10),
            attack,
        },
        moving: MovingEntityBundle::new(speed).with_position(position),
        ..Default::default()
    };
    commands.spawn_bundle(monster).id()
}

pub fn monster_ai(
    mut commands: Commands,
    map: Res<Map>,
    player: Query<&Position, With<Player>>,
    mut monsters: Query<(Entity, &Position, &mut Energy), (With<Monster>, With<TakingATurn>)>,
) {
    let player_pos = match player.get_single() {
        Ok(pos) => pos,
        Err(_) => return,
    };
    let vision_distance_squared = 36.0; // TODO
    for (monster_entity, monster_pos, mut energy) in monsters.iter_mut() {
        energy.0 = 0;
        let player_pos = player_pos.into();
        let monster_pos = monster_pos.into();
        if DistanceAlg::PythagorasSquared.distance2d(player_pos, monster_pos)
            > vision_distance_squared
        {
            continue;
        }
        let mut line = Bresenham::new(player_pos, monster_pos);
        if line.any(|p| map.opaque.contains(&p.into())) {
            continue;
        }

        let path = a_star_search(
            map.point2d_to_index(monster_pos) as i32,
            map.point2d_to_index(player_pos) as i32,
            &*map,
        );

        if path.success && path.steps.len() > 1 {
            if let Some(position) = map.idx_position(path.steps[1]) {
                commands
                    .entity(monster_entity)
                    .insert(WantToMove { position });
            }
        }
    }
}
