use std::collections::HashMap;

use crate::random::GetRandom;

lazy_static! {
    static ref WORDS: HashMap<String, Vec<String>> = {
        let data = include_bytes!("words.json");
        let data = std::str::from_utf8(data).unwrap();
        let map: HashMap<String, Vec<String>> = serde_json::from_str(data).unwrap();
        for (language, list) in &map {
            if list.len() < 25 {
                panic!("Language {} contains less than 25 words!", language);
            }
        }
        map
    };
}

#[derive(Debug)]
pub struct NoSuchLanguageError(String);

pub fn languages() -> Vec<String> {
    let words: &HashMap<String, Vec<String>> = &WORDS;
    words.keys().cloned().collect()
}

pub fn words(language: &str) -> Result<Vec<String>, NoSuchLanguageError> {
    let words: &HashMap<String, Vec<String>> = &WORDS;
    if let Some(found) = words.get(language) {
        Ok(found.clone())
    } else {
        Err(NoSuchLanguageError(language.to_string()))
    }
}

pub fn get_25_random(language: &str) -> Result<Vec<String>, NoSuchLanguageError> {
    let words: &HashMap<String, Vec<String>> = &WORDS;
    if let Some(found) = words.get(language) {
        Ok(found.get_n_random(25))
    } else {
        Err(NoSuchLanguageError(language.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::get_25_random;

    #[test_case("german" => 25)]
    #[test_case("english" => 25)]
    fn get_25_gets_25(language: &str) -> usize {
        get_25_random(language).unwrap().len()
    }
}