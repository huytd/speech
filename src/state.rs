use crate::data::{DataUtils, RandomExampleResponse};
use rand::Rng;
use std::collections::HashMap;

const PRONUNCIATION_DATA: &str = include_str!("../data/pronunciation.txt");
const EXAMPLE_LIST: &str = include_str!("../data/examples.txt");

#[derive(Clone)]
pub struct ServerState {
    pronounce_dict: HashMap<&'static str, &'static str>,
    example_list: Vec<&'static str>,
}

impl ServerState {
    pub fn new() -> Self {
        let mut state = ServerState {
            pronounce_dict: HashMap::new(),
            example_list: vec![],
        };
        state.load_pronounce_dictionary();
        state.load_example_list();
        state
    }

    fn load_pronounce_dictionary(&mut self) {
        PRONUNCIATION_DATA
            .lines()
            .into_iter()
            .filter(|&line| !line.starts_with(&";;;") && !line.is_empty())
            .for_each(|line| {
                if let Some((word, pronounce)) = line.split_once("  ") {
                    self.pronounce_dict.insert(word, pronounce);
                }
            });
    }

    fn load_example_list(&mut self) {
        self.example_list = EXAMPLE_LIST.lines().collect();
    }

    pub fn lookup_pronounce(&self, word: &str) -> &'static str {
        let word = DataUtils::clean_up_word(word).to_uppercase();
        let result = self.pronounce_dict.get(word.as_str()).unwrap_or(&"");
        *result
    }

    pub fn get_random_example(&self) -> RandomExampleResponse {
        let index = rand::thread_rng().gen_range(0..self.example_list.len());
        let selected_example = self.example_list[index];
        let pronounce = selected_example
            .split_whitespace()
            .map(|word| self.lookup_pronounce(word))
            .collect();
        RandomExampleResponse::new(selected_example, pronounce)
    }
}
