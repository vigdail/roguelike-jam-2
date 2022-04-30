use std::collections::HashMap;

use bevy::prelude::*;
use bevy_ascii_terminal::Tile;
use bracket_lib::prelude::{Point, RandomNumberGenerator, Rect};

use crate::{BlockMove, Layer, Player, Position, LAYER_MAP, LAYER_PLAYER};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Wall,
    Floor,
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
                bg_color: Color::DARK_GRAY,
                fg_color: Color::BLACK,
            },
            TileType::Floor => Tile {
                glyph: '.',
                bg_color: Color::BLACK,
                fg_color: Color::GRAY,
            },
        }
    }
}

impl TileType {
    pub fn as_name(&self) -> Name {
        match self {
            TileType::Wall => "Wall".into(),
            TileType::Floor => "Floor".into(),
        }
    }

    pub fn is_blocking(&self) -> bool {
        match self {
            TileType::Wall => true,
            TileType::Floor => false,
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
            entity.insert(BlockMove);
        }

        entity.id()
    }
}

#[allow(dead_code)]
pub struct Map {
    width: usize,
    height: usize,
    tiles: HashMap<Position, Vec<Entity>>,
}

#[allow(dead_code)]
impl Map {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: HashMap::new(),
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn at_position(&self, position: &Position) -> Vec<Entity> {
        self.tiles.get(position).cloned().unwrap_or_default()
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(build_map).add_system(collect_tiles);
    }
}

fn build_map(mut commands: Commands) {
    let width = 80;
    let height = 45;
    let mut builder = RoomMapBuilder::new(width, height);
    let map_info = builder.build();

    let map = Map::new(map_info.width, map_info.height);
    commands.insert_resource(map);

    let tile_entities = map_info
        .tiles
        .iter()
        .enumerate()
        .map(|(i, tile)| {
            let x = i % width;
            let y = i / width;
            (Position::new(x, y), tile)
        })
        .map(|(position, tile)| tile.spawn(&mut commands, position))
        .collect::<Vec<_>>();

    commands
        .spawn()
        .push_children(&tile_entities)
        .insert(Name::new("Map"));

    commands
        .spawn()
        .insert(Player)
        .insert(Tile {
            glyph: '@',
            fg_color: Color::WHITE,
            bg_color: Color::rgba(1.0, 1.0, 1.0, 0.0),
        })
        .insert(Position::from(
            map_info.player_start.unwrap_or_else(Point::zero),
        ))
        .insert(Name::new("Player"))
        .insert(Layer(LAYER_PLAYER));
}

fn collect_tiles(mut map: ResMut<Map>, tiles: Query<(Entity, &Position), With<Tile>>) {
    map.tiles.clear();
    for (entity, position) in tiles.iter() {
        map.tiles
            .entry(*position)
            .or_insert(Vec::new())
            .push(entity);
    }
}

pub struct MapInfo {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TileType>,
    pub player_start: Option<Point>,
}

impl MapInfo {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            player_start: None,
            tiles: vec![TileType::Wall; width * height],
        }
    }

    pub fn xy_idx<T: TryInto<usize>>(&self, x: T, y: T) -> Option<usize> {
        let x = x.try_into().ok()?;
        let y = y.try_into().ok()?;
        if x > self.width || y > self.height {
            None
        } else {
            let idx = y * self.width + x;
            Some(idx)
        }
    }
}

pub trait MapBuilder {
    fn build(&mut self) -> MapInfo;
}

pub struct RoomMapBuilder {
    width: usize,
    height: usize,
    rooms: Vec<Rect>,
    min_room_size: usize,
    max_room_size: usize,
    rng: RandomNumberGenerator,
}

impl RoomMapBuilder {
    pub fn new(width: usize, height: usize) -> Self {
        let min_room_size = 4;
        let max_room_size = (width / 3).min(height / 3).max(min_room_size);
        assert!(min_room_size <= max_room_size);
        Self {
            width,
            height,
            min_room_size,
            max_room_size,
            rng: RandomNumberGenerator::new(),
            rooms: Vec::new(),
        }
    }
}

impl MapBuilder for RoomMapBuilder {
    fn build(&mut self) -> MapInfo {
        const MAX_ROOMS: usize = 30;
        let mut map = MapInfo::new(self.width, self.height);
        for _ in 0..MAX_ROOMS {
            let room = self.build_random_room();
            if self.rooms.iter().any(|r| r.intersect(&room)) {
                continue;
            }

            self.apply_room(&mut map, &room);
            if !self.rooms.is_empty() {
                let Point { x: new_x, y: new_y } = room.center();
                let Point {
                    x: prev_x,
                    y: prev_y,
                } = self.rooms[self.rooms.len() - 1].center();

                if self.rng.rand::<bool>() {
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut map, prev_y, new_y, prev_x);
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, new_y);
                }
            }
            self.rooms.push(room);
        }

        let player_pos = self
            .rooms
            .get(0)
            .map(|r| r.center())
            .unwrap_or_else(Point::zero);

        map.player_start = Some(player_pos);
        map
    }
}

impl RoomMapBuilder {
    fn build_random_room(&mut self) -> Rect {
        let w = self.rng.range(self.min_room_size, self.max_room_size + 1);
        let h = self.rng.range(self.min_room_size, self.max_room_size + 1);
        let x = self.rng.range(2, self.width - w - 1) - 1;
        let y = self.rng.range(2, self.height - h - 1) - 1;

        Rect::with_size(x, y, w, h)
    }

    fn apply_room(&self, map: &mut MapInfo, room: &Rect) {
        for x in (room.x1)..room.x2 {
            for y in (room.y1)..room.y2 {
                let index = y as usize * self.width + x as usize;
                map.tiles[index] = TileType::Floor;
            }
        }
    }
}

fn apply_horizontal_tunnel(map: &mut MapInfo, x1: i32, x2: i32, y: i32) {
    for x in x1.min(x2)..=x1.max(x2) {
        if let Some(idx) = map.xy_idx(x, y) {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

fn apply_vertical_tunnel(map: &mut MapInfo, y1: i32, y2: i32, x: i32) {
    for y in y1.min(y2)..=y1.max(y2) {
        if let Some(idx) = map.xy_idx(x, y) {
            map.tiles[idx] = TileType::Floor;
        }
    }
}
