use common::{
    entities::{model::EntityModel, monsters::MonsterType, player::PlayerType, traits::GameEntity},
    grpc_codegen::{entity::Family, Classes, Entity, Species},
    network::conversion::NetEntityError,
    world::entities::ENTITY_RUN_STEP_DURATION,
    CellCoord, MapCoord, Orientation,
};
use graphics::{color, types::Color};

use crate::import::assets::{Animations, EntityAssets};

/// User Interface version of an EntityModel
#[derive(Clone, PartialEq)]
pub struct UIEntityModel {
    pub real_position: CellCoord,
    pub map: MapCoord,
    pub render_offset: [f64; 2],
}

/// Entity data needed by the graphic client
#[derive(Debug, Clone)]
pub struct GraphicEntityModel {
    render_offset: [f64; 2],
    font_color: Color,
    cell_offset: CellCoord,
    state: Animations,
    step: f64,
    frame: u8,
    timer: u128,
    orientation: Orientation,
}

impl GraphicEntityModel {
    fn new(render_offset: [f64; 2], font_color: Color) -> Self {
        GraphicEntityModel {
            render_offset,
            font_color,
            cell_offset: CellCoord::default(),
            state: Animations::default(),
            orientation: Orientation::Est,
            timer: 0,
            frame: 0,
            step: 0.0,
        }
    }

    pub fn get_timer(&self) -> u128 {
        self.timer
    }

    pub fn set_timer(&mut self, timer: u128) {
        self.timer = timer;
    }

    pub fn get_frame(&self) -> usize {
        self.frame as usize
    }

    pub fn set_frame(&mut self, frame: u8) {
        self.frame = frame;
    }

    fn set_state(&mut self, state: Animations) {
        if self.state != state {
            self.state = state;
            self.frame = 0;
            self.timer = 0;
        }
    }

    pub fn font_color(&self) -> Color {
        self.font_color
    }

    pub fn render_offset(&self) -> [f64; 2] {
        self.render_offset
    }

    pub fn get_orientation(&self) -> &Orientation {
        &self.orientation
    }
}

#[derive(Debug, Clone)]
pub enum ClientEntityType {
    Player(PlayerType),
    Monster(MonsterType),
}

#[derive(Debug, Clone)]
pub struct ClientEntity {
    pub entity: ClientEntityType,
    pub graphic: GraphicEntityModel,
}

impl TryFrom<Entity> for ClientEntity {
    type Error = NetEntityError;

    fn try_from(value: Entity) -> Result<Self, Self::Error> {
        match value.family {
            None => Err(NetEntityError::MissingFamily),
            val if val == Some(Family::Class(Classes::Warrior as i32)) => Ok(ClientEntity {
                entity: ClientEntityType::Player(value.try_into()?),
                graphic: GraphicEntityModel::new([0.0, 64.0], color::hex("0017ad")),
            }),
            val if val == Some(Family::Class(Classes::Mage as i32)) => Ok(ClientEntity {
                entity: ClientEntityType::Player(value.try_into()?),
                graphic: GraphicEntityModel::new([0.0, 64.0], color::hex("0017ad")),
            }),
            val if val == Some(Family::Species(Species::Bouftou as i32)) => Ok(ClientEntity {
                entity: ClientEntityType::Monster(value.try_into()?),
                graphic: GraphicEntityModel::new([0.0, 0.0], color::hex("c5c9e8")),
            }),
            val if val == Some(Family::Species(Species::Crabedoeuf as i32)) => Ok(ClientEntity {
                entity: ClientEntityType::Monster(value.try_into()?),
                graphic: GraphicEntityModel::new([0.0, 0.0], color::hex("c5c9e8")),
            }),
            Some(Family::Species(s)) => Err(NetEntityError::SpeciesNotFound(s)),
            Some(Family::Class(c)) => Err(NetEntityError::ClassNotFound(c)),
        }
    }
}

impl GameEntity for ClientEntityType {
    fn model(&self) -> &EntityModel {
        match self {
            ClientEntityType::Player(player) => player.model(),
            ClientEntityType::Monster(monster) => monster.model(),
        }
    }

    fn model_mut(&mut self) -> &mut EntityModel {
        match self {
            ClientEntityType::Player(player) => player.model_mut(),
            ClientEntityType::Monster(monster) => monster.model_mut(),
        }
    }
}

impl GameEntity for ClientEntity {
    fn model(&self) -> &EntityModel {
        self.entity.model()
    }

    fn model_mut(&mut self) -> &mut EntityModel {
        self.entity.model_mut()
    }
}

impl ClientEntity {
    pub fn ui_model(&self) -> UIEntityModel {
        UIEntityModel {
            real_position: self.get_screen_pos(),
            map: self.entity.model().position.map,
            render_offset: self.graphic.render_offset(),
        }
    }

    pub fn get_screen_pos(&self) -> CellCoord {
        let cell = self.entity.model().position.cell;
        CellCoord {
            x: cell.x * 64 + self.graphic.cell_offset.x,
            y: cell.y * 64 + self.graphic.cell_offset.y,
        }
    }

    pub fn asset(&self) -> EntityAssets {
        match &self.entity {
            ClientEntityType::Player(PlayerType::Mage(_)) => EntityAssets::Mage(self.graphic.state),
            ClientEntityType::Player(PlayerType::Warrior(_)) => {
                EntityAssets::Warrior(self.graphic.state)
            }
            ClientEntityType::Monster(MonsterType::Bouftou(_)) => {
                EntityAssets::Bouftou(self.graphic.state)
            }
            ClientEntityType::Monster(MonsterType::Crabedoeuf(_)) => {
                EntityAssets::Crabedoeuf(self.graphic.state)
            }
        }
    }

    pub fn update(&mut self, delta_ts: u128) {
        let entity_model_mut = &mut self.entity.model_mut();
        let Some(path) = &mut entity_model_mut.path else {
            self.graphic.set_state(Animations::Idle);
            return;
        };

        self.graphic.set_state(Animations::Run);
        if let Some(target) = path.last() {
            // Compute unit vector (x and y could be 1, -1 or 0)
            let mut direction = *target - entity_model_mut.position.cell;
            if self.graphic.step > 1.0 {
                if let Some(cell) = path.pop() {
                    entity_model_mut.position.cell = cell;
                    self.graphic.cell_offset.x = 0;
                    self.graphic.cell_offset.y = 0;
                    self.graphic.step = 0.0;
                }
            } else {
                direction *= 64.0;
                self.graphic.cell_offset = CellCoord {
                    x: lerp(0.0, direction.x as f64, self.graphic.step) as i64,
                    y: lerp(0.0, direction.y as f64, self.graphic.step) as i64,
                };
                self.graphic.step += delta_ts as f64 * ENTITY_RUN_STEP_DURATION;
            }

            if let Some(new_orientation) = match direction {
                dir if dir.x > 0 => Some(Orientation::Est),
                dir if dir.x < 0 => Some(Orientation::West),
                dir if dir.y > 0 => Some(Orientation::South),
                dir if dir.y < 0 => Some(Orientation::North),
                _ => None,
            } {
                self.graphic.orientation = new_orientation;
            }
        } else {
            // We failed to get last element, it mean path is empty
            entity_model_mut.path = None;
        }
    }
}

fn lerp(v0: f64, v1: f64, t: f64) -> f64 {
    (1.0 - t) * v0 + t * v1
}
