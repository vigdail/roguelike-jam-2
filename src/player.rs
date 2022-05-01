use bevy::prelude::*;

use crate::{movement, states::GameState};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::PlayerTurn)
                .with_system(movement)
                .with_system(end_turn.after(movement)),
        );
    }
}

pub fn end_turn(mut states: ResMut<State<GameState>>) {
    states.set(GameState::MonsterTurn).unwrap();
}
