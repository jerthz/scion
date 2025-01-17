use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::Hasher;

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use downcast_rs::{impl_downcast, Downcast};
use hecs::{
    Component, ComponentError, DynamicBundle, Entity, NoSuchEntity, Query, QueryBorrow,
    QueryMut, QueryOne, QueryOneError,
};

use crate::core::components::maths::camera::{Camera, DefaultCamera};
use crate::core::resources::asset_manager::AssetManager;
use crate::core::resources::audio::Audio;
use crate::core::resources::events::Events;
use crate::core::resources::focus_manager::FocusManager;
use crate::core::resources::font_atlas::FontAtlas;
use crate::core::resources::inputs::inputs_controller::InputsController;
use crate::core::resources::time::Timers;
use crate::core::resources::window::Window;
use crate::core::scene::SceneController;
use crate::core::state::GameState;

pub trait World {
    fn entities(&self) -> HashSet<Entity>;
    fn clear(&mut self);
    fn push(&mut self, components: impl DynamicBundle) -> Entity;
    fn remove(&mut self, entity: Entity) -> Result<(), NoSuchEntity>;
    fn add_components(
        &mut self,
        entity: Entity,
        components: impl DynamicBundle,
    ) -> Result<(), NoSuchEntity>;
    fn remove_component<T: Component>(&mut self, entity: Entity) -> Result<T, ComponentError>;
    fn query<Q: Query>(&self) -> QueryBorrow<'_, Q>;
    fn query_mut<Q: Query>(&mut self) -> QueryMut<'_, Q>;
    fn entry<Q: Query>(&self, entity: Entity) -> Result<QueryOne<'_, Q>, NoSuchEntity>;
    fn entry_mut<Q: Query>(&mut self, entity: Entity) -> Result<Q::Item<'_>, QueryOneError>;
    fn contains(&self, entity: Entity) -> bool;
    fn add_default_camera(&mut self) -> Entity;
}

#[derive(Default)]
pub struct GameData {
    pub(crate) subworld: SubWorld,
    pub(crate) resources: Resources,
}

impl GameData {
    pub fn split(&mut self) -> (&mut SubWorld, &mut Resources) {
        (&mut self.subworld, &mut self.resources)
    }

