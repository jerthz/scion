use crate::config::scion_config::ScionConfig;
use crate::core::package::Package;
use crate::core::scene::{Scene, SceneMachine};
use crate::core::scheduler::Scheduler;
use crate::core::state::GameState;
use crate::core::systems::InternalPackage;
use crate::core::world::GameData;
use crate::Scion;

/// Builder providing convenience functions to build the `Scion` application.
/// This builder is returned when calling [`Scion::app()`] of [`Scion::app_with_config()`]
/// and can't be obtained otherwise.
pub struct ScionBuilder {
    config: ScionConfig,
    scheduler: Scheduler,
    scene: Option<Box<dyn Scene + Send>>,
    world: GameData,
}

impl ScionBuilder {
    pub fn new(config: ScionConfig) -> Self {
        let builder = Self {
            config,
            scheduler: Default::default(),
            scene: Default::default(),
            world: Default::default(),
        };
        builder.with_package(InternalPackage)
    }

    /// Specify a system to add to the scheduler.
    pub fn with_system(mut self, system: fn(&mut GameData)) -> Self {
        self.scheduler.add_system(system);
        self
    }

    /// Specify a system to add to the scheduler with a conditional pausing flag function.
    pub fn with_pausable_system(mut self, system: fn(&mut GameData), pause_condition: fn(&GameState) -> bool) -> Self {
        self.scheduler.add_pausable_system(system, pause_condition);
        self
    }

    /// Set the scene to the given one. Only one scene can be executed at a time
    pub fn with_scene<T: Scene + Default + Send + 'static>(mut self) -> Self {
        self.scene = Some(Box::<T>::default());
        self
    }

    /// Add the given package to the build
    pub fn with_package<P: Package>(mut self, package: P) -> Self {
        package.prepare(&mut self.world);
        package.load(self)
    }

    /// Builds, setups and runs the Scion application, must be called at the end of the building process.
    pub fn run(self) {
        let scion = Scion {
            config: self.config,
            game_data: Some(self.world),
            scheduler: Some(self.scheduler),
            layer_machine: Some(SceneMachine { current_scene: self.scene, current_scene_started: false }),
            window_event_sender: None,
        };
        scion.run();
    }
}