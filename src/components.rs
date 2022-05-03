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

#[derive(Component, Default, PartialEq, Eq, Hash, Clone, Copy)]
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
