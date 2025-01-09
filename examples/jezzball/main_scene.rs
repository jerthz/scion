use hecs::Entity;
use std::fmt::{Display, Formatter};

use rand::{thread_rng, Rng};
use scion::graphics::components::color::Color;
use scion::graphics::components::shapes::rectangle::Rectangle;
use scion::graphics::components::tiles::tilemap::{TileInfos, Tilemap, TilemapInfo, TilemapType};
use scion::core::resources::events::topic::TopicConfiguration;
use scion::core::resources::events::{PollConfiguration, SubscriberId};

use scion::core::world::{GameData, SubWorld, World};
use scion::{
    graphics::components::{
        material::Material,
        tiles::{sprite::Sprite, tileset::Tileset},
    },
    core::resources::asset_manager::AssetRef,
    core::scene::Scene,
};
use winit::window::CursorIcon;
use scion::core::components::maths::collider::{Collider, ColliderMask, ColliderType};
use scion::core::components::maths::transform::Transform;

use crate::utils::{ball_animations, ball_asset, cases_asset};
use scion::utils::maths::{Dimensions, Position, Vector};

#[derive(Debug)]
pub enum BallDirection {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug)]
pub enum LineDirection {
    LEFT,
    RIGHT,
    TOP,
    BOTTOM,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CursorState {
    TOP,
    BOTTOM,
    LEFT,
    RIGHT,
    ROW,
    COLUMN,
}

impl Display for BallDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default)]
pub struct Ball {
    pub direction: Option<BallDirection>,
}

pub struct JezzBallAssets {
    ball_asset: AssetRef<Material>,
}

pub struct Line {
    pub direction: LineDirection,
}

pub struct MainScene {
    tilemap: Option<Entity>,
    subscriber_id: Option<SubscriberId>,
    mouse_state: CursorState,
    state_before_border: Option<CursorState>,
}

impl Default for MainScene {
    fn default() -> Self {
        Self {
            tilemap: None,
            subscriber_id: None,
            mouse_state: CursorState::ROW,
            state_before_border: None,
        }
    }
}

impl Scene for MainScene {
    fn on_start(&mut self, data: &mut GameData) {
        self.tilemap = Some(add_tilemap(data));
        let (world, resources) = data.split();
        world.add_default_camera();

        let ball_asset =
            resources.assets_mut().register_tileset(Tileset::new("balls".to_string(), ball_asset(), 4, 4, 28, 28));
        let assets = JezzBallAssets { ball_asset };

        // Creating the level

        init_balls(world, &assets);

        add_border(world);

        // default mouse cursor
        resources.window().set_cursor(CursorIcon::NResize);
        resources.insert_resource(assets);

        let _r = resources.events().create_topic("TILEMAP_UPDATE", TopicConfiguration::default());
        self.subscriber_id = Some(
            resources.events().subscribe("TILEMAP_UPDATE", PollConfiguration::default()).unwrap(),
        );
    }

