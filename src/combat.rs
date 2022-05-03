use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bracket_lib::prelude::RandomNumberGenerator;

use crate::{events::AttackEvent, log::GameLog};

#[derive(Clone, Copy, Inspectable)]
pub struct Dice {
    count: i32,
    sides: i32,
    modifier: i32,
}

impl Dice {
    pub fn roll(&self, rng: &mut RandomNumberGenerator) -> i32 {
        let result = rng.roll_dice(self.count, self.sides) + self.modifier;
        result.max(0)
    }
}

impl<T> From<(T, T, T)> for Dice
where
    T: Into<i32>,
{
    fn from((count, sides, modifier): (T, T, T)) -> Self {
        let count = count.into();
        let sides = sides.into();
        let modifier = modifier.into();
        Self {
            count,
            sides,
            modifier,
        }
    }
}

impl<T> From<(T, T)> for Dice
where
    T: Into<i32>,
{
    fn from((count, sides): (T, T)) -> Self {
        Self::from((count.into(), sides.into(), 0i32))
    }
}

#[derive(Component, Clone, Copy, Inspectable)]
pub struct Attack {
    dice: Dice,
}

impl Attack {
    pub fn new<T: Into<Dice>>(dice: T) -> Self {
        Self { dice: dice.into() }
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
    mut game_log: ResMut<GameLog>,
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
                .map(|(_, attack)| attack.dice.roll(&mut rng))
                .unwrap_or(0);

            let attacker_name = attacker.map(|(name, _)| name).cloned().unwrap_or_default();

            victim_health.take_damage(damage);
            let victim_name = victim_name.cloned().unwrap_or_else(|| Name::new("Unknown"));
            game_log.push(format!(
                "{} attacks {} with {} damage",
                attacker_name, victim_name, damage
            ));
        }
    }
}

pub fn track_dead(
    mut game_log: ResMut<GameLog>,
    mut commands: Commands,
    actors: Query<(Entity, &Health, Option<&Name>), Changed<Health>>,
) {
    for (entity, health, name) in actors.iter() {
        if health.is_dead() {
            game_log.push(format!(
                "{} died",
                name.cloned().unwrap_or_else(|| Name::new("Unknown"))
            ));
            commands.entity(entity).despawn_recursive();
        }
    }
}
