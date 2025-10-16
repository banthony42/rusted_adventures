use std::ops::{MulAssign, SubAssign};

use diesel_geometry::data_types::PgPoint;
use rand::distr::{Distribution, StandardUniform};

use crate::world::ColliderMap;

pub mod authenticator;
pub mod character;
pub mod database;
pub mod monster;
pub mod record;
pub mod utils;
pub mod world;

pub mod grpc_codegen {
    include!("../grpc_codegen/rpg.package.rs");
}

pub mod constants {
    use std::ops::Range;

    // Map and Interface size
    // Add check after loading map and gui, if size differ from const stop the program
    pub const TILEMAP_WIDTH: usize = 16;
    pub const TILEMAP_HEIGHT: usize = 12;
    pub const TILEMAP_LINEAR_SIZE: usize = TILEMAP_WIDTH * TILEMAP_HEIGHT;
    pub const TILE_WIDTH: usize = 64;
    pub const TILE_HEIGHT: usize = 64;

    pub const MAP_WIDTH: usize = (TILE_WIDTH * TILEMAP_WIDTH) as usize;
    pub const MAP_HEIGHT: usize = (TILE_HEIGHT * TILEMAP_HEIGHT) as usize;
    pub const MAP_WIDTH_CENTER: usize = (WINDOW_WIDTH - MAP_WIDTH) / 2;
    pub const MAP_HEIGHT_CENTER: usize = (WINDOW_HEIGHT - GAME_HEIGHT) / 2;

    pub const MAP_CHANGE_LIMIT: usize = 32;
    pub const MAP_EAST_LIMIT: usize = MAP_WIDTH - MAP_CHANGE_LIMIT;
    pub const MAP_SOUTH_LIMIT: usize = MAP_HEIGHT - MAP_CHANGE_LIMIT;

    pub const GUI_WIDTH: usize = MAP_WIDTH;
    pub const GUI_HEIGHT: usize = 192;
    pub const GUI_WIDTH_CENTER: usize = (WINDOW_WIDTH - GUI_WIDTH) / 2;
    pub const GAME_HEIGHT: usize = MAP_HEIGHT + GUI_HEIGHT;

    pub const CHAT_FONT_SIZE: u32 = 17;

    pub const GUI_CHAT_X: f64 = 16.0;
    pub const GUI_CHAT_Y: f64 = MAP_HEIGHT as f64 + 18.0;
    pub const GUI_CHAT_WIDTH: f64 = 416.0;
    pub const GUI_CHAT_HEIGHT: f64 = 140.0;
    pub const GUI_CHAT_SIZE: [f64; 4] = [GUI_CHAT_X, GUI_CHAT_Y, GUI_CHAT_WIDTH, GUI_CHAT_HEIGHT];
    pub const GUI_CHAT_PADDING_WIDTH: f64 = 5.0;

    pub const CHAT_INPUT_POSITION: [f64; 2] = [16.0, 928.0];
    pub const CHAT_INPUT_WIDTH: f64 = GUI_CHAT_WIDTH;

    pub const CHAT_WINDOW_TIMER: u128 = 8000;
    pub const CHAT_WINDOW_WIDTH: f64 = 128.0;
    pub const CHAT_WINDOW_PADDING_HEIGHT: f64 = 10.0;
    pub const CHAT_WINDOW_PADDING_WIDTH: f64 = GUI_CHAT_PADDING_WIDTH;
    pub const CHAT_WINDOW_MARGIN_BOTTOM: f64 = 5.0;

    pub const GUI_ENTITY_FONT_SIZE: u32 = 17;

    // Window size
    pub const WINDOW_WIDTH: usize = MAP_WIDTH;
    pub const WINDOW_HEIGHT: usize = MAP_HEIGHT + GUI_HEIGHT;
    pub const WINDOW_WIDTH_CENTER: usize = WINDOW_WIDTH / 2;

    pub const MAP_WIDTH_RANGE: Range<i64> = 0..MAP_WIDTH as i64;
    pub const MAP_HEIGHT_RANGE: Range<i64> = 0..MAP_HEIGHT as i64;

    pub const SERVER_ENDPOINT: &str = "http://127.0.0.1:21210";
    pub const CHAT_SERVER_ENDPOINT: &str = "http://127.0.0.1:21210";

