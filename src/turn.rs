use bevy::prelude::*;

use crate::{
    combat::{combat, track_dead},
    handle_want_to_move, movement,
    resources::{CurrentTurn, GameState},
};

#[derive(Default)]
pub struct NextState(pub Option<GameState>);

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextState>()
            .add_system_set(
                SystemSet::on_enter(GameState::Turn)
                    .with_system(handle_want_to_move)
                    .with_system(combat.after(handle_want_to_move))
                    .with_system(track_dead.after(combat))
                    .with_system(movement.after(combat))
                    .with_system(end_turn.after(movement)),
            )
            .add_system_to_stage(CoreStage::Last, change_state);
    }
}

pub fn end_turn(
    states: Res<State<GameState>>,
    mut turn: ResMut<CurrentTurn>,
    mut next_state: ResMut<NextState>,
) {
    let next = match states.current() {
        GameState::WaitingInput => GameState::Turn,
        GameState::MonsterAi => GameState::Turn,
        GameState::Turn => {
            let next = match *turn {
                CurrentTurn::Player => GameState::MonsterAi,
                CurrentTurn::Monster => GameState::WaitingInput,
            };
            turn.change();
            next
        }
        _ => return,
    };

    next_state.0 = Some(next);
}

fn change_state(mut states: ResMut<State<GameState>>, mut next_state: ResMut<NextState>) {
    if let Some(next) = next_state.0.take() {
        states.set(next).unwrap();
    }
}
