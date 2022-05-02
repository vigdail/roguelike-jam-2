use bevy::prelude::*;

use crate::components::Position;

pub struct AttackEvent {
    pub attacker: Entity,
    pub target: Entity,
}

pub struct MoveEvent {
    pub entity: Entity,
    pub position: Position,
}

#[derive(Component)]
pub struct WantPickup;

pub struct PickupEvent {
    pub collected_by: Entity,
    pub item: Entity,
}
