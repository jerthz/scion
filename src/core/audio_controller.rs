use std::collections::HashMap;
use std::io::BufReader;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use log::debug;
use rodio::{OutputStream, OutputStreamBuilder, Sink, Source};

use crate::core::resources::audio::AudioEvent;

pub(crate) struct AudioController {
    receiver: Receiver<AudioEvent>,
}

impl AudioController {
    pub(crate) fn new(receiver: Receiver<AudioEvent>) -> Self {
        Self { receiver }
    }
}

pub(crate) fn audio_thread(controller: AudioController) {
    let stream_handle = OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");
    let mut sinks: HashMap<usize, Sink> = HashMap::new();

    loop {
        if let Ok(message) = controller.receiver.recv_timeout(Duration::from_secs(1)) {
            match message {
                AudioEvent::PlaySound { path, config, sound_id } => {
                    debug!("Started to play sound {}", path);
                    let sink =  rodio::Sink::connect_new(&stream_handle.mixer());
                    let file = std::fs::File::open(path.as_str()).unwrap();
                    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                    if config.looped {
                        sink.append(source.repeat_infinite());
                    } else {
                        sink.append(source);
                    }
                    // TODO: handle categories
                    sink.set_volume(config.volume);
                    sinks.insert(sound_id, sink);
                }
                AudioEvent::StopSound { sound_id } => {
                    if let Some(sink) = sinks.remove(&sound_id) {
                        sink.stop();
                        drop(sink);
                    }
                }
            }
        }
        sinks.retain(|&_k, sink| {
            !sink.empty()
        });
    }
}
