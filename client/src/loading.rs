use graphics::{clear, color};
use piston_window::*;
use rectangle::Shape;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::client::ConnectionTask;
use crate::client::GameData;
use crate::client::TaskData;
use crate::client::TaskInterface;
use crate::login::Login;
use crate::{constants::*, states::GameState, ui::font::Font};

pub struct Loading {
    _rt: Runtime,
    timeout: u128,
    task: JoinHandle<()>,
    task_data: Arc<Mutex<TaskData>>,
    server_data: Vec<GameData>,
    next_state: Box<dyn GameState>,
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
    pub fn new(next_state: Box<dyn GameState>, task: ConnectionTask) -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();

        let amt = task.get_shared_data();
        let timeout = task.get_timeout();

        let master_task = runtime.spawn(async move {
            tokio::select! {
                _ = async { sleep(Duration::from_millis(task.get_timeout() as u64)).await; } => {},
                _ = task.task() => {},
            };
        });

        Loading {
            _rt: runtime,
            server_data: Vec::new(),
            task: master_task,
            task_data: amt,
            timeout: timeout as u128,
            progress: 0,
            next_state: next_state,
            font: Font::new(),
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
    fn pass_server_data(&mut self, data: Vec<GameData>) {
        self.server_data = data;
    }

    fn state_update(mut self: Box<Self>, window: &mut PistonWindow) -> Box<dyn GameState> {
        // timeout + 100 => keep displaying progress bar during 100ms when it reached it's maximum
        if self.task.is_finished() && self.progress >= self.timeout + 100 {
            self.next_state.resize_window(&ResizeArgs {
                window_size: window.size().into(),
                draw_size: window.draw_size().into(),
            });

            // Pass fetched data to the next state
            let fetch_data = self.task_data.lock().unwrap();
            if !fetch_data.success {
                let mut login_state = Login::new();
                login_state.pass_server_data(fetch_data.data.clone());
                login_state.font.load(window);
                login_state.resize_window(&ResizeArgs {
                    window_size: window.size().into(),
                    draw_size: window.draw_size().into(),
                });
                return Box::new(login_state);
            }
            self.next_state.pass_server_data(fetch_data.data.clone());
            return self.next_state;
        }
        return self;
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
}
