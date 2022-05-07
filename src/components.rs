use std::collections::HashSet;

use bevy::prelude::*;
use bracket_lib::prelude::Point;

#[derive(Component)]
pub struct MapViewTerminal;

#[derive(Component)]
pub struct LogTerminal;

#[derive(Component)]
pub struct StatusTerminal;

#[derive(Component, Default, PartialEq, Eq, Clone, Copy, Ord, PartialOrd)]
pub struct Layer(pub u32);

#[derive(Component, Clone, Copy)]
pub struct Blocker;

#[derive(Component, Clone, Copy)]
pub struct Opaque;

#[derive(Component, Clone, Copy)]
pub struct Revealed;

#[derive(Component, Clone, Copy)]
pub struct Unrevealable;

#[derive(Component, Clone, Copy)]
pub struct Visible;

#[derive(Component, Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new<T>(x: T, y: T) -> Self
    where
        T: TryInto<i32>,
    {
        Self {
            x: x.try_into().unwrap_or(0),
            y: y.try_into().unwrap_or(0),
        }
    }

    pub fn distance_squared(&self, other: &Position) -> i32 {
        (self.x - other.x).pow(2) + (self.y - other.y).pow(2)
    }
}

impl From<Point> for Position {
    fn from(point: Point) -> Self {
        Self::from(&point)
    }
}

impl From<&Point> for Position {
    fn from(point: &Point) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<&Position> for Point {
    fn from(position: &Position) -> Self {
        Self {
            x: position.x,
            y: position.y,
        }
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct WantToMove {
    pub position: Position,
}

#[derive(Component)]
pub struct Fov {
    pub visible_tiles: HashSet<Position>,
    pub range: u32,
}

impl Fov {
    pub fn new(range: u32) -> Self {
        Self {
            visible_tiles: HashSet::new(),
            range,
        }
    }
}

#[derive(Default, Debug, Component)]
pub struct Energy(pub i32);

#[derive(Debug, Component)]
pub struct Speed(pub i32);

#[derive(Default, Debug, Component)]
pub struct Actor;

#[derive(Debug, Component)]
pub struct TakingATurn;

#[derive(Bundle)]
pub struct MovingEntityBundle {
    pub position: Position,
    pub energy: Energy,
    pub speed: Speed,
    pub actor: Actor,
}

impl MovingEntityBundle {
    pub fn new(speed: i32) -> Self {
        Self {
            speed: Speed(speed),
            position: Position::default(),
            energy: Default::default(),
            actor: Default::default(),
        }
    }

    pub fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }
}
