use serde::Serialize;
use vosk::CompleteResultSingle;

pub struct DataUtils;

impl DataUtils {
    pub fn clean_up_word(word: &str) -> String {
        word.chars()
            .filter(|c| c.is_alphanumeric() || *c == '\'')
            .collect()
    }
}

#[derive(Serialize)]
pub struct RandomExampleResponse {
    text: &'static str,
    pronounce: Vec<&'static str>,
}

impl RandomExampleResponse {
    pub fn new(text: &'static str, pronounce: Vec<&'static str>) -> Self {
        Self { text, pronounce }
    }
}

#[derive(Serialize)]
pub struct Word {
    pub text: String,
    pub score: f32,
    pub start: f32,
    pub end: f32,
}

#[derive(Serialize)]
pub struct SpeechAnalyzeResult {
    pub text: String,
    pub words: Vec<Word>,
}

impl SpeechAnalyzeResult {
    pub fn from(single: CompleteResultSingle) -> Self {
        let text = single.text.to_owned();
        let words = single
            .result
            .iter()
            .map(|w| Word {
                text: w.word.to_owned(),
                score: w.conf,
                start: w.start,
                end: w.end,
            })
            .collect();
        Self { text, words }
    }
}
