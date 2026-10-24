use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Source};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::sync::Arc;

pub struct AudioEngine {
    sink_handle: Option<MixerDeviceSink>,
    sound_cache: HashMap<String, Arc<Vec<u8>>>,
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEngine {
    pub fn new() -> Self {
        let sink_handle = DeviceSinkBuilder::open_default_sink().ok();
        Self {
            sink_handle,
            sound_cache: HashMap::new(),
        }
    }

    pub fn play_sound(&mut self, path: &str) -> Result<(), String> {
        let data = self.load(path)?;
        let handle = match &self.sink_handle {
            Some(h) => h,
            None => return Err("Audio stream is not available".to_string()),
        };

        let cursor = Cursor::new(data.as_ref().clone());
        let source = Decoder::new(cursor)
            .map_err(|e| format!("Failed to decode audio '{}': {}", path, e))?;

        handle.mixer().add(source);
        Ok(())
    }

    pub fn play_music(&mut self, path: &str, volume: f32) -> Result<(), String> {
        let data = self.load(path)?;
        let handle = self
            .sink_handle
            .as_ref()
            .ok_or_else(|| "Audio stream is not available".to_string())?;
        let cursor = Cursor::new(data.as_ref().clone());
        let source = Decoder::new(cursor)
            .map_err(|error| format!("Failed to decode audio '{}': {}", path, error))?
            .repeat_infinite()
            .amplify(volume.clamp(0.0, 1.0));
        handle.mixer().add(source);
        Ok(())
    }

    fn load(&mut self, path: &str) -> Result<Arc<Vec<u8>>, String> {
        if let Some(cached) = self.sound_cache.get(path) {
            return Ok(cached.clone());
        }
        let mut file = File::open(path)
            .map_err(|error| format!("Failed to open file '{}': {}", path, error))?;
        let mut buffer = Vec::new();
        use std::io::Read;
        file.read_to_end(&mut buffer)
            .map_err(|error| format!("Failed to read file: {}", error))?;
        let data = Arc::new(buffer);
        self.sound_cache.insert(path.to_string(), data.clone());
        Ok(data)
    }
}

thread_local! {
    static AUDIO_ENGINE: RefCell<AudioEngine> = RefCell::new(AudioEngine::new());
}

pub fn play_sound(path: &str) -> Result<(), String> {
    AUDIO_ENGINE.with(|engine| engine.borrow_mut().play_sound(path))
}

pub fn play_music(path: &str, volume: f32) -> Result<(), String> {
    AUDIO_ENGINE.with(|engine| engine.borrow_mut().play_music(path, volume))
}
