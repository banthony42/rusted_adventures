
// Map and Interface size
// Add check after loading map and gui, if size differ from const stop the program
pub const TILEMAP_WIDTH: u32 = 16; 
pub const TILEMAP_HEIGHT: u32 = 12;
pub const TILE_WIDTH: u32 = 64;
pub const TILE_HEIGHT: u32 = 64;

pub const MAP_WIDTH:usize = (TILE_WIDTH * TILEMAP_WIDTH) as usize;
pub const MAP_HEIGHT:usize = (TILE_HEIGHT * TILEMAP_HEIGHT) as usize;
pub const MAP_WIDTH_CENTER:usize = (WINDOW_WIDTH - MAP_WIDTH) / 2;
pub const MAP_HEIGHT_CENTER:usize = (WINDOW_HEIGHT - GAME_HEIGHT) / 2;

pub const GUI_WIDTH:usize = MAP_WIDTH;
pub const GUI_HEIGHT:usize = 192;
pub const GUI_WIDTH_CENTER:usize = (WINDOW_WIDTH - GUI_WIDTH) / 2;
pub const GAME_HEIGHT:usize = MAP_HEIGHT + GUI_HEIGHT;

pub const GUI_CHAT_X: usize = 20;
pub const GUI_CHAT_Y: usize = MAP_HEIGHT + 18;
pub const GUI_CHAT_WIDTH: usize = 408; 
pub const GUI_CHAT_HEIGHT: usize = 140;

// Window size
pub const WINDOW_WIDTH:usize = MAP_WIDTH;
pub const WINDOW_HEIGHT:usize = MAP_HEIGHT + GUI_HEIGHT;

// Colors
pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];