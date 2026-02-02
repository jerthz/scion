use scion::core::world::{GameData, World};
use scion::{
    graphics::components::{
        shapes::rectangle::Rectangle,
    },
    core::resources::time::TimerType,
};
use scion::core::components::maths::collider::{Collider, ColliderMask, ColliderType};
use scion::core::components::maths::transform::Transform;

use crate::main_scene::Line;
use crate::main_scene::LineDirection;

const LINE_SPEED: f32 = 4.;

pub fn line_update_system(data: &mut GameData) {
    let (world, resources, commands) = data.split_with_command();
    let mut timers = resources.timers();
    let mut events = resources.events();

    let mut rectange_to_remove = Vec::new();
    let mut entities_to_remove = Vec::new();
    let mut collider_to_add = Vec::new();

    for (entity, (line, transform, rectangle, collider)) in
        world.query_mut::<(&Line, &mut Transform, &mut Rectangle, &Collider)>()
    {
        let timer_name = format!("{:?}_line", entity);
        let timer_str = timer_name.as_str();
        if !timers.exists(timer_str) {
            let _r = timers.add_timer(timer_str, TimerType::Cyclic, 0.02);
        }
        let timer = timers.get_timer(timer_str).unwrap();

        if timer.cycle() > 0 {
            match line.direction {
                LineDirection::LEFT => {
                    rectangle.set_width(rectangle.width() + LINE_SPEED);
                    commands.transform_commands.append_x(entity, -LINE_SPEED);
                }
                LineDirection::RIGHT => {
                    rectangle.set_width(rectangle.width() + LINE_SPEED);
                }
                LineDirection::TOP => {
                    commands.transform_commands.append_y(entity, -LINE_SPEED);
                    rectangle.set_height(rectangle.height() + LINE_SPEED)
                }
                LineDirection::BOTTOM => rectangle.set_height(rectangle.height() + LINE_SPEED),
            }

            if collider.is_colliding() {
                if collider
                    .collisions()
                    .iter()
                    .filter(|e| e.mask() == &ColliderMask::Bullet)
                    .count()
                    > 0
                {
                    entities_to_remove.push(entity);
                } else {
                    match line.direction {
                        LineDirection::LEFT | LineDirection::RIGHT => {
                            let y = (transform.translation().y() - 10.) as usize / 16;
                            let _r = events.publish(
                                "TILEMAP_UPDATE",
                                (
                                    ((transform.translation().x()) - 10.) as usize / 16,
                                    (((transform.translation().x() + rectangle.width()) - 10.)
                                        as usize
                                        / 16)
                                        - 1,
                                    y,
                                    y,
                                ),
                            );
                            collider_to_add.push((
                                entity,
                                Collider::new(
                                    ColliderMask::Custom("BORDER_CUSTOM_HORIZONTAL".to_string()),
                                    vec![],
                                    ColliderType::RectangleCollider(
                                        rectangle.width() as usize,
                                        rectangle.height() as usize,
                                    ),
                                ),
                            ));
                        }
                        LineDirection::TOP | LineDirection::BOTTOM => {
                            let x = (transform.translation().x() - 10.) as usize / 16;
                            let _r = events.publish(
                                "TILEMAP_UPDATE",
                                (
                                    x,
                                    x,
                                    (transform.translation().y() - 10.) as usize / 16,
                                    (((transform.translation().y() + rectangle.height()) - 10.)
                                        as usize
                                        / 16)
                                        - 1,
                                ),
                            );
                            collider_to_add.push((
                                entity,
                                Collider::new(
                                    ColliderMask::Custom("BORDER_CUSTOM_VERTICAL".to_string()),
                                    vec![],
                                    ColliderType::RectangleCollider(
                                        rectangle.width() as usize,
                                        rectangle.height() as usize,
                                    ),
                                ),
                            ));
                        }
                    }

                    rectange_to_remove.push(entity);
                }
            } else {
                collider_to_add.push((
                    entity,
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
                        ColliderType::RectangleCollider(
                            rectangle.width() as usize,
                            rectangle.height() as usize,
                        ),
                    ),
                ));
            }
        }
    }
    rectange_to_remove.drain(0..).for_each(|e| {
        let _r = world.remove_component::<Rectangle>(e);
    });
    entities_to_remove.drain(0..).for_each(|e| {
        let _r = world.remove(e);
    });
    collider_to_add.drain(0..).for_each(|e| {
        let _r = world.add_components(e.0, (e.1,));
    });
}
