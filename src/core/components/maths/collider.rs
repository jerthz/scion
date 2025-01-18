use geo_clipper::Clipper;
use geo_types::{Coord, LineString};
use hecs::Entity;

use crate::core::components::maths::{coordinates::Coordinates, transform::Transform, Pivot};
use crate::utils::maths::{centroid_polygon, rotate_point_around_pivot, Vector};

/// `ColliderMask` will serve as a 'mask' to allow filter while collisions happen
#[derive(PartialEq, Clone, Eq, Hash, Debug)]
pub enum ColliderMask {
    None,
    Character,
    Bullet,
    Death,
    Landscape,
    Item,
    Custom(String),
}

/// `ColliderType` will determine the shape of the collider.
#[derive(Clone)]
pub enum ColliderType {
    SquareCollider(usize),
    RectangleCollider(usize, usize),
    PolygonCollider(Vec<Coordinates>)
}

/// The main collider representation to add to an entity, using the new function
#[derive(Clone)]
pub struct Collider {
    collider_mask: ColliderMask,
    collider_type: ColliderType,
    collision_filter: Vec<ColliderMask>,
    collisions: Vec<Collision>,
    offset: Vector,
    debug_lines: bool,
    local_pivot: Option<Pivot>,
    parent_pivot: Option<Pivot>,
}

impl Collider {
    /// Creates a new collider. Note that an empty collision_filter means that this colliders
    /// won't collide
    pub fn new(
        collider_mask: ColliderMask,
        collision_filter: Vec<ColliderMask>,
        collider_type: ColliderType,
    ) -> Self {
        Collider {
            collider_mask,
            collider_type,
            collision_filter,
            collisions: vec![],
            offset: Vector::default(),
            debug_lines: false,
            local_pivot: None,
            parent_pivot: None,
        }
    }

    pub fn with_debug_lines(mut self) -> Self {
        self.debug_lines = true;
        self
    }

    pub fn with_custom_pivot(mut self, pivot: Pivot) -> Self {
        self.local_pivot = Some(pivot);
        self
    }

    pub fn with_offset(mut self, offset: Vector) -> Self {
        self.offset = offset;
        self
    }

    /// Return whether or not this collider colliding to any other collider ?
    pub fn is_colliding(&self) -> bool {
        !self.collisions.is_empty()
    }

    /// Returns an iterator of current collisions
    pub fn collisions(&self) -> &Vec<Collision> {
        &self.collisions
    }

    /// The mask of this collider
    pub fn mask(&self) -> &ColliderMask {
        &self.collider_mask
    }

    /// The mask of this collider
    pub fn mask_cloned(&self) -> ColliderMask {
        self.collider_mask.clone()
    }

    /// The filters of this collider
    pub fn filters(&self) -> &Vec<ColliderMask> {
        &self.collision_filter
    }

    /// Retrieve the collider type of this collider
    pub fn collider_type(&self) -> &ColliderType {
        &self.collider_type
    }

    /// Retrieve the offset of this collider
    pub fn offset(&self) -> &Vector {
        &self.offset
    }

    pub(crate) fn debug_lines(&self) -> bool {
        self.debug_lines
    }

    pub(crate) fn clear_collisions(&mut self) {
        self.collisions.clear();
    }
    pub(crate) fn set_parent_pivot(&mut self, parent_pivot: Pivot) {
        self.parent_pivot = Some(parent_pivot);
    }

    pub(crate) fn get_pivot(&self) -> Pivot {
        if self.local_pivot.is_some() {
            self.local_pivot.unwrap()
        } else if self.parent_pivot.is_some() {
            self.parent_pivot.unwrap()
        } else {
            Pivot::TopLeft
        }
    }

    pub(crate) fn collider_polygon(&self, transform: &Transform) -> geo_types::Polygon::<f32> {
        let base_x = transform.global_translation.x + self.offset.x;
        let base_y = transform.global_translation.y + self.offset.y;
        let vec = self.collider_coordinates(base_x, base_y);
        let pivot_point = match self.get_pivot() {
            Pivot::TopLeft => { Coordinates::new(base_x, base_y) }
            Pivot::Center => { centroid_polygon(&vec) }
            Pivot::Custom(x,y) => {Coordinates::new(base_x + x,base_y + y)}
        };

        let coords: Vec<Coord<f32>> = vec.iter().map(|c| rotate_point_around_pivot(c, &pivot_point, transform.global_angle))
            .map(|c| {
                Coord { x: c.x, y: c.y }
            })
            .collect();


        geo_types::Polygon::<f32>::new(LineString::<f32>(coords), vec![])
    }

    pub(crate) fn collider_coordinates(&self, base_x: f32, base_y: f32) -> Vec<Coordinates> {
        match self.collider_type() {
            ColliderType::SquareCollider(size) => {
                vec![
                    Coordinates::new(base_x + 0., base_y + 0.),
                    Coordinates::new(base_x + *size as f32, base_y + 0.),
                    Coordinates::new(base_x + *size as f32, base_y + *size as f32),
                    Coordinates::new(base_x + 0., base_y + *size as f32),
                ]
            }
            ColliderType::RectangleCollider(width, height) => {
                vec![
                    Coordinates::new(base_x + 0., base_y + 0.),
                    Coordinates::new(base_x + *width as f32, base_y + 0.),
                    Coordinates::new(base_x + *width as f32, base_y + *height as f32),
                    Coordinates::new(base_x + 0., base_y + *height as f32),
                ]
            }
            ColliderType::PolygonCollider(coordinates) => {
                coordinates.iter().map(|c|  Coordinates::new(base_x + c.x, base_y + c.y)).collect()
            }
        }
    }