    fn on_update(&mut self, data: &mut GameData) {
        let mut to_modify = Vec::new();
        let mut modified = false;
        data.events()
            .poll::<(usize, usize, usize, usize)>(self.subscriber_id.as_ref().unwrap())
            .unwrap()
            .into_iter()
            .for_each(|(x_min, x_max, y_min, y_max)| {
                for x in x_min..=x_max {
                    for y in y_min..=y_max {
                        modified = true;
                        to_modify.push((Position::new(x, y, 0), 2));
                    }
                }
            });
        to_modify.drain(0..).for_each(|e| {
            Tilemap::modify_sprite_tile(data, self.tilemap.unwrap(), e.0, e.1);
        });
        if modified {
            let ball_pos: Vec<(usize, usize)> = data
                .query::<(&Ball, &Transform)>()
                .iter()
                .map(|(_, (_b, t))| {
                    (
                        (t.translation().x() as usize - 10) / 16,
                        (t.translation().y() as usize - 10) / 16,
                    )
                })
                .collect();

            let mut pathfinded_cases = [[false; 38]; 68];
            ball_pos.iter().for_each(|(pos_x, pos_y)| {
                let mut visited = [[false; 38]; 68];
                pathfind_from((*pos_x, *pos_y), data, self.tilemap.unwrap(), &mut visited)
                    .iter()
                    .for_each(|e| {
                        pathfinded_cases[e.0][e.1] = true;
                    });
            });

            let mut open = 0;
            for i in 0..68 {
                for j in 0..38 {
                    if !pathfinded_cases[i][j] {
                        let tmp_pos = Position::new(i, j, 0);
                        Tilemap::modify_sprite_tile(data, self.tilemap.unwrap(), tmp_pos, 2);
                    } else {
                        open += 1;
                    }
                }
            }

            if open < 150 {
                data.scene_controller().switch::<MainScene>();
            }
        }
        let mut change_mouse_state = false;
        let mouse_pos = data.inputs().mouse_xy();
        if !self.compute_mouse_on_border(mouse_pos, data) {
            data.inputs().on_right_click_released(|_pos_x, _pos_y| change_mouse_state = true);
            if change_mouse_state {
                let new_icon = self.compute_new_icon();
                data.window().set_cursor(new_icon);
            } else if self.mouse_state != CursorState::ROW && self.mouse_state != CursorState::COLUMN {
                self.mouse_state = self.state_before_border.as_ref().unwrap().clone();
                self.state_before_border = None;
                self.change_cursor(data);
            }
        }
        let (world, resources) = data.split();
        resources.inputs().on_left_click_pressed(|pos_x, pos_y| {
            let len = world.query::<&Rectangle>().iter().len();

            if len == 0 && pos_x > 10. && pos_y > 10. {
                let x = (pos_x as usize - 10) / 16;
                let y = (pos_y as usize - 10) / 16;
                if self.mouse_state == CursorState::ROW {
                    world.push((
                        Transform::from_xyz(x as f32 * 16. + 10., y as f32 * 16. + 10., 20),
                        Rectangle::new(16., 16., None),
                        Material::Diffuse(Color::new_hex("#00ff664c")),
                        Line { direction: LineDirection::TOP },
                        Collider::new(
                            ColliderMask::Landscape,
                            vec![
                                ColliderMask::Bullet,
                                ColliderMask::Custom("BORDER_CUSTOM_HORIZONTAL".to_string()),
                                ColliderMask::Custom("BORDER_CUSTOM_VERTICAL".to_string()),
                                ColliderMask::Custom("BORDER_TOP".to_string()),
                                ColliderMask::Custom("BORDER_BOTTOM".to_string()),
                                ColliderMask::Custom("BORDER_LEFT".to_string()),
                                ColliderMask::Custom("BORDER_RIGHT".to_string()),
                            ],
                            ColliderType::RectangleCollider(16, 16),
                        ),
                    ));
                    world.push((
                        Transform::from_xyz(x as f32 * 16. + 10., y as f32 * 16. + 26., 20),
                        Rectangle::new(16., 16., None),
                        Material::Diffuse(Color::new_hex("#008cae")),
                        Line { direction: LineDirection::BOTTOM },
                        Collider::new(
                            ColliderMask::Landscape,
                            vec![
                                ColliderMask::Bullet,
                                ColliderMask::Custom("BORDER_CUSTOM".to_string()),
                                ColliderMask::Custom("BORDER_TOP".to_string()),
                                ColliderMask::Custom("BORDER_BOTTOM".to_string()),
                                ColliderMask::Custom("BORDER_LEFT".to_string()),
                                ColliderMask::Custom("BORDER_RIGHT".to_string()),
                            ],
                            ColliderType::RectangleCollider(16, 16),
                        ),
                    ));
                } else if self.mouse_state == CursorState::COLUMN {
                    world.push((
                        Transform::from_xyz(x as f32 * 16. + 10., y as f32 * 16. + 10., 20),
                        Rectangle::new(16., 16., None),
                        Material::Diffuse(Color::new_hex("#ff00004c")),
                        Line { direction: LineDirection::LEFT },
                        Collider::new(
                            ColliderMask::Landscape,
                            vec![
                                ColliderMask::Bullet,
                                ColliderMask::Custom("BORDER_CUSTOM".to_string()),
                                ColliderMask::Custom("BORDER_TOP".to_string()),
                                ColliderMask::Custom("BORDER_BOTTOM".to_string()),
                                ColliderMask::Custom("BORDER_LEFT".to_string()),
                                ColliderMask::Custom("BORDER_RIGHT".to_string()),
                            ],
                            ColliderType::RectangleCollider(16, 16),
                        ),
                    ));
                    world.push((
                        Transform::from_xyz(x as f32 * 16. + 26., y as f32 * 16. + 10., 20),
                        Rectangle::new(16., 16., None),
                        Material::Diffuse(Color::new_hex("#0000ff4c")),
                        Line { direction: LineDirection::RIGHT },
                        Collider::new(
                            ColliderMask::Landscape,
                            vec![
                                ColliderMask::Bullet,
                                ColliderMask::Custom("BORDER_CUSTOM".to_string()),
                                ColliderMask::Custom("BORDER_TOP".to_string()),
                                ColliderMask::Custom("BORDER_BOTTOM".to_string()),
                                ColliderMask::Custom("BORDER_LEFT".to_string()),
                                ColliderMask::Custom("BORDER_RIGHT".to_string()),
                            ],
                            ColliderType::RectangleCollider(16, 16),
                        ),
                    ));
                }
            }
        });
    }

