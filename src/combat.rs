use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bracket_lib::prelude::RandomNumberGenerator;

use crate::events::AttackEvent;

#[derive(Component, Clone, Copy, Inspectable)]
pub struct Attack {
    dice_count: u8,
    dice: u8,
}

impl Attack {
    pub fn new(dice_count: u8, dice: u8) -> Self {
        Self { dice_count, dice }
    }
}

#[derive(Component, Clone, Copy, Inspectable)]
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

#[derive(Bundle)]
pub struct CombatStatsBundle {
    pub health: Health,
    pub attack: Attack,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Health>()
            .register_inspectable::<Attack>();
    }
}

pub fn combat(
    mut attack_events: EventReader<AttackEvent>,
    attackers: Query<(&Name, &Attack)>,
    mut victims: Query<(&mut Health, Option<&Name>)>,
) {
    let mut rng = RandomNumberGenerator::new();
    for event in attack_events.iter() {
        let attacker = attackers.get(event.attacker).ok();
        let victim = victims.get_mut(event.target).ok();

        if let Some((mut victim_health, victim_name)) = victim {
            let damage = attacker
                .map(|(_, attack)| rng.roll_dice(attack.dice_count as i32, attack.dice as i32))
                .unwrap_or(0);

            let attacker_name = attacker.map(|(name, _)| name).cloned().unwrap_or_default();

            victim_health.take_damage(damage);
            let victim_name = victim_name.cloned().unwrap_or_else(|| Name::new("Unknown"));
            info!(
                "{} attacks {} with {} damage",
                attacker_name, victim_name, damage
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