    pub fn contains_resource<T: Resource>(&self) -> bool {
        self.resources.internal_resources.storage.contains_key(&ResourceTypeId::of::<T>())
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resources
            .internal_resources
            .storage
            .insert(ResourceTypeId::of::<T>(), AtomicResourceCell::new(Box::new(resource)));
    }

    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        let resource = self
            .resources
            .internal_resources
            .remove_internal(&ResourceTypeId::of::<T>())?
            .downcast::<T>()
            .ok()?;
        Some(*resource)
    }

    pub fn get_resource<T: Resource>(&self) -> Option<AtomicRef<T>> {
        let type_id = &ResourceTypeId::of::<T>();
        self.resources.internal_resources.storage.get(type_id).map(|x| x.get::<T>())
    }

    pub fn get_resource_mut<T: Resource>(&self) -> Option<AtomicRefMut<T>> {
        let type_id = &ResourceTypeId::of::<T>();
        self.resources.internal_resources.storage.get(type_id).map(|x| x.get_mut::<T>())
    }

    /// retrieves the game_state from the resources.
    pub fn game_state(&self) -> AtomicRef<GameState> {
        self.get_resource::<GameState>()
            .expect("The engine is missing the mandatory game state resource")
    }

    /// retrieves the game_state mutable from the resources.
    pub fn game_state_mut(&self) -> AtomicRefMut<GameState> {
        self.get_resource_mut::<GameState>()
            .expect("The engine is missing the mandatory game state resource")
    }

    /// retrieves the asset manager from the resources.
    pub fn assets(&self) -> AtomicRef<AssetManager> {
        self.get_resource::<AssetManager>()
            .expect("The engine is missing the mandatory asset manager resource")
    }

    /// retrieves the asset manager from the resources.
    pub fn assets_mut(&self) -> AtomicRefMut<AssetManager> {
        self.get_resource_mut::<AssetManager>()
            .expect("The engine is missing the mandatory asset manager resource")
    }

    /// retrieves the timers resource from the resources.
    pub fn timers(&self) -> AtomicRefMut<Timers> {
        self.get_resource_mut::<Timers>()
            .expect("The engine is missing the mandatory timers resource")
    }

    /// retrieves the inputs resource from the resources
    pub fn inputs(&self) -> AtomicRefMut<InputsController> {
        self.get_resource_mut::<InputsController>()
            .expect("The engine is missing the mandatory inputs controller resource")
    }

    /// retrieves the events resource from the resources
    pub fn events(&self) -> AtomicRefMut<Events> {
        self.get_resource_mut::<Events>()
            .expect("The engine is missing the mandatory events resource")
    }

    /// retrieves the audio player from the resources
    pub fn audio(&self) -> AtomicRefMut<Audio> {
        self.get_resource_mut::<Audio>()
            .expect("The engine is missing the mandatory audio player resource")
    }

    /// retrieves the window from the resources
    pub fn window(&self) -> AtomicRefMut<Window> {
        self.get_resource_mut::<Window>()
            .expect("The engine is missing the mandatory window resource")
    }

    /// retrieves the window from the resources
    pub fn scene_controller(&self) -> AtomicRefMut<SceneController> {
        self.get_resource_mut::<SceneController>()
            .expect("The engine is missing the mandatory scene controller resource")
    }

    /// retrieves the font_atlas from the resources.
    pub(crate) fn font_atlas(&self) -> AtomicRefMut<FontAtlas> {
        self.get_resource_mut::<FontAtlas>()
            .expect("The engine is missing the mandatory font_atlas resource")
    }

    /// retrieves the focus manager from the resources.
    #[allow(dead_code)]
    pub(crate) fn focus_manager(&self) -> AtomicRefMut<FocusManager> {
        self.get_resource_mut::<FocusManager>()
            .expect("The engine is missing the mandatory focus manager resource")
    }

    pub(crate) fn has_camera(&self)-> bool{
        self.subworld.query::<&Camera>().iter().count() > 0
    }

    pub(crate) fn take_despawned(&mut self) -> Option<Vec<Entity>>{
        self.subworld.entity_cleaner.take()
    }
}

impl World for GameData {
    fn entities(&self) -> HashSet<Entity> {
        self.subworld
            .internal_world
            .iter()
            .map(|entity_ref| entity_ref.entity())
            .collect::<HashSet<_>>()
    }

    fn clear(&mut self) {
        self.subworld.internal_world.clear();
    }

    fn push(&mut self, components: impl DynamicBundle) -> Entity {
        self.subworld.internal_world.spawn(components)
    }

    fn remove(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.subworld.remove(entity)
    }

    fn add_components(
        &mut self,
        entity: Entity,
        components: impl DynamicBundle,
    ) -> Result<(), NoSuchEntity> {
        self.subworld.internal_world.insert(entity, components)
    }

    fn remove_component<T: Component>(&mut self, entity: Entity) -> Result<T, ComponentError> {
        self.subworld.internal_world.remove_one::<T>(entity)
    }

    fn query<Q: Query>(&self) -> QueryBorrow<'_, Q> {
        self.subworld.internal_world.query::<Q>()
    }

    fn query_mut<Q: Query>(&mut self) -> QueryMut<'_, Q> {
        self.subworld.internal_world.query_mut::<Q>()
    }

    fn entry<Q: Query>(&self, entity: Entity) -> Result<QueryOne<'_, Q>, NoSuchEntity> {
        self.subworld.internal_world.query_one::<Q>(entity)
    }

    fn entry_mut<Q: Query>(&mut self, entity: Entity) -> Result<Q::Item<'_>, QueryOneError> {
        self.subworld.internal_world.query_one_mut::<Q>(entity)
    }

    fn contains(&self, entity: Entity) -> bool {
        self.subworld.internal_world.contains(entity)
    }

    fn add_default_camera(&mut self) -> Entity {
        self.push((DefaultCamera,))
    }
}

#[derive(Default)]
pub struct SubWorld {
    internal_world: hecs::World,
    entity_cleaner: Option<Vec<Entity>>,
}

#[derive(Default)]
pub struct Resources {
    internal_resources: InternalResources,
}

