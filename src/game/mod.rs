use std::fmt::Display;
use std::time::SystemTime;

use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::export::Formatter;

use crate::game::Color::{Blue, Red};
use crate::game::RevealOutcome::{Nop, Opened};
use crate::print::ColoredDesc;
use crate::res::words::NoSuchLanguageError;
use uuid::Uuid;

pub mod cache;

#[derive(Clone, Debug, PartialEq)]
pub enum Color {
    Red,
    Blue,
}

impl Color {
    pub fn invert(&self) -> Self {
        match self {
            Red => Blue,
            Blue => Red,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.eq(&Color::Red) { "red" } else { "blue" })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Team {
    Player(Color),
    None,
    Death,
}

#[derive(Clone)]
pub struct GameWord {
    pub word: String,
    pub team: Team,
    pub opened: bool,
}

#[derive(Debug, PartialEq)]
pub enum RevealOutcome {
    Nop,
    Opened(String, Team),
}

#[derive(Clone)]
pub struct Game {
    pub name: String,
    pub ident: String,
    pub created: SystemTime,
    pub words: Vec<GameWord>,
    pub turn: Color,
    pub winner: Option<Color>,
}

impl Game {
    pub fn new(name: String, language: &str) -> Result<Self, NoSuchLanguageError> {
        let game = Game {
            name,
            ident: Uuid::new_v4().to_string(),
            created: SystemTime::now(),
            words: words_for_game(language)?,
            turn: Red,
            winner: None,
        };
        println!("{}", game.desc_colored());
        Ok(game)
    }

    pub fn reveal(&mut self, word: &str) -> RevealOutcome {
        if self.winner.is_some() {
            return Nop;
        }
        let mut outcome = Nop;
        let word = self.words.iter_mut().find(|w| w.word.eq(word));
        if let Some(w) = word {
            if !w.opened {
                w.opened = true;
                match &w.team {
                    Team::Player(color) => {
                        if color.ne(&self.turn) {
                            self.turn = self.turn.invert();
                        }
                    }
                    Team::None => self.turn = self.turn.invert(),
                    Team::Death => self.winner = Some(self.turn.invert()),
                }

                outcome = Opened(w.word.clone(), w.team.clone());
            }
        }
        let winner = self.determine_winner();
        self.winner = winner;
        outcome
    }

    fn determine_winner(&self) -> Option<Color> {
        println!("winner is: {:?}", &self.winner);
        if self.winner.is_some() {
            self.winner.clone()
        } else {
            for color in vec![Color::Red, Color::Blue] {
                let team = Team::Player(color.clone());
                let number = number_of_words_for_team(&team);
                let revealed = self.words.iter()
                    .filter(|w| w.opened)
                    .filter(|w| w.team.eq(&team))
                    .count();
                println!("{} opened: {}", &color, &revealed);
                if revealed >= number {
                    return Some(color);
                }
            }
            None
        }
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
    words.shuffle(&mut thread_rng());
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

#[cfg(test)]
mod tests {
    mod color {
        use crate::game::Color;
        use crate::game::Color::*;

        #[test_case(Red => Blue)]
        #[test_case(Blue => Red)]
        fn invert_team_color(actual: Color) -> Color {
            actual.invert()
        }

        #[test_case(Red => "red")]
        #[test_case(Blue => "blue")]
        fn color_to_string(color: Color) -> String {
            color.to_string()
        }
    }

    mod game {
        use crate::game::Color::*;
        use crate::game::{Game, Color, Team};

        #[test]
        fn determine_existing_winner() {
            let mut game = Game::new("test".into(), "german").unwrap();
            game.winner = Some(Red);
            assert_eq!(game.winner, game.determine_winner());
        }

        #[test_case(Color::Red; "red")]
        #[test_case(Color::Blue; "blue")]
        fn determine_winner_by_revealed_cards(color: Color) {
            let mut game = Game::new("test".into(), "german").unwrap();
            game.winner = None;
            open_all_with_color(&mut game, color.clone());
            assert_eq!(Some(color), game.determine_winner());
        }

        pub fn open_all_with_color(game: &mut Game, color: Color) {
            let team = Team::Player(color);
            game.words.iter_mut()
                .filter(|w| w.team.eq(&team))
                .for_each(|w| w.opened = true);
        }
    }
}