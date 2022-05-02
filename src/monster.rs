use bevy::prelude::*;
use bevy_ascii_terminal::Tile;
use bracket_lib::prelude::{
    a_star_search, Algorithm2D, Bresenham, DistanceAlg, RandomNumberGenerator,
};

use crate::{
    combat::{Attack, CombatStatsBundle, Health},
    components::{BlockMove, Player, WantToMove},
    map::Map,
    resources::GameState,
    turn::end_turn,
    Layer, Position, Unrevealable, LAYER_MONSTER,
};

#[derive(Component)]
pub struct Monster;

pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::MonsterAi)
                .with_system(monster_ai)
                .with_system(end_turn.after(monster_ai)),
        );
    }
}

pub fn spawn_monster(commands: &mut Commands, position: Position) -> Entity {
    let mut rng = RandomNumberGenerator::new();
    let roll = rng.roll_dice(1, 6);
    let (glyph, name, attack) = match roll {
        1 => ('o', "Orc", Attack::new((1, 6))),
        _ => ('g', "Goblin", Attack::new((1, 4))),
    };
    commands
        .spawn()
        .insert(Tile {
            glyph,
            fg_color: Color::RED,
            bg_color: Color::BLACK,
        })
        .insert(Monster)
        .insert(position)
        .insert(Layer(LAYER_MONSTER))
        .insert(Unrevealable)
        .insert(Name::new(name))
        .insert(BlockMove)
        .insert_bundle(CombatStatsBundle {
            health: Health::new(10),
            attack,
        })
        .id()
}

pub fn monster_ai(
    mut commands: Commands,
    map: Res<Map>,
    players: Query<&Position, With<Player>>,
    monsters: Query<(Entity, &Position), With<Monster>>,
) {
    let player_pos = players.get_single();
    if player_pos.is_err() {
        return;
    }
    let player_pos = player_pos.unwrap();
    let vision_distance_squared = 36.0; // TODO
    for (monster_entity, monster_pos) in monsters.iter() {
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
