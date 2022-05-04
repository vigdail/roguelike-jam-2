use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bracket_lib::prelude::{
    Algorithm2D, BaseMap, DistanceAlg, FastNoise, FractalType, NoiseType, Point,
    RandomNumberGenerator, Rect, SmallVec,
};

use crate::{
    items::health_potion, map_tile::TileType, monster::spawn_monster, player::spawn_player,
    Blocker, Opaque, Position, MAP_SIZE,
};

#[allow(dead_code)]
pub struct Map {
    width: usize,
    height: usize,
    pub tiles: HashMap<Position, Vec<Entity>>,
    pub opaque: HashSet<Position>,
    pub blockers: HashSet<Position>,
}

#[allow(dead_code)]
impl Map {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: HashMap::new(),
            opaque: HashSet::new(),
            blockers: HashSet::new(),
        }
    }

    pub fn is_in_bounds(&self, position: &Position) -> bool {
        position.x >= 0
            && position.y >= 0
            && position.x < self.width as i32
            && position.y < self.height as i32
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

    pub fn idx_position<T>(&self, idx: T) -> Option<Position>
    where
        T: TryInto<usize>,
    {
        let idx = idx.try_into().ok()?;
        if idx >= self.width * self.height {
            None
        } else {
            let x = idx % self.width;
            let y = idx / self.width;
            Some(Position::new(x, y))
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(Point::new(x, y)) {
            return false;
        }
        !self.blockers.contains(&Position::new(x, y))
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(build_map)
            .add_system_to_stage(CoreStage::PreUpdate, collect_tiles);
    }
}

fn build_map(mut commands: Commands) {
    let width = MAP_SIZE[0] as usize;
    let height = MAP_SIZE[1] as usize;
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

    spawn_player(
        &mut commands,
        map_info.player_start.map(|p| p.into()).unwrap_or_default(),
    );

    health_potion(&mut commands, map_info.player_start.unwrap().into());

    map_info.rooms.iter().skip(1).for_each(|room| {
        spawn_room(&mut commands, room);
    });
}

fn spawn_room(commands: &mut Commands, room: &Rect) {
    let mut spawned = HashMap::new();
    let mut rng = RandomNumberGenerator::new();

    let num_monsters = rng.roll_dice(1, 3) - 1;
    assert!(num_monsters >= 0);
    for _ in 0..num_monsters {
        let mut added = false;
        while !added {
            let x = rng.range(room.x1 + 1, room.x2 - 1) as usize;
            let y = rng.range(room.y1 + 1, room.y2 - 1) as usize;
            if spawned.get(&(x, y)).is_none() {
                spawned.insert((x, y), "monster");
                added = true;
            }
        }
    }

    let num_items = rng.roll_dice(1, 2) - 1;
    assert!(num_items >= 0);
    for _ in 0..num_items {
        let mut added = false;
        while !added {
            let x = rng.range(room.x1 + 1, room.x2 - 1) as usize;
            let y = rng.range(room.y1 + 1, room.y2 - 1) as usize;
            if spawned.get(&(x, y)).is_none() {
                spawned.insert((x, y), "item");
                added = true;
            }
        }
    }

    for ((x, y), name) in spawned {
        let position = Position::new(x, y);
        match name {
            "monster" => spawn_monster(commands, position),
            "item" => health_potion(commands, position),
            _ => unreachable!(),
        };
    }
}

fn collect_tiles(
    mut map: ResMut<Map>,
    tiles: Query<(Entity, &Position, Option<&Blocker>, Option<&Opaque>)>,
) {
    map.tiles.clear();
    map.opaque.clear();
    map.blockers.clear();
    for (entity, position, blocks_move, opaque) in tiles.iter() {
        map.tiles
            .entry(*position)
            .or_insert(Vec::new())
            .push(entity);

        if opaque.is_some() {
            map.opaque.insert(*position);
        }

        if blocks_move.is_some() {
            map.blockers.insert(*position);
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.idx_position(idx)
            .map(|pos| self.opaque.contains(&pos))
            .unwrap_or(true)
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = (idx % self.width) as i32;
        let y = (idx / self.width) as i32;
        let w = self.width;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

pub struct MapInfo {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub player_start: Option<Point>,
}

impl MapInfo {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            player_start: None,
            rooms: vec![],
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
        map.rooms = self.rooms.clone();
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
        let mut rng = RandomNumberGenerator::new();
        let mut noise = FastNoise::seeded(rng.next_u64());
        noise.set_noise_type(NoiseType::PerlinFractal);
        noise.set_fractal_type(FractalType::FBM);
        noise.set_fractal_octaves(5);
        noise.set_fractal_gain(0.6);
        noise.set_fractal_lacunarity(2.0);
        noise.set_frequency(8.0);

        let mut max = -10.0f32;
        let mut min = 10.0f32;
        for x in (room.x1)..room.x2 {
            for y in (room.y1)..room.y2 {
                let index = y as usize * self.width + x as usize;
                let n =
                    noise.get_noise(x as f32 / self.width as f32, y as f32 / self.height as f32);
                max = max.max(n);
                min = min.min(n);
                let is_grass = n < 0.0;
                if is_grass {
                    map.tiles[index] = TileType::Grass;
                } else {
                    map.tiles[index] = TileType::Floor;
                }
            }
        }
        println!("Min: {}, Max: {}", min, max);
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
