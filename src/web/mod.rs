use std::sync::{Arc, Mutex};

use askama::Template;
use rocket_contrib::serve::StaticFiles;

use crate::game::Game;
use crate::game_cache;

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

#[derive(Template)]
#[template(path = "game.html")]
struct GamePage {
    game_name: String,
    cards: Vec<Card>,
}

impl From<Arc<Mutex<Game>>> for GamePage {
    fn from(game: Arc<Mutex<Game>>) -> Self {
        let guard = game.lock().unwrap();
        Self {
            game_name: guard.name.clone(),
            cards: guard.words.iter().map(|w| Card { word: w.word.clone(), }).collect(),
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