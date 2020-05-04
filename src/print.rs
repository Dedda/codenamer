use crate::game::{Game, Team, Color};
use colored::Colorize;

pub trait ColoredDesc {
    fn desc_colored(&self) -> String;
}

const GAME_DESC_DELIMITER: &'static str = "==================";

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
        format!(
            "{}\nGame: {}\n{}\n{}",
            GAME_DESC_DELIMITER,
            if self.turn.eq(&Color::Red) { self.name.red() } else { self.name.blue() },
            field,
            GAME_DESC_DELIMITER
        )
    }
}