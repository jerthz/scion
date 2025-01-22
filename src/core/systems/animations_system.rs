use crate::core::components::maths::transform::Transform;
use crate::core::world::{GameData, SubWorld, World};
use crate::graphics::components::ui::ui_text::UiText;
use crate::{
    core::resources::time::TimerType,
    graphics::components::{
        animations::{
            AnimationModifier, AnimationModifierType, AnimationStatus, Animations,
            ComputedKeyframeModifier,
        },
        color::Color,
        material::Material,
        tiles::sprite::Sprite,
        Hide,
    },
};
use crate::graphics::components::HidePropagated;

#[derive(PartialEq)]
enum BlinkResult {
    REMOVE,
    ADD,
}

/// System responsible of applying modifiers data to the dedicated components
/// It will use timers to keep track of the animation and will merge keyframes in case
/// of long frames.
pub(crate) fn animation_executer_system(data: &mut GameData) {

    let (subworld, resources) = data.split();
    start_delayed_animation(subworld);
    let mut timers = resources.timers();
    let mut remove_blink = Vec::new();
    let mut add_blink = Vec::new();
    for (entity, (animations, mut transform, mut sprite, mut material, mut text, hide)) in subworld
        .query_mut::<(
            &mut Animations,
            Option<&mut Transform>,
            Option<&mut Sprite>,
            Option<&mut Material>,
            Option<&mut UiText>,
            Option<&Hide>,
        )>()
    {
        animations
            .animations_mut()
            .iter_mut()
            .filter(|(_, v)| !matches!(v.status, AnimationStatus::Stopped | AnimationStatus::WaitingStartTime(_)))
            .for_each(|(key, animation)| {
                for modifier in animation.modifiers.iter_mut() {
                    let mut timer_created = false;
                    let timer_id = format!("{:?}-{}-{}", entity, key, modifier);
                    let timer_id = timer_id.as_str();
                    if let Ok(timer) = timers.add_timer(
                        timer_id,
                        TimerType::Cyclic,
                        modifier
                            .single_keyframe_duration
                            .expect("Single keyframe duration is missing for animations")
                            .as_secs_f32(),
                    ) {
                        timer.reset();
                        timer_created = true;
                    }

                    let timer_cycle = {
                        let cycles = timers.get_timer(timer_id).expect("Timer must exist").cycle();
                        let keyframes_left =
                            modifier.number_of_keyframes - modifier.current_keyframe;
                        if cycles > keyframes_left {
                            keyframes_left
                        } else {
                            cycles
                        }
                    };

                    if timer_cycle > 0
                        || timer_created
                        || &animation.status == &AnimationStatus::ForceStopped
                    {
                        match modifier.modifier_type.clone() {
                            AnimationModifierType::TransformModifier { .. } => {
                                apply_transform_modifier(transform.as_mut(), modifier, timer_cycle)
                            }
                            AnimationModifierType::SpriteModifier {
                                tile_numbers,
                                tile_numbers_variant,
                                end_tile_number,
                            } => apply_sprite_modifier(
                                sprite.as_mut(),
                                &animation.status,
                                modifier,
                                &tile_numbers,
                                &tile_numbers_variant,
                                end_tile_number,
                            ),
                            AnimationModifierType::ColorModifier { .. } => {
                                apply_color_modifier(material.as_mut(), modifier, timer_cycle)
                            }
                            AnimationModifierType::Blink => {
                                match apply_blink_modifier(modifier, timer_cycle, hide) {
                                    Some(action) if action == BlinkResult::ADD => {
                                        add_blink.push(entity)
                                    }
                                    Some(action) if action == BlinkResult::REMOVE => {
                                        remove_blink.push(entity)
                                    }
                                    _ => {}
                                }
                            }
                            AnimationModifierType::Text { content: _ } => {
                                apply_text_modifier(modifier, text.as_mut())
                            }
                        }
                        modifier.current_keyframe += timer_cycle;
                        if modifier.current_keyframe >= modifier.number_of_keyframes {
                            modifier.next_sprite_index = None;
                            let _r = timers.delete_timer(timer_id);
                        }
                    }
                }
                animation.try_update_status();
            });

        animations
            .animations_mut()
            .iter_mut()
            .filter(|(_, v)| v.status == AnimationStatus::Stopped)
            .for_each(|(key, animation)| {
                for modifier in animation.modifiers.iter_mut() {
                    let timer_id = format!("{:?}-{}-{}", entity, key, modifier);
                    let _r = timers.delete_timer(timer_id.as_str());
                }
            });
    }

    remove_blink.drain(0..).for_each(|e| {
        let _r = subworld.remove_component::<Hide>(e);
    });
    add_blink.drain(0..).for_each(|e| {
        let _r = subworld.add_components(e, (Hide,));
    });
}