    #[derive(Hash, PartialEq, Eq, Debug, Clone)]
    pub enum Species {
        Warrior,
        Mage,
        Bouftou,
        Crabedoeuf,
    }

    use crate::grpc_codegen;

    use super::grpc_codegen::entity::Family;

    impl From<Family> for Species {
        fn from(value: Family) -> Self {
            match value {
                Family::Species(species) => match species {
                    val if val == grpc_codegen::Species::Bouftou as i32 => Species::Bouftou,
                    val if val == grpc_codegen::Species::Crabedoeuf as i32 => Species::Crabedoeuf,
                    _ => todo!(),
                },
                Family::Class(class) => match class {
                    val if val == grpc_codegen::Classes::Warrior as i32 => Species::Warrior,
                    val if val == grpc_codegen::Classes::Mage as i32 => Species::Mage,
                    _ => todo!(),
                },
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CellCoord {
    pub x: i64,
    pub y: i64,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MapCoord {
    pub x: i8,
    pub y: i8,
}

impl Into<PgPoint> for MapCoord {
    fn into(self) -> PgPoint {
        PgPoint(self.x as f64, self.y as f64)
    }
}

impl Into<PgPoint> for CellCoord {
    fn into(self) -> PgPoint {
        PgPoint(self.x as f64, self.y as f64)
    }
}

impl CellCoord {
    /// Get linear index of this Map coordinate.
    /// Then it can be use to retrieve the same pixel in an 1D array,
    /// where its size is TILEMAP_LINEAR_SIZE (TILEMAP_WIDTH * TILEMAP_HEIGHT)
    pub fn linear_index(&self) -> usize {
        self.limit();
        self.y as usize * constants::TILEMAP_WIDTH + self.x as usize
    }

    /// Limit this Map Coordinate to TILEMAP limits.
    pub fn limit(mut self) -> Self {
        self.x = self.x.max(0).min(constants::TILEMAP_WIDTH as i64 - 1);
        self.y = self.y.max(0).min(constants::TILEMAP_HEIGHT as i64 - 1);
        self
    }

    pub fn min(mut self, rhs: Self) -> Self {
        self.x = self.x.min(rhs.x);
        self.y = self.y.min(rhs.y);
        self
    }

    pub fn is_null(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    pub fn spawn() -> Self {
        Self { x: 5, y: 5 }
    }

    pub fn random() -> Self {
        Self {
            x: rand::random_range(0..constants::TILEMAP_WIDTH as i64 - 1),
            y: rand::random_range(0..constants::TILEMAP_HEIGHT as i64 - 1),
        }
    }

    pub fn random_not_collider(collider_map: &ColliderMap) -> Self {
        loop {
            let cell = Self {
                x: rand::random_range(0..constants::TILEMAP_WIDTH as i64 - 1),
                y: rand::random_range(0..constants::TILEMAP_HEIGHT as i64 - 1),
            };

            if collider_map.is_not_collider(cell.y as usize, cell.x as usize) {
                break cell;
            }
        }
    }
}

impl std::ops::Mul<f64> for CellCoord {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: (self.x as f64 * rhs) as i64,
            y: (self.y as f64 * rhs) as i64,
        }
    }
}

impl MulAssign<f64> for CellCoord {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl std::ops::Sub for CellCoord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for CellCoord {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl std::ops::Add for CellCoord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign for CellCoord {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Add for MapCoord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign for MapCoord {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Orientation {
    Est,
    West,
    North,
    South,
}

impl Orientation {
    /// Return the opposite Orientation.
    ///
    /// # Example
    ///
    /// ```
    /// let north = Orientation::North;
    /// assert_eq!(Orientation::South, north.invert());
    /// ```
    pub fn invert(&self) -> Self {
        match self {
            Orientation::North => Orientation::South,
            Orientation::Est => Orientation::West,
            Orientation::West => Orientation::Est,
            Orientation::South => Orientation::North,
        }
    }
}

impl Distribution<Orientation> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Orientation {
        match rng.random_range(0..=3) {
            0 => Orientation::North,
            1 => Orientation::West,
            2 => Orientation::Est,
            _ => Orientation::South,
        }
    }
}
