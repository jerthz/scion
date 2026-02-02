use scion::core::components::maths::transform::{Transform, TransformBuilder};
use scion::graphics::components::material::Material;
use scion::core::resources::asset_manager::AssetRef;
use scion::core::world::{GameData, World};
use scion::graphics::components::tiles::sprite::Sprite;

use crate::{
    components::{Bloc, BlocKind, NextBloc, BLOC_SIZE, BOARD_HEIGHT, BOARD_OFFSET},
    resources::{TetrisResource, TetrisState},
};

pub fn piece_update_system(data: &mut GameData) {
    let (world, resources, commands) = data.split_with_command();

    let mut timers = resources.timers();
    let mut tetris = resources.get_resource_mut::<TetrisResource>().unwrap();

    let mut to_remove = Vec::new();
    let mut to_add = Vec::new();

    let timer =
        timers.get_timer("piece").expect("Missing a mandatory timer in the game : piece timer");
    if timer.cycle() > 0 {
        match tetris.state {
            TetrisState::WAITING => {
                tetris.switch_to_next_piece();
                for (e, _) in world.query::<&NextBloc>().iter() {
                    to_remove.push(e);
                }
                let offsets = tetris.next_piece.get_current_offsets();
                for offset in offsets {
                    to_add.push(initialize_bloc(&offset, &mut tetris, 12., 2., true));
                }

                let offsets = tetris.active_piece.get_current_offsets();
                for offset in offsets {
                    to_add.push(initialize_bloc(&offset, &mut tetris, 4., 0., false));
                }
            }
            TetrisState::MOVING(x, y) => {
                let mut static_values: Vec<(u32, u32)> = Vec::new();
                let mut piece_values: Vec<(u32, u32)> = Vec::new();
                for (_e, (bloc, transform)) in world.query_mut::<(&mut Bloc, &mut Transform)>() {
                    let t = (
                        ((transform.translation().x() - BOARD_OFFSET.0) / BLOC_SIZE) as u32,
                        ((transform.translation().y() - BOARD_OFFSET.1) / BLOC_SIZE) as u32,
                    );
                    match bloc.kind {
                        BlocKind::Moving => piece_values.push(t),
                        _ => static_values.push(t),
                    };
                }
                let should_move_piece = {
                    let mut res = true;
                    for (x, y) in piece_values.iter() {
                        for (xx, yy) in static_values.iter() {
                            if x == xx && y == &(yy - 1) {
                                res = false;
                            }
                        }
                        if y == &(BOARD_HEIGHT - 1) {
                            res = false;
                        }
                    }
                    res
                };
                if should_move_piece {
                    for (z, (bloc, _)) in world.query_mut::<(&mut Bloc, &mut Transform)>() {
                        match bloc.kind {
                            BlocKind::Moving => {
                                commands.transform_commands.append_y(z, BLOC_SIZE);
                                tetris.state = TetrisState::MOVING(x, y + 1);
                            }
                            _ => {}
                        };
                    }
                } else {
                    for (_, (bloc, _)) in world.query_mut::<(&mut Bloc, &mut Transform)>() {
                        match bloc.kind {
                            BlocKind::Moving => {
                                bloc.kind = BlocKind::Static;
                                tetris.state = TetrisState::WAITING;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    to_remove.drain(0..).for_each(|e| {
        let _r = world.remove(e);
    });
    to_add.drain(0..).for_each(|comps| {
        if comps.3 {
            let _r = world.push((comps.0, comps.1, comps.2, NextBloc));
        } else {
            let _r = world.push((comps.0, comps.1, comps.2, Bloc::new(BlocKind::Moving)));
        }
    });
}

pub fn initialize_bloc(
    offset: &(f32, f32),
    tetris: &TetrisResource,
    coord_x: f32,
    coord_y: f32,
    is_next_bloc: bool,
) -> (Transform, Sprite, AssetRef<Material>, bool) {
    let bloc_transform = TransformBuilder::default().with_translation(
        coord_x * BLOC_SIZE + offset.0 * BLOC_SIZE,
        coord_y * BLOC_SIZE + offset.1 * BLOC_SIZE,
        1
    ).build();
    let tuple = (
        bloc_transform,
        Sprite::new(if is_next_bloc { tetris.next_piece.color } else { tetris.active_piece.color }),
        tetris.asset.as_ref().unwrap().clone(),
    );

    if is_next_bloc {
        (tuple.0, tuple.1, tuple.2, is_next_bloc)
    } else {
        (tuple.0, tuple.1, tuple.2, is_next_bloc)
    }
}
