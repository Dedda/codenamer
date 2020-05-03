#![feature(decl_macro, proc_macro_hygiene)]

extern crate askama;
#[macro_use]
extern crate lazy_static;
extern crate rand;
#[macro_use]
extern crate rocket;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use crate::game::cache::RamGameCache;

pub mod game;
pub mod random;
pub mod res;
mod web;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("Codenamer server v{}!", VERSION);
    println!("Languages: {}", res::words::languages().len());
    web::start(Some(RamGameCache::new()));
}
