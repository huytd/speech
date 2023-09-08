use std::sync::OnceLock;
use vosk::{Model, Recognizer};

pub static SPEECH_MODEL: OnceLock<Model> = OnceLock::new();

pub struct SpeechEngine;

impl SpeechEngine {
    pub fn initialize_speech_model() {
        let model = Model::new("./model").expect("Could not initialize Vosk model!");
        SPEECH_MODEL.set(model).ok();
    }

    pub fn create_recognizer(sample_rate: f32) -> Option<Recognizer> {
        let model = SPEECH_MODEL.get()?;
        let mut recognizer = Recognizer::new(model, sample_rate)?;
        recognizer.set_max_alternatives(0);
        recognizer.set_words(true);
        Some(recognizer)
    }
}