impl World for SubWorld {
    fn entities(&self) -> HashSet<Entity> {
        self.internal_world.iter().map(|entity_ref| entity_ref.entity()).collect::<HashSet<_>>()
    }

    fn clear(&mut self) {
        self.internal_world.clear();
    }

    fn push(&mut self, components: impl DynamicBundle) -> Entity {
        self.internal_world.spawn(components)
    }

    fn remove(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        match &mut self.entity_cleaner {
            Some(vec) => vec.push(entity),
            None => {
                self.entity_cleaner = Some(vec![entity]);
            }
        }
        self.internal_world.despawn(entity)
    }

    fn add_components(
        &mut self,
        entity: Entity,
        components: impl DynamicBundle,
    ) -> Result<(), NoSuchEntity> {
        self.internal_world.insert(entity, components)
    }

    fn remove_component<T: Component>(&mut self, entity: Entity) -> Result<T, ComponentError> {
        self.internal_world.remove_one::<T>(entity)
    }

    fn query<Q: Query>(&self) -> QueryBorrow<'_, Q> {
        self.internal_world.query::<Q>()
    }

    fn query_mut<Q: Query>(&mut self) -> QueryMut<'_, Q> {
        self.internal_world.query_mut::<Q>()
    }

    fn entry<Q: Query>(&self, entity: Entity) -> Result<QueryOne<'_, Q>, NoSuchEntity> {
        self.internal_world.query_one::<Q>(entity)
    }

    fn entry_mut<Q: Query>(&mut self, entity: Entity) -> Result<Q::Item<'_>, QueryOneError> {
        self.internal_world.query_one_mut::<Q>(entity)
    }

    fn contains(&self, entity: Entity) -> bool {
        self.internal_world.contains(entity)
    }

    fn add_default_camera(&mut self) -> Entity {
        self.push((DefaultCamera,))
    }
}

impl Resources {
    pub fn contains_resource<T: Resource>(&self) -> bool {
        self.internal_resources.storage.contains_key(&ResourceTypeId::of::<T>())
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.internal_resources
            .storage
            .insert(ResourceTypeId::of::<T>(), AtomicResourceCell::new(Box::new(resource)));
    }

    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        let resource = self
            .internal_resources
            .remove_internal(&ResourceTypeId::of::<T>())?
            .downcast::<T>()
            .ok()?;
        Some(*resource)
    }

    pub fn get_resource<T: Resource>(&self) -> Option<AtomicRef<T>> {
        let type_id = &ResourceTypeId::of::<T>();
        self.internal_resources.storage.get(type_id).map(|x| x.get::<T>())
    }

    pub fn get_resource_mut<T: Resource>(&self) -> Option<AtomicRefMut<T>> {
        let type_id = &ResourceTypeId::of::<T>();
        self.internal_resources.storage.get(type_id).map(|x| x.get_mut::<T>())
    }

    /// retrieves the asset manager from the resources.
    pub fn game_state(&self) -> AtomicRef<GameState> {
        self.get_resource::<GameState>()
            .expect("The engine is missing the mandatory game state resource")
    }

    /// retrieves the asset manager from the resources.
    pub fn game_state_mut(&self) -> AtomicRefMut<GameState> {
        self.get_resource_mut::<GameState>()
            .expect("The engine is missing the mandatory game state resource")
    }

    /// retrieves the asset manager from the resources.
    pub fn assets(&self) -> AtomicRef<AssetManager> {
        self.get_resource::<AssetManager>()
            .expect("The engine is missing the mandatory asset manager resource")
    }

    /// retrieves the asset manager from the resources.
    pub fn assets_mut(&self) -> AtomicRefMut<AssetManager> {
        self.get_resource_mut::<AssetManager>()
            .expect("The engine is missing the mandatory asset manager resource")
    }

    /// retrieves the timers resource from the resources.
    pub fn timers(&self) -> AtomicRefMut<Timers> {
        self.get_resource_mut::<Timers>()
            .expect("The engine is missing the mandatory timers resource")
    }

    /// retrieves the inputs resource from the resources
    pub fn inputs(&self) -> AtomicRefMut<InputsController> {
        self.get_resource_mut::<InputsController>()
            .expect("The engine is missing the mandatory inputs controller resource")
    }

    /// retrieves the events resource from the resources
    pub fn events(&self) -> AtomicRefMut<Events> {
        self.get_resource_mut::<Events>()
            .expect("The engine is missing the mandatory events resource")
    }

    /// retrieves the audio player from the resources
    pub fn audio(&self) -> AtomicRefMut<Audio> {
        self.get_resource_mut::<Audio>()
            .expect("The engine is missing the mandatory audio player resource")
    }

    /// retrieves the window from the resources
    pub fn window(&self) -> AtomicRefMut<Window> {
        self.get_resource_mut::<Window>()
            .expect("The engine is missing the mandatory window resource")
    }

    /// retrieves the window from the resources
    pub fn scene_controller(&self) -> AtomicRefMut<SceneController> {
        self.get_resource_mut::<SceneController>()
            .expect("The engine is missing the mandatory scene controller resource")
    }

    /// retrieves the font_atlas from the resources.
    pub(crate) fn font_atlas(&self) -> AtomicRefMut<FontAtlas> {
        self.get_resource_mut::<FontAtlas>()
            .expect("The engine is missing the mandatory font_atlas resource")
    }

    /// retrieves the focus manager from the resources.
    pub(crate) fn focus_manager(&self) -> AtomicRefMut<FocusManager> {
        self.get_resource_mut::<FocusManager>()
            .expect("The engine is missing the mandatory focus manager resource")
    }
}

