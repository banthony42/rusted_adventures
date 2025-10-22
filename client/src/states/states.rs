use std::marker::PhantomData;

use piston_window::*;

use crate::{
    tasks::task::{GameData, TaskInterface},
    ui::font::Font,
};

use super::{
    game::Game,
    loading::{Loading, LoadingNextState},
    login::Login,
};

pub trait GameState {
    fn pass_data(&mut self, _data: Vec<GameData>) {}

    fn state_update(self: Box<Self>, window: &mut PistonWindow) -> Box<dyn GameState>;
    fn render(&mut self, evnt: &Event, window: &mut PistonWindow);

    fn get_font(&mut self) -> &mut Font;

    /// Submits all text commands to be rendered.
    /// Not calling this function may result in text rendered partially.
    ///
    /// Should be call once per frame. See implementation for more details.
    fn font_flush(&mut self, evnt: &Event, window: &mut PistonWindow) {
        window.draw_2d(evnt, |_, _, device| {
            // This is necessary to execute each text drawing per frame
            // flush documentation recomend to call it once per frame.
            // That's why i have added this `font_flush` function
            // to the GameState trait.
            // Therefore the main loop within main.rs
            // Handle this call once per frame, and then each state can totally forget about this detail.
            self.get_font().get().factory.encoder.flush(device);
        });
    }

    fn handle_resize(&mut self, new_size: Size, max_width: usize, max_height: usize) -> Size {
        let mut margin: Size = Size {
            width: 0.0,
            height: 0.0,
        };
        if new_size.width as usize >= max_width {
            margin.width = ((new_size.width as usize - max_width) / 2) as f64;
        } else {
            margin.width = 0.0;
        }

        if new_size.height as usize >= max_height {
            margin.height = ((new_size.height as usize - max_height) / 2) as f64;
        } else {
            margin.height = 0.0;
        }
        return margin;
    }

    fn resize_window(&mut self, args: &ResizeArgs);
    fn update(&mut self, _args: &UpdateArgs, _delta_ts: u128) {}
    fn key_press(&mut self, _args: &Button) {}
    fn key_release(&mut self, _args: &Button) {}
    fn text_input(&mut self, _args: &String) {}
    fn mouse_cursor_args(&mut self, _args: &[f64; 2]) {}
    fn mouse_scroll_args(&mut self, _args: &[f64; 2]) {}
}

pub struct StateFactory<T>
where
    T: GameState,
{
    __unused: PhantomData<T>,
}

// Factory state Code common to all states
impl<T> StateFactory<T>
where
    T: GameState,
{
    fn init_state(state: &mut T, window: &mut PistonWindow) {
        state.resize_window(&ResizeArgs {
            window_size: window.size().into(),
            draw_size: window.draw_size().into(),
        });
    }
}

impl StateFactory<Login> {
    pub fn new(window: &mut PistonWindow) -> Box<dyn GameState> {
        let mut new_state = Login::new(window);
        Self::init_state(&mut new_state, window);
        Box::new(new_state)
    }
}

impl StateFactory<Loading> {
    pub fn new(
        window: &mut PistonWindow,
        next_state: LoadingNextState,
        task: Box<dyn TaskInterface>,
    ) -> Box<dyn GameState> {
        let mut new_state = Loading::new(next_state, task, window);
        Self::init_state(&mut new_state, window);
        Box::new(new_state)
    }
}

impl StateFactory<Game> {
    pub fn new(window: &mut PistonWindow) -> Box<dyn GameState> {
        let mut new_state = Game::new(window);
        Self::init_state(&mut new_state, window);
        Box::new(new_state)
    }
}
