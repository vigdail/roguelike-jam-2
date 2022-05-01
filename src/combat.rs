use bevy::prelude::*;
use bracket_lib::prelude::RandomNumberGenerator;

use crate::events::AttackEvent;

#[derive(Component, Clone, Copy)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

impl Health {
    pub fn new(amount: u32) -> Self {
        Self {
            current: amount,
            max: amount,
        }
    }
    pub fn take_damage<T: TryInto<u32>>(&mut self, damage: T) {
        let damage = damage.try_into().unwrap_or(0);
        self.current = self.current.saturating_sub(damage);
    }

    pub fn is_dead(&self) -> bool {
        self.current == 0
    }
}

pub fn combat(
    mut attack_events: EventReader<AttackEvent>,
    attackers: Query<&Name>,
    mut victims: Query<(&mut Health, Option<&Name>)>,
) {
    let mut rng = RandomNumberGenerator::new();
    for event in attack_events.iter() {
        let attacker = attackers
            .get(event.attacker)
            .ok()
            .cloned()
            .unwrap_or_else(|| Name::new("Unknown"));
        let victim = victims.get_mut(event.target).ok();

        if let Some((mut victim_health, victim_name)) = victim {
            let damage = rng.roll_dice(1, 6);
            victim_health.take_damage(damage);
            let victim_name = victim_name.cloned().unwrap_or_else(|| Name::new("Unknown"));
            info!(
                "{} attacks {} with {} damage",
                attacker, victim_name, damage
            );
        }
    }
}

pub fn track_dead(
    mut commands: Commands,
    actors: Query<(Entity, &Health, Option<&Name>), Changed<Health>>,
) {
    for (entity, health, name) in actors.iter() {
        if health.is_dead() {
            info!(
                "{} died",
                name.cloned().unwrap_or_else(|| Name::new("Unknown"))
            );
            commands.entity(entity).despawn_recursive();
        }
    }
}
