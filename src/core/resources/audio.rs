use std::sync::mpsc;

use crate::core::audio_controller;
use crate::core::audio_controller::AudioController;

/// `AudioPlayer` is the resource responsible to handle musics, sound effects, and action on them
pub struct Audio {
    event_sender: mpsc::Sender<AudioEvent>,
    sounds_cursor: usize,
}

impl Audio {
    pub(crate) fn default() -> Self {
        let (event_sender, receiver) = mpsc::channel();

        std::thread::spawn(move || audio_controller::audio_thread(AudioController::new(receiver)));
        Audio { event_sender, sounds_cursor: 0 }
    }

    /// Start to play the sound identified with `name`
    pub fn play(&mut self, path: String, config: PlayConfig) -> Result<usize, Error> {
        let sound_id = self.sounds_cursor;
        if let Ok(()) = self.event_sender.send(AudioEvent::PlaySound { path, config, sound_id }) {
            self.sounds_cursor += 1;
            return Ok(sound_id);
        }
        Err(Error::ImpossibleToLoadSound)
    }
}

/// Error that can be thrown by the AudioPlayer
#[derive(Debug)]
pub enum Error {
    SoundNotRegistered,
    SoundAlreadyExists,
    ImpossibleToLoadSound,
}

/// `PlayConfig` describe how sound must be played
pub struct PlayConfig {
    /// Volume of the sound (should be between 0 and 1)
    pub volume: f32,
    /// Should this sound be looped
    pub looped: bool,
    /// Category of the sound. Usefull when you want to be able to change volume of a given category of sounds
    pub category: Option<String>,
}

impl Default for PlayConfig {
    fn default() -> Self {
        Self { volume: 0.2, looped: false, category: None }
    }
}

/// `AudioEvent` represents events send from the audio controller to the Audio Thread
#[allow(dead_code)]
pub(crate) enum AudioEvent {
    PlaySound { path: String, config: PlayConfig, sound_id: usize },
    StopSound { sound_id: usize },
}