fn start_delayed_animation(world: &mut SubWorld) {
    for (_, animation) in world.query_mut::<&mut Animations>()
        .without::<&Hide>()
        .without::<&HidePropagated>() {
        animation.run_eligible_delayed_animations();
    }
}

fn apply_transform_modifier(
    mut transform: Option<&mut &mut Transform>,
    modifier: &mut AnimationModifier,
    timer_cycle: usize,
) {
    if let ComputedKeyframeModifier::TransformModifier { vector: coordinates, scale, rotation } =
        modifier.retrieve_keyframe_modifier()
    {
        if let Some(ref mut transform) = transform {
            for _i in 0..timer_cycle {
                if let Some(coordinates) = coordinates {
                    transform.append_translation(coordinates.x(), coordinates.y());
                }
                if let Some(scale) = scale {
                    transform.set_scale(transform.scale + scale);
                }
                if let Some(rotation) = rotation {
                    transform.append_angle(*rotation);
                }
            }
        }
    }
}

fn apply_sprite_modifier(
    mut sprite: Option<&mut &mut Sprite>,
    status: &AnimationStatus,
    modifier: &mut AnimationModifier,
    tile_numbers: &Vec<usize>,
    tile_numbers_variant: &Option<Vec<usize>>,
    end_tile_number: usize,
) {
    if let Some(ref mut animation_sprite) = sprite {
        if status == &AnimationStatus::ForceStopped {
            return;
        }

        if modifier.next_sprite_index.is_none() {
            modifier.next_sprite_index = Some(0);
        }
        if modifier.current_keyframe == (modifier.number_of_keyframes - 1)
            && status != &AnimationStatus::Looping
        {
            animation_sprite.set_tile_nb(end_tile_number);
            modifier.next_sprite_index = None;
            modifier.variant = !modifier.variant;
        } else {
            if tile_numbers_variant.is_some() && modifier.variant {
                animation_sprite.set_tile_nb(
                    *tile_numbers_variant
                        .as_ref()
                        .unwrap()
                        .get(modifier.next_sprite_index.unwrap())
                        .unwrap(),
                );
            } else {
                animation_sprite
                    .set_tile_nb(*tile_numbers.get(modifier.next_sprite_index.unwrap()).unwrap());
            }

            if modifier.next_sprite_index.unwrap() >= modifier.number_of_keyframes {
                modifier.next_sprite_index.replace(0);
                modifier.variant = !modifier.variant;
            } else {
                modifier.next_sprite_index.replace(modifier.next_sprite_index.unwrap() + 1);
            }
        }
    }
}

fn apply_color_modifier(
    material: Option<&mut &mut Material>,
    modifier: &mut AnimationModifier,
    timer_cycle: usize,
) {
    if let Some(Material::Diffuse(ref mut color)) = material {
        if modifier.is_first_frame() {
            modifier.compute_keyframe_modifier_for_animation(color);
        }
        if let Some(ComputedKeyframeModifier::Color { r, g, b, a }) =
            &modifier.single_keyframe_modifier
        {
            for i in 0..timer_cycle {
                if modifier.will_be_last_keyframe(i) {
                    if let AnimationModifierType::ColorModifier { target } = &modifier.modifier_type {
                        color.replace(target.clone())
                    }
                } else {
                    let new_color = Color::new(
                        (r + color.red() as i16) as u8,
                        (g + color.green() as i16) as u8,
                        (b + color.blue() as i16) as u8,
                        (a + color.alpha()).max(0.).min(1.0),
                    );
                    color.replace(new_color);
                }
            }
        }
    }
}

fn apply_blink_modifier(
    modifier: &mut AnimationModifier,
    timer_cycle: usize,
    hide: Option<&Hide>,
) -> Option<BlinkResult> {
    if timer_cycle > 0 {
        if modifier.will_be_last_keyframe(timer_cycle) {
            return Some(BlinkResult::REMOVE);
        } else if hide.is_none() {
            return Some(BlinkResult::ADD);
        } else {
            return Some(BlinkResult::REMOVE);
        }
    }
    None
}

fn apply_text_modifier(modifier: &mut AnimationModifier, mut text: Option<&mut &mut UiText>) {
    let mut next_cursor = 0;
    if let ComputedKeyframeModifier::Text { cursor } = modifier.retrieve_keyframe_modifier() {
        if let Some(ref mut uitext) = text {
            if let AnimationModifierType::Text { content } = modifier.modifier_type() {
                let res = content.as_str()[..*cursor].to_string();
                uitext.set_text(res);
                next_cursor = if *cursor < content.len() { cursor + 1 } else { 0 };
            }
        }
    }
    if let ComputedKeyframeModifier::Text { ref mut cursor } =
        modifier.retrieve_keyframe_modifier_mut()
    {
        *cursor = next_cursor;
    }
}
