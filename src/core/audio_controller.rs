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
    let mut sinks: HashMap<usize, (Sink, bool)> = HashMap::new();

    loop {
        if let Ok(message) = controller.receiver.recv_timeout(Duration::from_millis(100)) {
            match message {
                AudioEvent::PlaySound { path, config, sound_id } => {
                    debug!("Started to play sound {}", path);
                    let sink =  rodio::Sink::connect_new(&stream_handle.mixer());
                    if config.looped {
                        let file = std::fs::File::open(path.as_str()).unwrap();
                        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                        let buf_source = rodio::buffer::SamplesBuffer::new(
                            source.channels(),
                            source.sample_rate(),
                            source.collect::<Vec<_>>()
                        );
                        sink.append(buf_source.repeat_infinite());
                    } else {
                        let file = std::fs::File::open(path.as_str()).unwrap();
                        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                        sink.append(source);
                    }
                    // TODO: handle categories
                    sink.set_volume(config.volume);
                    sink.play();
                    sinks.insert(sound_id, (sink, config.looped));
                }
                AudioEvent::StopSound { sound_id } => {
                    if let Some((sink, _)) = sinks.remove(&sound_id) {
                        sink.stop();
                        drop(sink);
                    }
                }
            }
            // Only clean up finished sinks when we receive a message
            sinks.retain(|&_k, (sink, looped)| if *looped { true } else { !sink.empty() });
        }
    }
}
