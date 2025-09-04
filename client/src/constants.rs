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
