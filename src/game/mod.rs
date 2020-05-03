use crate::game::Color::Red;
use crate::res::words::NoSuchLanguageError;
use std::time::SystemTime;

pub mod cache;

#[derive(Clone)]
pub enum Color {
    Red,
    Blue,
}

#[derive(Clone)]
pub enum Team {
    Player(Color),
    None,
    Death,
}

pub struct GameWord {
    pub word: String,
    pub team: Team,
    pub opened: bool,
}

pub struct Game {
    pub name: String,
    pub created: SystemTime,
    pub words: Vec<GameWord>,
    pub turn: Color,
}

impl Game {
    pub fn new(name: String, language: &str) -> Result<Self, NoSuchLanguageError> {
        Ok(Game {
            name,
            created: SystemTime::now(),
            words: words_for_game(language)?,
            turn: Red,
        })
    }
}

fn words_for_game(language: &str) -> Result<Vec<GameWord>, NoSuchLanguageError> {
    use Color::*;
    use Team::*;

    let raw_words = crate::res::words::get_25_random(language)?;
    let indices: Vec<usize> = (0..25).collect();
    let mut words: Vec<GameWord> = Vec::new();
    let mut team_count = 0;
    let mut team = Player(Red);
    for i in indices {
        words.push(GameWord {
            word: raw_words.get(i).unwrap().clone(),
            team: team.clone(),
            opened: false,
        });
        team_count += 1;
        if team_count == number_of_words_for_team(&team) {
            team_count = 0;
            team = match team {
                Player(Red) => Player(Blue),
                Player(Blue) => Death,
                Death => None,
                None => None,
            }
        }
    }
    Ok(words)
}

fn number_of_words_for_team(team: &Team) -> usize {
    use Color::*;
    use Team::*;

    match team {
        Player(Red) => 9,
        Player(Blue) => 8,
        Death => 1,
        None => 7,
    }
}