use std::collections::HashMap;

use scion::core::components::maths::transform::Transform;
use scion::core::world::{GameData, World};

use crate::{
    components::{Bloc, BlocKind, BLOC_SIZE, BOARD_HEIGHT},
    resources::TetrisResource,
};

pub fn score_system(data: &mut GameData) {
    let mut lines = HashMap::new();
    for i in 1..=BOARD_HEIGHT {
        lines.insert(i as usize, 0);
    }
    for (_, (bloc, transform)) in data.query_mut::<(&Bloc, &mut Transform)>() {
        match bloc.kind {
            BlocKind::Static => {
                let line_idx = (transform.translation().y() / BLOC_SIZE) as usize;
                let bloc_counter = match lines.get(&line_idx) {
                    Some(val) => val + 1,
                    None => 1,
                };
                lines.insert(line_idx, bloc_counter);
            }
            _ => {}
        }
    }

    let full_lines = {
        let mut full_lines = Vec::new();
        for (line_idx, bloc_counter) in lines.iter() {
            if bloc_counter == &10 {
                data.get_resource_mut::<TetrisResource>().unwrap().score += 1;
                full_lines.push(*line_idx);
            }
        }
        full_lines.sort_unstable();
        full_lines
    };

    let mut to_remove = Vec::new();

    if !full_lines.is_empty() {
        for (entity, (bloc, transform)) in data.query_mut::<(&Bloc, &mut Transform)>() {
            match bloc.kind {
                BlocKind::Static => {
                    let line_idx = (transform.translation().y() / BLOC_SIZE) as usize;
                    if full_lines.contains(&line_idx) {
                        to_remove.push(entity);
                    }
                }
                _ => {}
            };
        }
        let (world, _, com) = data.split_with_command();
        for full_line_idx in full_lines.iter() {
            for (e, (bloc, transform)) in world.query_mut::<(&Bloc, &mut Transform)>() {
                match bloc.kind {
                    BlocKind::Static => {
                        let line_idx = (transform.translation().y() / BLOC_SIZE) as usize;
                        if *full_line_idx > line_idx {
                            com.transform_commands.append_y(e,  BLOC_SIZE);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    to_remove.drain(0..).for_each(|e| {
        let _r = data.remove(e);
    });
}
