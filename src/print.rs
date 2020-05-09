use colored::Colorize;

use crate::game::{Color, Game, GameWord, Team};

pub trait ColoredDesc {
    fn desc_colored(&self) -> String;
}

const GAME_DESC_DELIMITER: &'static str = "==================";

impl ColoredDesc for Color {
    fn desc_colored(&self) -> String {
        match self {
            Color::Red => "red".red(),
            Color::Blue => "blue".blue(),
        }.to_string()
    }
}

impl ColoredDesc for Game {
    fn desc_colored(&self) -> String {
        let mut field = String::new();
        for y in 0..5 {
            let mut line = String::new();
            for x in 0..5 {
                let word = self.words.get(x + y * 5).unwrap();
                println!("Word: {}, desc: '{}'", word.word, word.desc_colored());
                line = format!("{} {}", line, word.desc_colored());
            }
            field = format!("{}\n{}", field, line);
        }
        let is_won = self.winner.is_some();
        let turn = if is_won { "winner" } else { "turn" };
        let turn_team = if is_won { self.winner.clone().unwrap() } else { self.turn.clone() };
        format!(
            "{}\nGame: {}\nIdent: {}\n{}: {}\n{}\n{}",
            GAME_DESC_DELIMITER,
            &self.name,
            &self.ident,
            turn,
            turn_team.desc_colored(),
            field,
            GAME_DESC_DELIMITER
        )
    }
}

impl ColoredDesc for GameWord {
    fn desc_colored(&self) -> String {
        let visibility = if self.opened { '+' } else { '-' };
        let colored = color_for_team(&format!("{:^10}", self.word), &self.team);
        format!("{}{}", visibility, colored)
    }
}

fn color_for_team(text: &str, team: &Team) -> String {
    use Color::*;
    use Team::*;

    match team {
        Player(Red) => text.red(),
        Player(Blue) => text.blue(),
        None => text.white(),
        Death => text.magenta(),
    }.to_string()
}