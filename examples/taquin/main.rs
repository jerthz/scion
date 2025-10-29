mod animations;

use rand::Rng;
use scion::core::resources::audio::PlayConfig;
use scion::core::world::{GameData, World};
use scion::{
    config::{scion_config::ScionConfigBuilder, window_config::WindowConfigBuilder},
    core::{
        scene::Scene,
    },
    graphics::components::{
        tiles::{sprite::Sprite, tileset::Tileset},
    },
    utils::file::app_base_path,
    Scion,
};
use scion::core::components::maths::coordinates::Coordinates;
use scion::core::components::maths::transform::Transform;
use scion::graphics::components::animations::Animations;
use scion::graphics::components::color::Color;
use crate::animations::get_case_animation;

const N: usize = 4;
const TILE: f32 = 192.0;

#[derive(Debug)]
struct Case(Coordinates);

#[derive(PartialEq)]
pub(crate) enum MoveDirection {
    Left, Top, Right, Bottom, None,
}

struct Taquin {
    empty: (usize, usize),
}

impl Taquin {
    fn new(cases: &[Option<usize>]) -> Self {
        let idx = cases.iter().position(|c| c.is_none()).unwrap_or(N * N - 1);
        Self { empty: (idx % N, idx / N) }
    }

    fn try_move(&mut self, col: usize, row: usize) -> MoveDirection {
        let (ex, ey) = self.empty;
        let dir = if col > 0 && (col - 1, row) == (ex, ey) {
            MoveDirection::Left
        } else if row > 0 && (col, row - 1) == (ex, ey) {
            MoveDirection::Top
        } else if col + 1 < N && (col + 1, row) == (ex, ey) {
            MoveDirection::Right
        } else if row + 1 < N && (col, row + 1) == (ex, ey) {
            MoveDirection::Bottom
        } else {
            MoveDirection::None
        };

        if dir != MoveDirection::None {
            self.empty = (col, row);
        }
        dir
    }
}

fn taquin_system(data: &mut GameData) {
    let (world, resources) = data.split();
    let inputs = resources.inputs();
    let (w, h) = resources.window().dimensions();
    let (cw, ch) = (w as f32 / N as f32, h as f32 / N as f32);
    let mut taquin = resources.get_resource_mut::<Taquin>().unwrap();

    // skip si une animation est en cours
    if world.query_mut::<&mut Animations>().into_iter().any(|(_, a)| a.any_animation_running()) {
        return;
    }

    for (_, (case, anim)) in world.query_mut::<(&mut Case, &mut Animations)>() {
        inputs.on_left_click_pressed(|mx, my| {
            let (x, y) = (case.0.x(), case.0.y());
            let inside = (mx as f32) >= x * cw && (mx as f32) <= x * cw + cw
                      && (my as f32) >= y * ch && (my as f32) <= y * ch + ch;

            if !inside { return; }

            match taquin.try_move(x as usize, y as usize) {
                MoveDirection::Left   => { case.0.set_x(x - 1.0); anim.run_animation("LEFT"); }
                MoveDirection::Top    => { case.0.set_y(y - 1.0); anim.run_animation("TOP"); }
                MoveDirection::Right  => { case.0.set_x(x + 1.0); anim.run_animation("RIGHT"); }
                MoveDirection::Bottom => { case.0.set_y(y + 1.0); anim.run_animation("BOTTOM"); }
                MoveDirection::None   => return,
            }

            let _ = resources.audio().play(
                app_base_path().join("examples/taquin/assets/tap.ogg").get(),
                PlayConfig::default(),
            );
        });
    }
}

#[derive(Default)]
struct MainScene;

impl Scene for MainScene {
    fn on_start(&mut self, data: &mut GameData) {
        let tileset_ref = data.assets_mut().register_tileset(Tileset::new(
            "taquin_texture".into(),
            app_base_path().join("examples/taquin/assets/taquin.png").get(),
            N as usize, N as usize, TILE as usize, TILE as usize,
        ));

        let cases = compute_mixed_cases();

        for i in 0..N * N {
            if let Some(tile) = cases[i] {
                let col = (i % N) as f32;
                let row = (i / N) as f32;
                data.push((
                    Transform::from_xy(col * TILE, row * TILE),
                    tileset_ref.clone(),
                    Sprite::new(tile),
                    Case(Coordinates::new(col, row)),
                    Animations::new(get_case_animation()),
                ));
            }
        }

        data.add_default_camera();
        data.insert_resource(Taquin::new(&cases));
    }
}

fn compute_mixed_cases() -> Vec<Option<usize>> {
    // plateau: 0..15 et une case vide à la fin
    let mut v: Vec<Option<usize>> = (0..(N * N - 1)).map(Some).collect();
    v.push(None);

    // petit mélange simple (pas forcément solvable, même comportement que l’original)
    let mut rng = rand::thread_rng();
    for _ in 0..300 {
        let a = rng.gen_range(0..v.len());
        let b = rng.gen_range(0..v.len());
        v.swap(a, b);
    }
    v
}

fn main() {
    Scion::app_with_config(
        ScionConfigBuilder::new()
            .with_window_config(
                WindowConfigBuilder::new()
                    .with_resizable(true)
                    .with_dimensions((768, 768))
                    .with_default_background_color(Some(Color::new_hex("#000000")))
                    .get(),
            )
            .get(),
    )
    .with_system(taquin_system)
    .with_scene::<MainScene>()
    .run();
}
 