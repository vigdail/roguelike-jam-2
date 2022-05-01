use bevy::prelude::*;
use bevy_ascii_terminal::Tile;
use bracket_lib::prelude::RandomNumberGenerator;

use crate::{
    components::BlockMove, movement, states::GameState, Layer, Position, Unrevealable,
    LAYER_MONSTER,
};

#[derive(Component)]
pub struct Monster;

pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::MonsterTurn)
                .with_system(monster_ai)
                .with_system(movement.after(monster_ai))
                .with_system(end_turn.after(monster_ai)),
        );
    }
}

pub fn spawn_monster(commands: &mut Commands, position: &Position) -> Entity {
    let mut rng = RandomNumberGenerator::new();
    let roll = rng.roll_dice(1, 6);
    let (glyph, name) = match roll {
        1 => ('o', "Ogre"),
        _ => ('g', "Goblin"),
    };
    commands
        .spawn()
        .insert(Tile {
            glyph,
            fg_color: Color::RED,
            bg_color: Color::BLACK,
        })
        .insert(Monster)
        .insert(*position)
        .insert(Layer(LAYER_MONSTER))
        .insert(Unrevealable)
        .insert(Name::new(name))
        .insert(BlockMove)
        .id()
}

pub fn monster_ai(monsters: Query<&Name, With<Monster>>) {
    for name in monsters.iter() {
        info!("{} shouts", name);
    }
}

pub fn end_turn(mut states: ResMut<State<GameState>>) {
    states.set(GameState::WaitingInput).unwrap();
}