    fn on_stop(&mut self, data: &mut GameData) {
        let _r = data.remove(self.tilemap.unwrap());
        let mut to_delete: Vec<Entity> = {
            let mut res = Vec::new();
            for (e, _) in data.query::<&Line>().iter() {
                res.push(e);
            }
            res
        };
        to_delete.drain(0..).for_each(|e| {
            let _r = data.remove(e);
        });
    }
}

impl MainScene {
    fn compute_mouse_on_border(&mut self, mouse_pos: (f64, f64), data: &mut GameData) -> bool {
        let (x, y) = mouse_pos;
        if x < 11. {
            if self.state_before_border.is_none() {
                self.state_before_border = Some(self.mouse_state.clone())
            };
            data.window().set_cursor(CursorIcon::EResize);
            self.mouse_state = CursorState::LEFT;
            true
        } else if x > 1097. {
            if self.state_before_border.is_none() {
                self.state_before_border = Some(self.mouse_state.clone())
            };
            data.window().set_cursor(CursorIcon::WResize);
            self.mouse_state = CursorState::RIGHT;
            true
        } else if y < 11. {
            if self.state_before_border.is_none() {
                self.state_before_border = Some(self.mouse_state.clone())
            };
            data.window().set_cursor(CursorIcon::SResize);
            self.mouse_state = CursorState::TOP;
            true
        } else if y > 618. {
            if self.state_before_border.is_none() {
                self.state_before_border = Some(self.mouse_state.clone())
            };
            data.window().set_cursor(CursorIcon::NResize);
            self.mouse_state = CursorState::BOTTOM;
            true
        } else {
            false
        }
    }

    fn compute_new_icon(&mut self) -> CursorIcon {
        if self.mouse_state == CursorState::ROW {
            self.mouse_state = CursorState::COLUMN;
            return CursorIcon::ColResize;
        }
        self.mouse_state = CursorState::ROW;
        CursorIcon::RowResize
    }

