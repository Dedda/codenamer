use crate::game::Game;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::game::cache::GameSessionCacheError::{GameNameTakenError, GameDoesNotExistsError};

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
        if self.games.lock().unwrap().remove(name).is_some() {
            Ok(())
        } else {
            Err(GameDoesNotExistsError(name.to_string()))
        }
    }
}