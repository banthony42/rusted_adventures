use std::sync::{Arc, Mutex};
use rectangle::Shape;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;
use tokio::time::error::Elapsed;
use tokio::time::{sleep, Duration};
use graphics::{clear, color};
use piston_window::*;

use crate::{constants::*, states::GameState, ui::font::Font};

struct Task {
    data: Option<String>
}

pub struct Loading {
    rt: Runtime,
    timeout: u128,
    task: JoinHandle<()>,
    task_data: Arc<Mutex<Task>>,
    next_state: Box<dyn GameState>,
    margin: Size,
    progress: u128,
    pub font: Font,
}


/*
** Loading state take the next state, an async task to run and a timeout
** This state launch the async task and display a progress bar at the same time.
** When the task is finished the state pass to next_state
** TODO: When the timeout is reached we should pass to the previous state.
*/

impl Loading {
    async fn test(e: Arc<Mutex<Task>>) {
        println!("===> Task begin");
        // Simulate network requests
        sleep(Duration::from_millis(500)).await;

        e.lock().unwrap().data = Some(String::from("add fetch data to the shared memory"));
        println!("===> Task finished")
    }

    pub fn new(next_state: Box<dyn GameState>, timeout: u128) -> Self {
        let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

        let amt = Arc::new(Mutex::new(Task { data: None }));
        let amt_shared = amt.clone();

        let master_task = runtime.spawn(async move {
            tokio::select! {
                _ = async { println!("===> Timeout Task begin"); tokio::time::sleep(tokio::time::Duration::from_millis(timeout as u64)).await;  println!("===> Timeout Task finished") } => {},
                _ = Loading::test(amt_shared) => {}
            }
        });

        Loading {
            rt: runtime,
            task: master_task,
            task_data: amt,
            timeout: timeout,
            progress: 0,
            next_state: next_state,
            font: Font::new(),
            margin: Size { width: 0.0, height: 0.0 }
        }
    }
}

const TITLE: &str = "Loading";
const TITLE_FONT_SIZE: u32 = 28;
const PROGRESS_BAR_HEIGHT : f64 = WINDOW_HEIGHT as f64 / 2.0;
const LOGIN_TITLE_POS: [f64; 2] = [WINDOW_WIDTH_CENTER as f64, PROGRESS_BAR_HEIGHT - 20.0];

impl GameState for Loading {
    fn state_update(mut self: Box<Self>, window: &mut PistonWindow) -> Box<dyn GameState> {

        if self.task.is_finished() && self.progress >= self.timeout + 300 {
            self.next_state.resize_window(&ResizeArgs {
                window_size: window.size().into(),
                draw_size: window.draw_size().into(),
            });
            // Pass fetched data to the next state
            // let fetch_data = self.task_data.lock().unwrap().data.clone();
            // self.next_state.pass_fetch_data(fetch_data);
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
            let bg_rect = [self.margin.width + WINDOW_WIDTH_CENTER as f64 - width / 2.0, self.margin.height + WINDOW_HEIGHT as f64 / 2.0, width, height];

            Rectangle::new([1.0; 4])
            .color(color::BLACK)
            .shape(Shape::Round(8.0, 32))
            .draw(bg_rect, &_ctx.draw_state, _ctx.transform, gl);

            let progress_width = (self.progress as f64 / self.timeout as f64).min(1.0) * width;
            let mut progress_rect = bg_rect.clone();
            progress_rect[2] = progress_width;

            Rectangle::new([1.0; 4])
            .color(color::WHITE)
            .shape(Shape::Round(8.0, 32))
            .draw(progress_rect, &_ctx.draw_state, _ctx.transform, gl);
        });

        let title_width = self.font.get().width(TITLE_FONT_SIZE, TITLE).unwrap();

        self.font.render_text(
            TITLE,
            TITLE_FONT_SIZE,
            evnt,
            window,
            color::WHITE,
            [LOGIN_TITLE_POS[0] - title_width / 2.0, LOGIN_TITLE_POS[1]],
            Some(&self.margin),
        );
    }

    fn update(&mut self, _args: &UpdateArgs, _delta_ts: u128) {
        if self.task.is_finished() && self.progress < self.timeout {
            self.progress = self.timeout;
        } else {
            self.progress += _delta_ts;
        }
    }

    fn resize_window(&mut self, args: &ResizeArgs) {
        let window_width = args.window_size[0];
        let window_height = args.window_size[1];
        println!("==> (Loading)Resized: {window_width}x{window_height}");

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