    fn change_cursor(&self, data: &mut GameData) {
        let mut window = data.window();
        window.set_cursor(match self.mouse_state {
            CursorState::TOP => CursorIcon::NResize,
            CursorState::BOTTOM => CursorIcon::NResize,
            CursorState::LEFT => CursorIcon::WResize,
            CursorState::RIGHT => CursorIcon::WResize,
            CursorState::ROW => CursorIcon::RowResize,
            CursorState::COLUMN => CursorIcon::ColResize,
        })
    }
}
pub fn init_balls(world: &mut SubWorld, assets: &JezzBallAssets) {
    let x = thread_rng().gen_range(10..58) as f32;
    let y = thread_rng().gen_range(5..33) as f32;

    world.push((
        Transform::from_xyz(10. + x * 16., 10. + y * 16., 15),
        Sprite::new(0),
        assets.ball_asset.clone(),
        Ball::default(),
        Collider::new(
            ColliderMask::Bullet,
            vec![
                ColliderMask::Custom("BORDER_TOP".to_string()),
                ColliderMask::Custom("BORDER_CUSTOM_HORIZONTAL".to_string()),
                ColliderMask::Custom("BORDER_CUSTOM_VERTICAL".to_string()),
                ColliderMask::Custom("BORDER_BOTTOM".to_string()),
                ColliderMask::Custom("BORDER_LEFT".to_string()),
                ColliderMask::Custom("BORDER_RIGHT".to_string()),
            ],
            ColliderType::SquareCollider(38),
        )
        .with_offset(Vector::new(-5., -5.)),
        ball_animations(),
    ));
}

fn add_border(world: &mut SubWorld) {
    world.push((
        Transform::from_xy(2.0, 0.),
        Collider::new(
            ColliderMask::Custom("BORDER_TOP".to_string()),
            vec![ColliderMask::None],
            ColliderType::RectangleCollider(1104, 10),
        ),
    ));

    world.push((
        Transform::from_xy(2.0, 619.),
        Collider::new(
            ColliderMask::Custom("BORDER_BOTTOM".to_string()),
            vec![ColliderMask::None],
            ColliderType::RectangleCollider(1104, 10),
        ),
    ));

    world.push((
        Transform::from_xy(1.0, 10.),
        Collider::new(
            ColliderMask::Custom("BORDER_LEFT".to_string()),
            vec![ColliderMask::None],
            ColliderType::RectangleCollider(10, 609),
        ),
    ));

    world.push((
        Transform::from_xy(1098., 10.),
        Collider::new(
            ColliderMask::Custom("BORDER_RIGHT".to_string()),
            vec![ColliderMask::None],
            ColliderType::RectangleCollider(10, 609),
        ),
    ));
}

fn pathfind_from(
    pos: (usize, usize),
    data: &mut GameData,
    tilemap: Entity,
    visited: &mut [[bool; 38]; 68],
) -> Vec<(usize, usize)> {
    let mut res = Vec::new();
    let sides = compute_sides(pos);
    sides.iter().for_each(|side_pos| {
        if !visited[side_pos.0][side_pos.1] {
            visited[side_pos.0][side_pos.1] = true;
            let sprite = Tilemap::retrieve_sprite_tile(
                data,
                tilemap,
                &Position::new(side_pos.0, side_pos.1, 0),
            )
            .unwrap_or(2);
            if sprite != 2 {
                res.push(*side_pos);
                let mut recursive_call_result = pathfind_from(*side_pos, data, tilemap, visited);
                res.append(&mut recursive_call_result);
            }
        }
    });
    res
}

fn compute_sides(pos: (usize, usize)) -> Vec<(usize, usize)> {
    let mut res = Vec::new();

    if pos.0 > 0 {
        res.push((pos.0 - 1, pos.1));
    }

    if pos.0 < 67 {
        res.push((pos.0 + 1, pos.1));
    }

    if pos.1 > 0 {
        res.push((pos.0, pos.1 - 1));
    }

    if pos.1 < 37 {
        res.push((pos.0, pos.1 + 1));
    }
    res
}

fn add_tilemap(data: &mut GameData) -> Entity {
    let cases_asset = data.assets_mut().register_tileset(Tileset::new("cases".to_string(), cases_asset(), 3, 2, 16, 16));
    let infos = TilemapInfo::new(
        Dimensions::new(68, 38, 1),
        Transform::from_xyz(10., 10., 95),
        cases_asset,
        TilemapType::Standard
    );
    Tilemap::create(infos, data, |position| {
        let line = position.y() % 2;
        let column = position.x() % 2;
        TileInfos::new(Some(line * 3 + column), None)
    })
}
