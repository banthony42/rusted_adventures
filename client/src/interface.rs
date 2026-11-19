extern crate image;

use std::{collections::HashMap, path::Path};

use piston_window::*;
use winit::window::Icon;

use crate::{
    import::assets::UITexture,
    ui::{
        font::Font,
        text_area::{TextArea, TextAreaFormat},
    },
};
use common::{constants::*, MapCoord};

struct MapInfo(String);

pub struct Interface {
    img: Image,
    font: Font,
    margin: Size,
    delta_ts: u128,
    map_info_area: TextArea<MapInfo>,
}

impl TextAreaFormat for MapInfo {
    fn colored_format(&self) -> (types::Color, String) {
        (color::hex("fcf5e6"), self.0.to_string())
    }
}

const GUI_MAP_INFO_FONT_SIZE: u32 = 17;
const GUI_MAP_INFO_RECT: [f64; 4] = [15.0, 15.0, 200.0, GUI_MAP_INFO_FONT_SIZE as f64 * 2.5];

pub struct InterfaceUpdate {
    pub map_info: String,
    pub map_coord: MapCoord,
}

type TextureLib = HashMap<UITexture, G2dTexture>;

impl Interface {
    pub fn new(window: &mut PistonWindow) -> Self {
        Interface {
            map_info_area: TextArea::new(GUI_MAP_INFO_FONT_SIZE, GUI_MAP_INFO_RECT, window),
            font: Font::new(16).load(window),
            delta_ts: 0,
            margin: Size {
                width: MAP_WIDTH_CENTER as f64,
                height: MAP_HEIGHT_CENTER as f64,
            },
            img: Image::new().rect([
                GUI_WIDTH_CENTER as f64,
                MAP_HEIGHT as f64,
                GUI_WIDTH as f64,
                GUI_HEIGHT as f64,
            ]),
        }
    }

    pub fn font(&mut self) -> Vec<&mut Font> {
        vec![&mut self.font, self.map_info_area.font()]
    }

    pub fn update(&mut self, _args: &UpdateArgs, delta_ts: u128, data: InterfaceUpdate) {
        self.delta_ts = delta_ts;
        self.map_info_area.update(
            delta_ts,
            vec![
                MapInfo(data.map_info.to_string()),
                MapInfo(data.map_coord.to_string()),
            ],
        );
    }

    pub fn render(&mut self, evnt: &Event, window: &mut PistonWindow, texture_lib: &TextureLib) {
        window.draw_2d(evnt, |ctx, gl, _device| {
            self.img.draw(
                &texture_lib[&UITexture::Interface],
                &DrawState::default(),
                ctx.transform,
                gl,
            );
        });

        self.map_info_area.render(evnt, window);
    }

    pub fn resize(&mut self, margin: &Size) {
        self.margin = *margin;
        self.img = Image::new().rect([
            margin.width,
            margin.height + MAP_HEIGHT as f64,
            GUI_WIDTH as f64,
            GUI_HEIGHT as f64,
        ]);
        self.map_info_area.resize(margin);
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
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to create icon.")
}
