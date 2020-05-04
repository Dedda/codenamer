use crate::game::cache::{GameSessionCache, RamGameCache};

use askama::Template;
use std::sync::{Arc, Mutex};
use crate::game::Game;
use rocket_contrib::serve::StaticFiles;

lazy_static! {
    static ref GAME_CACHE: Arc<Mutex<Box<dyn GameSessionCache + Send>>> = {
        Arc::new(Mutex::new(Box::new(RamGameCache::new())))
    };
}

pub mod socket;

pub fn game_cache() -> Arc<Mutex<Box<dyn GameSessionCache + Send>>> {
    GAME_CACHE.clone()
}

pub fn start<C>(game_cache: Option<C>) where C: 'static + GameSessionCache + Send {
    {
        let mut static_game_cache = GAME_CACHE.lock().unwrap();
        if let Some(cache) = game_cache {
            *static_game_cache = Box::new(cache);
        }
        println!("Running games in cache: {}", static_game_cache.count());
    }
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
        num_running_games: GAME_CACHE.lock().unwrap().count(),
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
    let mut cache = GAME_CACHE.lock().unwrap();
    let g = if let Some(g) = cache.by_name(&game_name) {
        g
    } else {
        let game = Game::new(game_name.clone(), "english").unwrap();
        cache.put(game).unwrap();
        cache.by_name(&game_name).unwrap()
    };
    g.into()
}