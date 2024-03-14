
// Map and Interface size
// Add check after loading map and gui, if size differ from const stop the program
pub const TILEMAP_WIDTH: usize = 16; 
pub const TILEMAP_HEIGHT: usize = 12;
pub const TILE_WIDTH: usize = 64;
pub const TILE_HEIGHT: usize = 64;

pub const MAP_WIDTH:usize = TILE_WIDTH * TILEMAP_WIDTH;
pub const MAP_HEIGHT:usize = TILE_HEIGHT * TILEMAP_HEIGHT;
pub const MAP_WIDTH_CENTER:usize = (WINDOW_WIDTH - MAP_WIDTH) / 2;
pub const MAP_HEIGHT_CENTER:usize = (WINDOW_HEIGHT - GAME_HEIGHT) / 2;

pub const GUI_WIDTH:usize = MAP_WIDTH;
pub const GUI_HEIGHT:usize = 192;
pub const GUI_WIDTH_CENTER:usize = (WINDOW_WIDTH - GUI_WIDTH) / 2;
pub const GAME_HEIGHT:usize = MAP_HEIGHT + GUI_HEIGHT;

// Window size
pub const WINDOW_WIDTH:usize = MAP_WIDTH;
pub const WINDOW_HEIGHT:usize = MAP_HEIGHT + GUI_HEIGHT;

// Colors
pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];