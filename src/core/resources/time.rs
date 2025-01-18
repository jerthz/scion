pub use time::*;
pub use timer::*;

#[derive(Debug)]
pub enum Error {
    TimerAlreadyExists,
    TimerDoesNotExist,
}

mod time {
    use std::time::{Duration, Instant};

    /// ['Time'] is a resource dedicated to compute the time durations between frames and keep a track of
    /// frame numbers
    pub struct Time {
        delta_duration: Duration,
        frame_number: u64,
        measure_start: Instant,
    }

    impl Default for Time {
        fn default() -> Self {
            Self {
                delta_duration: Default::default(),
                frame_number: 0,
                measure_start: Instant::now(),
            }
        }
    }

    impl Time {
        /// finish the last frame and return its duration
        pub(crate) fn frame(&mut self) -> Duration {
            self.frame_number += 1;
            self.delta_duration = self.measure_start.elapsed();
            self.measure_start = Instant::now();
            self.delta_duration
        }

        /// Returns the duration of the last executed frame
        pub fn delta_duration(&self) -> Duration {
            self.delta_duration
        }
    }
}

mod timer {
    use std::{collections::HashMap, time::Duration};

    use crate::core::resources::time::Error;

    /// Different types of timer that car be used
    pub enum TimerType {
        /// Manual timers are meant to be launched manually each time.
        /// Once finished, it will wait the next user trigger to restart.
        Manual,
        /// Cyclic timers will cycle the preset time until the user specifically tells
        /// it to stop
        Cyclic,
    }

    pub struct Timer {
        /// Type of the current timer
        timer_type: TimerType,
        /// Is the timer currently running
        running: bool,
        /// Elapsed time since the start of the current timer. In case of cyclic, current cycle.
        current_duration: f32,
        /// Total duration of this timer before it's ended or per cycle
        total_duration: f32,
        /// A flag to keep track if the current timer just finished its measure.
        dirty: bool,
        /// Total cycles since last cycle fn call
        current_elapsed_cycles: usize,
    }

    impl Timer {
        /// Creates a new timer, only to be used internally.
        pub(crate) fn new(total_duration: f32, timer_type: TimerType) -> Self {
            Self {
                timer_type,
                running: true,
                current_duration: 0.0,
                total_duration,
                dirty: false,
                current_elapsed_cycles: 0,
            }
        }

        /// Adds the duration to the current timer and return whether or not the timer has ended or
        /// done a cycle
        pub fn add_delta_duration(&mut self, delta_duration: f32) -> bool {
            self.dirty = false;
            if !self.running {
                return false;
            }

            match self.timer_type {
                TimerType::Manual => {
                    self.current_duration += delta_duration;
                    if self.current_duration >= self.total_duration {
                        self.running = false;
                        self.dirty = true;
                    }
                }
                TimerType::Cyclic => {
                    let total = self.current_duration + delta_duration;
                    if total > self.total_duration {
                        self.dirty = true;
                    }
                    self.current_elapsed_cycles += (total / self.total_duration) as usize;
                    self.current_duration = total % self.total_duration;
                }
            }

            self.dirty
        }

        // returns the elapsed time of the current timer's run
        pub fn elapsed(&self) -> f32 {
            self.current_duration
        }

        /// returns whether or not the timer has ended
        pub fn ended(&self) -> bool {
            !self.running
        }

        /// reset the timer end start it
        pub fn reset(&mut self) {
            self.running = true;
            self.current_duration = 0.;
            self.dirty = false;
            self.current_elapsed_cycles = 0;
        }

        /// changes the total duration of this timer
        pub fn change_cycle(&mut self, new_cycle: f32) {
            self.total_duration = new_cycle;
        }

        /// Returns the number of cycles elapsed since the last call of this fn
        pub fn cycle(&mut self) -> usize {
            let res = self.current_elapsed_cycles;
            self.current_elapsed_cycles = 0;
            res
        }
    }

    /// Timers is a convenience resource provided by `Scion`
    /// in order to help users to create timers in their systems/layers
    #[derive(Default)]
    pub struct Timers {
        timers: HashMap<String, Timer>,
    }

    impl Timers {
        /// Create and adds a timer to the list of known timers
        pub fn add_timer(
            &mut self,
            name: &str,
            timer_type: TimerType,
            duration_in_second: f32,
        ) -> Result<&mut Timer, Error> {
            if self.timers.contains_key(name) {
                return Err(Error::TimerAlreadyExists);
            }
            self.timers.insert(name.to_string(), Timer::new(duration_in_second, timer_type));
            Ok(self.timers.get_mut(name).expect("Missing the timer we just inserted..."))
        }

        /// Delete a timer from the list of known timers
        pub fn delete_timer(&mut self, name: &str) -> Result<(), Error> {
            if self.timers.contains_key(name) {
                self.timers.remove(name);
                Ok(())
            } else {
                Err(Error::TimerDoesNotExist)
            }
        }

        /// Returns whether or not a timer with the `name` identifier exists
        pub fn exists(&mut self, name: &str) -> bool {
            self.timers.contains_key(name)
        }

        /// Returns the timer identified by the `name` if it exist
        pub fn get_timer(&mut self, name: &str) -> Result<&mut Timer, Error> {
            self.timers.get_mut(name).ok_or(Error::TimerDoesNotExist)
        }

        pub(crate) fn add_delta_duration(&mut self, delta_duration: Duration) {
            let delta = delta_duration.as_secs_f32();
            self.timers.values_mut().for_each(|timer| {
                timer.add_delta_duration(delta);
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::resources::time::{TimerType, Timers};

    #[test]
    fn add_timer_test() {
        let mut timers = Timers::default();
        let timer = timers.add_timer("test_timer", TimerType::Manual, 1.0);
        assert!(timer.is_ok());
        let timer = timers.add_timer("test_timer", TimerType::Manual, 1.0);
        assert!(timer.is_err());

        // Test manual timer
        let timer = timers.get_timer("test_timer");
        assert!(timer.is_ok());
        let timer = timer.expect("impossible");
        assert!(!timer.add_delta_duration(0.5));
        assert!(timer.add_delta_duration(0.5));
        assert!(timer.ended());

        // Test cyclic timer
        let timer = timers.add_timer("test_timer2", TimerType::Cyclic, 1.0);
        assert!(timer.is_ok());
        let timer = timer.unwrap();
        assert!(!timer.add_delta_duration(0.5));
        assert!(timer.add_delta_duration(1.));
        assert_eq!(1, timer.cycle());
        assert_eq!(0.5, timer.elapsed());
        assert!(!timer.ended());
    }
}
