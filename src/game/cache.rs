use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use itertools::Itertools;

use crate::game::cache::GameSessionCacheError::{GameDoesNotExistsError, GameNameTakenError};
use crate::game::Game;

#[derive(Debug)]
pub enum GameSessionCacheError {
    GameNameTakenError(String),
    GameDoesNotExistsError(String),
}

pub trait GameSessionCache {
    fn count(&self) -> usize;
    fn by_name(&self, name: &str) -> Option<Arc<Mutex<Game>>>;
    fn put(&mut self, game: Game) -> Result<(), GameSessionCacheError>;
    fn delete(&mut self, name: &str) -> Result<(), GameSessionCacheError>;
    fn cleanup(&mut self, max_age: &Duration);
}

pub struct RamGameCache {
    games: Mutex<HashMap<String, Arc<Mutex<Game>>>>,
}

impl RamGameCache {
    pub fn new() -> Self {
        Self {
            games: Mutex::new(HashMap::new()),
        }
    }
}

impl GameSessionCache for RamGameCache {
    fn count(&self) -> usize {
        self.games.lock().unwrap().keys().count()
    }

    fn by_name(&self, name: &str) -> Option<Arc<Mutex<Game>>> {
        self.games.lock().unwrap().get(name).cloned()
    }

    fn put(&mut self, game: Game) -> Result<(), GameSessionCacheError> {
        let mut games = self.games.lock().unwrap();
        if games.contains_key(&game.name) {
            Err(GameNameTakenError(game.name.clone()))
        } else {
            games.insert(game.name.clone(), Arc::new(Mutex::new(game)));
            Ok(())
        }
    }

    fn delete(&mut self, name: &str) -> Result<(), GameSessionCacheError> {
        println!("Game {} will be removed", name);
        if self.games.lock().unwrap().remove(name).is_some() {
            Ok(())
        } else {
            Err(GameDoesNotExistsError(name.to_string()))
        }
    }

    fn cleanup(&mut self, max_age: &Duration) {
        let now = SystemTime::now();
        let obsolete: Vec<String> = {
            let games_guard = self.games.lock().unwrap();
            let obsolete: Vec<String> = games_guard.iter().filter(|(_, game)| {
                let game_guard = game.lock().unwrap();
                let elapsed = now.duration_since(game_guard.created.clone()).unwrap();
                elapsed.gt(max_age)
            }).map(|(name, _)| name.clone()).collect();
            obsolete
        }.into_iter().unique().collect();
        obsolete.iter().for_each(|name| {
            self.delete(name).unwrap();
        });
    }
}