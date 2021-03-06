use std::sync::{Arc, Mutex};

use askama::Template;
use rocket_contrib::serve::StaticFiles;

use crate::game::{Game, GameWord};
use crate::game_cache;

pub mod language;
pub mod socket;

pub fn start() {
    let rocket = rocket::Rocket::ignite();
    let rocket = rocket.mount("/css", StaticFiles::from("static/css"));
    let rocket = rocket.mount("/js", StaticFiles::from("static/js"));
    let rocket = rocket.mount("/", routes![favicon, index, game]);
    rocket.launch();
}

#[get("/favicon.ico")]
fn favicon() {}

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    num_running_games: usize,
}

#[get("/")]
fn index() -> Index {
    Index {
        num_running_games: game_cache().lock().unwrap().count(),
    }
}

#[derive(Serialize)]
struct Card {
    word: String
}

impl From<&GameWord> for Card {
    fn from(w: &GameWord) -> Self {
        Self {
            word: w.word.clone(),
        }
    }
}

#[derive(Template)]
#[template(path = "game.html")]
struct GamePage {
    game_name: String,
    game_ident: String,
    socket_url: String,
    cards: Vec<Card>,
}

impl From<Arc<Mutex<Game>>> for GamePage {
    fn from(game: Arc<Mutex<Game>>) -> Self {
        let guard = game.lock().unwrap();
        Self {
            game_name: guard.name.clone(),
            game_ident: guard.ident.clone(),
            socket_url: socket::socket_url(),
            cards: guard.words.iter().map(|w| w.into()).collect(),
        }
    }
}

#[get("/g/<game_name>")]
fn game(game_name: String) -> GamePage {
    let gc = game_cache();
    let mut cache = gc.lock().unwrap();
    let g = if let Some(g) = cache.by_name(&game_name) {
        g
    } else {
        let game = Game::new(game_name.clone(), "english").unwrap();
        cache.put(game).unwrap();
        cache.by_name(&game_name).unwrap()
    };
    g.into()
}

#[cfg(test)]
mod tests {
    mod card {
        use crate::game::{GameWord, Team};
        use crate::web::Card;

        #[test]
        fn from_game_word() {
            let word = GameWord {
                word: "horse".into(),
                team: Team::None,
                opened: false,
            };
            let card = Card::from(&word);
            assert_eq!(word.word, card.word);
        }
    }

    mod game_page {
        use std::sync::{Arc, Mutex};
        use crate::game::Game;
        use crate::web::GamePage;

        #[test]
        fn from_game() {
            let arc = Arc::new(Mutex::new(Game::new("abc".into(), "german").unwrap()));
            let game_page = GamePage::from(arc.clone());
            assert_eq!(game_page.game_name, arc.lock().unwrap().name);
            assert_eq!(game_page.game_ident, arc.lock().unwrap().ident);
        }
    }
}