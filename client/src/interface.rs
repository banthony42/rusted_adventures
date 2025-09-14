extern crate image;

use std::{collections::HashMap, path::Path};

use piston_window::*;
use winit::window::Icon;

use crate::import::assets::HardTexture;
use common::constants::*;

pub struct Interface {
    img: Image,
    delta_ts: u128,
}

type TextureLib = HashMap<HardTexture, G2dTexture>;

impl Interface {
    pub fn new() -> Self {
        return Interface {
            delta_ts: 0,
            img: Image::new().rect([
                GUI_WIDTH_CENTER as f64,
                MAP_HEIGHT as f64,
                GUI_WIDTH as f64,
                GUI_HEIGHT as f64,
            ]),
        };
    }

    pub fn update(&mut self, _args: &UpdateArgs, delta_ts: u128) {
        self.delta_ts = delta_ts;
    }

    pub fn render(&mut self, evnt: &Event, window: &mut PistonWindow, texture_lib: &TextureLib) {
        window.draw_2d(evnt, |ctx, gl, _device| {
            self.img.draw(
                &texture_lib[&HardTexture::Interface],
                &DrawState::default(),
                ctx.transform,
                gl,
            );
        });
    }

    pub fn resize(&mut self, margin: &Size) {
        self.img = Image::new().rect([
            margin.width,
            margin.height + MAP_HEIGHT as f64,
            GUI_WIDTH as f64,
            GUI_HEIGHT as f64,
        ]);
    }
}

pub fn load_icon(path: &Path) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
