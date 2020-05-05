#![feature(decl_macro, proc_macro_hygiene)]

extern crate askama;
extern crate colored;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate rand;
#[macro_use]
extern crate rocket;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate ws;

use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Duration;

use crate::game::cache::{GameSessionCache, RamGameCache};

pub mod game;
pub mod print;
pub mod random;
pub mod res;
mod web;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

lazy_static! {
    static ref GAME_CACHE: Arc<Mutex<Box<dyn GameSessionCache + Send>>> = {
        Arc::new(Mutex::new(Box::new(RamGameCache::new())))
    };
}

pub fn game_cache() -> Arc<Mutex<Box<dyn GameSessionCache + Send>>> {
    GAME_CACHE.clone()
}

fn main() {
    println!("Codenamer server v{}!", VERSION);
    println!("Languages: {}", res::words::languages().len());
    {
        let gc = game_cache();
        let mut cache =gc .lock().unwrap();
        *cache = Box::new(RamGameCache::new());
    }
    let web_handle = spawn(|| {
        web::start();
    });
    let web_socket_handle = spawn(|| {
        web::socket::start();
    });
    let clean_cache_handle = spawn(|| {
        loop {
            sleep(Duration::from_secs(5));
            let gc = game_cache();
            let mut cache = gc.lock().unwrap();
            cache.cleanup(&Duration::from_secs(24 * 60 * 60 * 1000));
        }
    });
    web_handle.join().unwrap();
    web_socket_handle.join().unwrap();
    clean_cache_handle.join().unwrap();
}
