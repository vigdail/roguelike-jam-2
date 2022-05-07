use bevy::prelude::*;

use crate::{
    combat::{combat, track_dead},
    components::{Actor, Energy, Speed, TakingATurn},
    handle_want_to_move, keyboard_handling, movement,
};

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, turn_begin)
            .add_system_to_stage(CoreStage::PostUpdate, turn_end)
            .add_system(keyboard_handling)
            .add_system(handle_want_to_move)
            .add_system(combat.after(handle_want_to_move))
            .add_system(movement.after(combat))
            .add_system_to_stage(CoreStage::PostUpdate, track_dead);
    }
}

fn turn_begin(
    mut commands: Commands,
    mut q_waiting_actors: Query<(Entity, &mut Energy, &Speed), (With<Actor>, Without<TakingATurn>)>,
    q_acting_actors: Query<&Actor, (With<Energy>, With<Speed>, With<TakingATurn>)>,
) {
    if !q_acting_actors.is_empty() {
        return;
    }

    let mut done = false;
    while !done {
        for (entity, mut energy, speed) in q_waiting_actors.iter_mut() {
            energy.0 += speed.0;

            if energy.0 >= 100 {
                done = true;
                commands.entity(entity).insert(TakingATurn);
            }
        }
    }
}

fn turn_end(
    mut commands: Commands,
    q_actors: Query<(Entity, &Energy), (With<Actor>, With<TakingATurn>)>,
) {
    for (entity, energy) in q_actors.iter() {
        if energy.0 < 100 {
            commands.entity(entity).remove::<TakingATurn>();
        }
    }
}
