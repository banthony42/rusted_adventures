
use piston_window::*;

use crate::game::Game;

pub struct Login {
    pub login: bool
}

pub trait GameState {
    fn state_update(self: Box<Self>, window: &mut PistonWindow) -> Box<dyn GameState>;
    fn render(&mut self, evnt: &Event, window: &mut PistonWindow);
    fn resize_window(&mut self, args: &ResizeArgs);
    fn update(&mut self, args: &UpdateArgs) {}
    fn key_press(&mut self, args: &Button) {}
    fn key_release(&mut self, args: &Button) {}
    fn text_input(&mut self, args: String) {}
    fn mouse_cursor_args(&mut self, args: [f64; 2]) {}
    fn mouse_scroll_args(&mut self, args: [f64; 2]) {}
}

impl GameState for Login {
    fn state_update(self: Box<Self>, window: &mut PistonWindow) -> Box<dyn GameState> {
        if self.login {
            let mut game = Game::new(window);
            game.font.load(window);
            game.resize_window(&ResizeArgs {
                window_size: window.size().into(),
                draw_size: window.draw_size().into(),
            });
            return Box::new(game);
        }
        self
    }
    
    fn render(&mut self, evnt: &Event, window: &mut PistonWindow) {
        window.draw_2d(evnt, |_ctx, gl, _device| {
            clear(color::GREEN, gl);
        });
    }
    
    fn update(&mut self, args: &UpdateArgs) {
    }
    
    fn key_press(&mut self, args: &Button) {
        if let &Button::Keyboard(key) = args {
            match key {
                piston::Key::Return => self.login = true,
                _ => {}
            }
        }
    }
    
    fn resize_window(&mut self, args: &ResizeArgs) {
    }
}
