use crate::game::{Game, Team, Color};
use colored::Colorize;

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
                let open_status = if word.opened { "+" } else { "-" };
                let word = match word.team {
                    Team::Player(Color::Red) => word.word.red(),
                    Team::Player(Color::Blue) => word.word.blue(),
                    Team::None => word.word.white(),
                    Team::Death => word.word.magenta(),
                };
                line = format!("{} {}{:^10}", line, open_status, word);
            }
            field = format!("{}\n{}", field, line);
        }
        let is_won = self.winner.is_some();
        format!(
            "{}\nGame: {}\n{}: {}\n{}\n{}",
            GAME_DESC_DELIMITER,
            &self.name,
            if is_won { "winner" } else { "turn" },
            {
                let color = if is_won { self.winner.clone().unwrap() } else { self.turn.clone() };
                color.desc_colored()
            },
            field,
            GAME_DESC_DELIMITER
        )
    }
}