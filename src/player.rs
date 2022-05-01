use bevy::prelude::*;

use crate::{combat::combat, handle_want_to_move, movement, states::GameState};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::PlayerTurn)
                .with_system(handle_want_to_move)
                .with_system(combat.after(handle_want_to_move))
                .with_system(movement.after(handle_want_to_move))
                .with_system(end_turn.after(movement)),
        );
    }
}

pub fn end_turn(mut states: ResMut<State<GameState>>) {
    states.set(GameState::MonsterTurn).unwrap();
}
