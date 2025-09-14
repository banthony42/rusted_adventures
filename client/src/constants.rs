use std::{collections::HashMap, ops::Range};

use graphics::{color, types::Color};

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

pub const GUI_CHAT_X: usize = 20;
pub const GUI_CHAT_Y: usize = MAP_HEIGHT + 18;
pub const GUI_CHAT_WIDTH: usize = 408;
pub const GUI_CHAT_HEIGHT: usize = 140;

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
    Crabedoeuf
}

pub struct SpeciesConstants {
    pub font_color: Color,
    // See notes about render_offest below
    pub _render_offset_x: f64,
    pub render_offset_y: f64,
}

pub struct SpeciesLibrary(HashMap<Species, SpeciesConstants>);

impl SpeciesLibrary {
    pub fn new() -> Self {
        SpeciesLibrary(HashMap::from([
            (Species::Warrior, SpeciesConstants { font_color: color::hex("0017ad"), _render_offset_x: 0.0, render_offset_y: 64.0 }),
            (Species::Mage, SpeciesConstants{ font_color: color::hex("0017ad"), _render_offset_x: 0.0, render_offset_y: 64.0 }),
            (Species::Bouftou, SpeciesConstants{ font_color: color::hex("c5c9e8"), _render_offset_x: 0.0, render_offset_y: 0.0 }),
            (Species::Crabedoeuf, SpeciesConstants{ font_color: color::hex("c5c9e8"), _render_offset_x: 0.0, render_offset_y: 0.0 })
        ]))
    }

    fn _get_species_constants(&self, species: &Species) ->  &SpeciesConstants {
        self.0.get(&species).expect(format!("SpeciesLibrary: Unknow species: {:?}", species).as_str())
    }

    pub fn get_font_color(&self, species: &Species) -> Color {
        self._get_species_constants(species).font_color.clone()
    }

    pub fn get_height_offset(&self, species: &Species) -> f64 {
        self._get_species_constants(species).render_offset_y
    }

}


// render_offset:
// Sprites are render at their position, and drawn from top to bottom
// Knowing that we need offsets for sprites higher and or larger than one tile.
