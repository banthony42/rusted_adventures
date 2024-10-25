use core::panic;
use graphics::{clear, color};
use piston_window::*;
use rectangle::Shape;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::game::Game;
use crate::login::Login;
use crate::states::StateFactory;
use crate::tasks::task::TaskData;
use crate::tasks::task::TaskInterface;
use crate::{constants::*, states::GameState, ui::font::Font};

pub enum LoadingNextState {
    Game,
    Login,
}

pub struct Loading {
    _rt: Runtime,
    timeout: u128,
    task: JoinHandle<()>,
    task_data: Arc<Mutex<TaskData>>,
    next_state: LoadingNextState,
    margin: Size,
    progress: u128,
    pub font: Font,
}

/*
** Loading state take the next state, an async task to run and a timeout
** This state launch the async task and display a progress bar at the same time.
** When the task is finished the state pass to next_state
*/
impl Loading {
    pub fn new(
        next_state: LoadingNextState,
        task: Box<dyn TaskInterface>,
        window: &mut PistonWindow,
    ) -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();

        let amt = task.get_shared_data();
        let timeout = task.get_timeout();

        let master_task = runtime.spawn(async move {
            tokio::select! {
                _ = async { sleep(Duration::from_millis(task.get_timeout())).await; } => {},
                _ = task.task() => {},
            };
        });

        let mut font = Font::new();
        font.load(window);
        Loading {
            _rt: runtime,
            task: master_task,
            task_data: amt,
            timeout: timeout as u128,
            progress: 0,
            next_state: next_state,
            font: font,
            margin: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }
}

const TITLE: &str = "Loading";
const TITLE_FONT_SIZE: u32 = 28;
const PROGRESS_BAR_HEIGHT: f64 = WINDOW_HEIGHT as f64 / 2.0;
const LOGIN_TITLE_POS: [f64; 2] = [WINDOW_WIDTH_CENTER as f64, PROGRESS_BAR_HEIGHT - 20.0];

impl GameState for Loading {
    fn state_update(self: Box<Self>, window: &mut PistonWindow) -> Box<dyn GameState> {
        // timeout + 100 => keep displaying progress bar during 100ms when it reached it's maximum
        if !self.task.is_finished() || self.progress < self.timeout + 100 {
            return self;
        }

        let fetch_data = self.task_data.lock().unwrap();
        let mut new_state = match fetch_data.success {
            true => match self.next_state {
                LoadingNextState::Game => StateFactory::<Game>::new(window),
                LoadingNextState::Login => StateFactory::<Login>::new(window),
            },
            false => StateFactory::<Login>::new(window),
        };

        new_state.pass_data(fetch_data.data.clone()); // TODO: pass data at instanciation
        return new_state;
    }

    fn render(&mut self, evnt: &Event, window: &mut PistonWindow) {
        window.draw_2d(evnt, |_ctx, gl, _device| {
            clear(color::BLACK, gl);

            let rect = Rectangle::new([1.0; 4]).color(color::hex("2d2d2d"));
            let mut final_position = [0.0, 0.0, WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64];
            final_position[0] += self.margin.width;
            final_position[1] += self.margin.height;
            rect.draw(final_position, &_ctx.draw_state, _ctx.transform, gl);

            let width = 200.0;
            let height = 20.0;
            let bg_rect = [
                self.margin.width + WINDOW_WIDTH_CENTER as f64 - width / 2.0,
                self.margin.height + WINDOW_HEIGHT as f64 / 2.0,
                width,
                height,
            ];

            Rectangle::new([1.0; 4])
                .color(color::BLACK)
                .shape(Shape::Round(8.0, 32))
                .draw(bg_rect, &_ctx.draw_state, _ctx.transform, gl);

            let progress_width = (self.progress as f64 / self.timeout as f64).min(1.0) * width;
            let mut progress_rect = bg_rect.clone();
            progress_rect[2] = progress_width;

            if progress_width > 10.0 {
                Rectangle::new([1.0; 4])
                    .color(color::WHITE)
                    .shape(Shape::Round(8.0, 32))
                    .draw(progress_rect, &_ctx.draw_state, _ctx.transform, gl);
            }
        });

        self.font.render_text_centered(
            TITLE,
            TITLE_FONT_SIZE,
            evnt,
            window,
            color::WHITE,
            [LOGIN_TITLE_POS[0], LOGIN_TITLE_POS[1]],
            Some(&self.margin),
        );
    }

    fn update(&mut self, _args: &UpdateArgs, _delta_ts: u128) {
        if self.task.is_finished() {
            self.progress += _delta_ts * 20; // quickly increase progress bar to it's maximum
        } else {
            let task_data = self.task_data.lock().unwrap();
            let time_prog = self.timeout * task_data.step as u128 / (task_data.steps) as u128;
            if self.progress < time_prog {
                self.progress = time_prog;
                println!(
                    "==> (Loading) progress: {} - {}",
                    task_data.step as f64 / task_data.steps as f64,
                    time_prog
                );
            }
        }
    }

    fn resize_window(&mut self, args: &ResizeArgs) {
        let window_width = args.window_size[0];
        let window_height = args.window_size[1];
        println!("==> (Loading) Resized: {window_width}x{window_height}");

        self.margin = self.handle_resize(
            Size {
                width: window_width,
                height: window_height,
            },
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        );
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
}
