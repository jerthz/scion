use std::{collections::HashMap, time::Duration};

use scion::{
    graphics::components::animations::{Animation, AnimationModifier, Animations},
    utils::{file::app_base_path, maths::Vector},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(300);
const BALL_SPEED: f32 = 30.;

pub fn ball_asset() -> String {
    app_base_path().join("examples/jezzball/assets/ball.png").get()
}

pub fn cases_asset() -> String {
    app_base_path().join("examples/jezzball/assets/cases.png").get()
}

pub fn ball_animations() -> Animations {
    let mut animations = HashMap::new();
    animations.insert(
        "TopLeft".to_string(),
        Animation::new(
            ANIMATION_DURATION,
            vec![
                AnimationModifier::sprite(vec![10, 9, 8, 3], 3),
                AnimationModifier::transform(
                    15,
                    Some(Vector::new(-BALL_SPEED, -BALL_SPEED)),
                    None,
                    None,
                ),
            ],
        ),
    );
    animations.insert(
        "BottomLeft".to_string(),
        Animation::new(
            ANIMATION_DURATION,
            vec![
                AnimationModifier::sprite(vec![12, 13, 14, 3], 3),
                AnimationModifier::transform(
                    15,
                    Some(Vector::new(-BALL_SPEED, BALL_SPEED)),
                    None,
                    None,
                ),
            ],
        ),
    );
    animations.insert(
        "TopRight".to_string(),
        Animation::new(
            ANIMATION_DURATION,
            vec![
                AnimationModifier::sprite(vec![14, 13, 12, 3], 3),
                AnimationModifier::transform(
                    15,
                    Some(Vector::new(BALL_SPEED, -BALL_SPEED)),
                    None,
                    None,
                ),
            ],
        ),
    );
    animations.insert(
        "BottomRight".to_string(),
        Animation::new(
            ANIMATION_DURATION,
            vec![
                AnimationModifier::sprite(vec![8, 9, 10, 3], 3),
                AnimationModifier::transform(
                    15,
                    Some(Vector::new(BALL_SPEED, BALL_SPEED)),
                    None,
                    None,
                ),
            ],
        ),
    );
    Animations::new(animations)
}
