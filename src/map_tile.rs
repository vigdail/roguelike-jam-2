use bevy::prelude::*;
use bevy_ascii_terminal::Tile;

use crate::{
    components::{Blocker, Layer, Opaque, Position},
    LAYER_MAP,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Wall,
    Floor,
    Grass,
}

impl From<TileType> for Tile {
    fn from(ty: TileType) -> Self {
        Tile::from(&ty)
    }
}

impl From<&TileType> for Tile {
    fn from(ty: &TileType) -> Self {
        match ty {
            TileType::Wall => Tile {
                glyph: '#',
                bg_color: Color::BLACK,
                fg_color: Color::SEA_GREEN,
            },
            TileType::Floor => Tile {
                glyph: '.',
                bg_color: Color::BLACK,
                fg_color: Color::OLIVE,
            },
            TileType::Grass => Tile {
                glyph: '"',
                bg_color: Color::BLACK,
                fg_color: Color::DARK_GREEN,
            },
        }
    }
}

impl TileType {
    pub fn as_name(&self) -> Name {
        match self {
            TileType::Wall => "Wall".into(),
            TileType::Floor => "Floor".into(),
            TileType::Grass => "Grass".into(),
        }
    }

    pub fn is_blocking(&self) -> bool {
        match self {
            TileType::Wall => true,
            TileType::Floor | TileType::Grass => false,
        }
    }

    pub fn is_opaque(&self) -> bool {
        match self {
            TileType::Wall => true,
            TileType::Floor | TileType::Grass => false,
        }
    }

    pub fn spawn(&self, commands: &mut Commands, position: Position) -> Entity {
        let mut entity = commands.spawn();
        entity
            .insert(Tile::from(self))
            .insert(position)
            .insert(self.as_name())
            .insert(Layer(LAYER_MAP));
        if self.is_blocking() {
            entity.insert(Blocker);
        }
        if self.is_opaque() {
            entity.insert(Opaque);
        }

        entity.id()
    }
}