    pub(crate) fn can_collide_with(&self, other: &Collider) -> bool {
        self.collision_filter.is_empty() || self.collision_filter.contains(&other.collider_mask)
    }

    pub(crate) fn collides_with(
        &self,
        self_transform: &Transform,
        target_collider: &Collider,
        target_transform: &Transform,
    ) -> Option<CollisionArea> {
        if !self.can_collide_with(target_collider) {
            return None;
        }

        let self_polygon = self.collider_polygon(self_transform);
        let target_polygon = target_collider.collider_polygon(target_transform);
        let result = self_polygon.intersection(&target_polygon, 1.0);

        if !result.0.is_empty() {
            let collision = result.0.first().unwrap();
            let coordinates: Vec<Coordinates> = collision.exterior().0.iter().map(|c| Coordinates::new(c.x, c.y)).collect();
            Some(CollisionArea{
                coordinates
            })
        } else {
            None
        }
    }

    pub(crate) fn add_collisions(&mut self, collisions: &mut Vec<Collision>) {
        self.collisions.append(collisions);
    }
}

/// Representation of a collision
#[derive(Clone, Debug)]
pub struct Collision {
    pub(crate) mask: ColliderMask,
    pub(crate) entity: Entity,
    pub(crate) coordinates: Coordinates,
    pub(crate) collision_area: CollisionArea,
}

impl Collision {
    pub fn entity(&self) -> &Entity {
        &self.entity
    }
    pub fn mask(&self) -> &ColliderMask {
        &self.mask
    }
    pub fn coordinates(&self) -> &Coordinates {
        &self.coordinates
    }
    pub fn area(&self) -> &CollisionArea {
        &self.collision_area
    }
}

#[derive(Clone, Debug)]
pub struct CollisionArea {
    pub(crate) coordinates: Vec<Coordinates>,
}

impl CollisionArea {
    pub fn polygon(&self) -> &Vec<Coordinates> {
        &self.coordinates
    }

    pub fn max_x(&self) -> f32 {
        self.coordinates.iter().map(|c|c.x).reduce(f32::max).unwrap()
    }

    pub fn min_x(&self) -> f32 {
        self.coordinates.iter().map(|c|c.x).reduce(f32::min).unwrap()
    }

    pub fn max_y(&self) -> f32 {
        self.coordinates.iter().map(|c|c.y).reduce(f32::max).unwrap()
    }

    pub fn min_y(&self) -> f32 {
        self.coordinates.iter().map(|c|c.y).reduce(f32::min).unwrap()
    }

}

/// Internal component used to keep track of a collider debug display
pub(crate) struct ColliderDebug;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_collide_with() {
        let bullet = Collider::new(ColliderMask::Bullet, vec![], ColliderType::SquareCollider(5));
        let ship = Collider::new(
            ColliderMask::Character,
            vec![ColliderMask::Bullet],
            ColliderType::SquareCollider(5),
        );
        let land = Collider::new(ColliderMask::Landscape, vec![], ColliderType::SquareCollider(5));

        assert!(!ship.can_collide_with(&land));
        assert!(ship.can_collide_with(&bullet));
    }

    #[test]
    fn test_collides_with_square() {
        let bullet = Collider::new(ColliderMask::Bullet, vec![], ColliderType::SquareCollider(5));
        let ship = Collider::new(ColliderMask::Character, vec![], ColliderType::SquareCollider(5));

        let bullet_transform = Transform::from_xy(4., 4.);
        let bullet_transform2 = Transform::from_xy(9., 9.);
        let ship_transform_in = Transform::from_xy(5., 5.);
        let ship_transform_in2 = Transform::from_xy(8.99999, 8.99999);
        let ship_transform_out = Transform::from_xy(50., 50.);

        assert!(ship.collides_with(&ship_transform_in, &bullet, &bullet_transform).is_some());
        assert!(ship.collides_with(&ship_transform_in2, &bullet, &bullet_transform).is_some());
        assert!(ship.collides_with(&ship_transform_in, &bullet, &bullet_transform2).is_some());
        assert!(bullet.collides_with(&ship_transform_out, &bullet, &bullet_transform).is_none());
    }

    #[test]
    fn test_does_notcollides_with_square_if_offsets_too_far() {
        let mut bullet = Collider::new(
            ColliderMask::Bullet,
            vec![ColliderMask::Character],
            ColliderType::SquareCollider(5),
        );
        bullet = bullet.with_offset(Vector::new(-3., -3.));

        let mut ship = Collider::new(
            ColliderMask::Character,
            vec![ColliderMask::Bullet],
            ColliderType::SquareCollider(5),
        );
        ship = ship.with_offset(Vector::new(3., 3.));

        let bullet_transform = Transform::from_xy(5., 5.);
        let ship_transform = Transform::from_xy(5., 5.);

        assert!(bullet.collides_with(&bullet_transform, &ship, &ship_transform).is_none());
    }

    #[test]
    fn test_does_collides_with_square_if_offsets_close_enough() {
        let mut bullet = Collider::new(
            ColliderMask::Bullet,
            vec![ColliderMask::Character],
            ColliderType::SquareCollider(5),
        );
        bullet = bullet.with_offset(Vector::new(-1., -1.));

        let mut ship = Collider::new(
            ColliderMask::Character,
            vec![ColliderMask::Bullet],
            ColliderType::SquareCollider(5),
        );
        ship = ship.with_offset(Vector::new(1., 1.));

        let bullet_transform = Transform::from_xy(5., 5.);
        let ship_transform = Transform::from_xy(5., 5.);

        assert!(bullet.collides_with(&bullet_transform, &ship, &ship_transform).is_some());
    }
}