#[derive(Default)]
pub struct InternalResources {
    storage: HashMap<ResourceTypeId, AtomicResourceCell>
}

unsafe impl Send for InternalResources {}

unsafe impl Sync for InternalResources {}

impl InternalResources {
    fn remove_internal(&mut self, type_id: &ResourceTypeId) -> Option<Box<dyn Resource>> {
        self.storage.remove(type_id).map(|cell| cell.into_inner())
    }
}

pub trait Resource: 'static + Downcast {}

impl<T> Resource for T where T: 'static {}
impl_downcast!(Resource);

#[derive(Copy, Clone, Debug, Eq, PartialOrd, Ord)]
pub struct ResourceTypeId {
    type_id: TypeId,
    name: &'static str,
}

impl ResourceTypeId {
    /// Returns the resource type ID of the given resource type.
    pub fn of<T: Resource>() -> Self {
        Self { type_id: TypeId::of::<T>(), name: std::any::type_name::<T>() }
    }
}

impl std::hash::Hash for ResourceTypeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}

impl PartialEq for ResourceTypeId {
    fn eq(&self, other: &Self) -> bool {
        self.type_id.eq(&other.type_id)
    }
}

impl Display for ResourceTypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct AtomicResourceCell {
    data: AtomicRefCell<Box<dyn Resource>>,
}

impl AtomicResourceCell {
    fn new(resource: Box<dyn Resource>) -> Self {
        Self { data: AtomicRefCell::new(resource) }
    }

    fn into_inner(self) -> Box<dyn Resource> {
        self.data.into_inner()
    }

    pub fn get<T: Resource>(&self) -> AtomicRef<T> {
        let borrow = self.data.borrow(); // panics if this is borrowed already
        AtomicRef::map(borrow, |inner| inner.downcast_ref::<T>().unwrap())
    }

    pub fn get_mut<T: Resource>(&self) -> AtomicRefMut<T> {
        let borrow = self.data.borrow_mut(); // panics if this is borrowed already
        AtomicRefMut::map(borrow, |inner| inner.downcast_mut::<T>().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_read_write_test() {
        struct TestOne {
            value: String,
        }

        struct TestTwo {
            value: String,
        }

        let mut world = GameData::default();
        world.insert_resource(TestOne { value: "one".to_string() });
        world.insert_resource(TestTwo { value: "two".to_string() });

        world.push((1, TestOne { value: "".to_string() }));

        let (subworld, resources) = world.split();

        let res1 = resources.get_resource_mut::<TestOne>().unwrap();
        let res2 = resources.get_resource_mut::<TestTwo>().unwrap();

        for (_e, _i) in subworld.query_mut::<&mut TestOne>() {
            println!("hello");
            println!("{:?}", res1.value);
            println!("{:?}", res2.value);
        }

        assert_eq!(res1.value, "one");
        assert_eq!(res2.value, "two");
    }
}
