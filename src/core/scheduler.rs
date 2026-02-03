use crate::core::state::GameState;
use crate::core::world::GameData;
use profiling_macros::profile;
use std::collections::LinkedList;

#[derive(Default)]
pub(crate) struct Scheduler {
    systems: LinkedList<(Option<fn(&GameState) -> bool>, fn(&mut GameData))>,
}

impl Scheduler {
    pub(crate) fn add_system(&mut self, system: fn(&mut GameData)) {
        self.systems.push_back((None, system));
    }

    pub(crate) fn add_pausable_system(&mut self,
                                      system: fn(&mut GameData),
                                      pause_condition: fn(&GameState) -> bool) {
        self.systems.push_back((Some(pause_condition), system));
    }
    #[profile("scheduler::execute")]
    pub(crate) fn execute(&mut self, data: &mut GameData) {
        let systems_to_execute : LinkedList<&(Option<fn(&GameState) -> bool>, fn(&mut GameData))> = {
            let game_state = data.get_resource::<GameState>().expect("Missing game state resource");
            self.systems.iter().filter(|s| s.0.is_none() || !s.0.unwrap()(&game_state)).collect()
        };
        systems_to_execute.iter().for_each(|s| s.1(data));
    }
}
