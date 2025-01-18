use hecs::Component;

use crate::core::world::{GameData, World};
use crate::core::resources::asset_manager::{AssetManager, AssetRef};
use crate::graphics::components::material::Material;

pub(crate) trait AssetResolverFn<T: Component> {
    fn resolve(manager: &AssetManager, asset_ref: &AssetRef<T>) -> T;
}

/// System responsible to add an asset of type T to each entity with an assetRef<T>
pub(crate) fn asset_ref_resolver_system<T: Component, F: AssetResolverFn<T>>(data: &mut GameData) {
    let mut to_add = Vec::new();
    {
        let asset_manager = data.assets();
        for (e, asset_ref) in data.query::<&AssetRef<T>>().without::<&T>().iter() {
            to_add.push((e, (F::resolve(&asset_manager, asset_ref))));
        }
    }
    to_add.drain(0..).for_each(|(e, a)| {
        let _r = data.add_components(e, (a,));
    });
}

pub(crate) struct MaterialAssetResolverFn;
impl AssetResolverFn<Material> for MaterialAssetResolverFn {
    fn resolve(manager: &AssetManager, asset_ref: &AssetRef<Material>) -> Material {
        manager.get_material_for_ref(asset_ref)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::world::World;
    use crate::core::{
        resources::asset_manager::AssetManager,
        systems::asset_ref_resolver_system::MaterialAssetResolverFn,
    };
    use crate::graphics::components::color::Color;

    use super::*;

    #[test]
    fn asset_ref_resolver_material_system_test() {
        let mut world = GameData::default();

        let mut manager = AssetManager::default();
        let asset_ref = manager.register_material(Material::Diffuse(Color::new(1, 1, 1, 1.)));
        world.insert_resource(manager);

        let e = world.push((1, asset_ref.clone()));

        assert!(world.entry::<&Material>(e).expect("").get().is_none());

        asset_ref_resolver_system::<Material, MaterialAssetResolverFn>(&mut world);

        assert!(world.entry::<&Material>(e).expect("").get().is_some());
    }
}
