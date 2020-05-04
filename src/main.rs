#![feature(decl_macro, proc_macro_hygiene)]

extern crate askama;
extern crate colored;
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

use crate::game::cache::RamGameCache;
use std::thread::spawn;

pub mod game;
pub mod print;
pub mod random;
pub mod res;
mod web;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("Codenamer server v{}!", VERSION);
    println!("Languages: {}", res::words::languages().len());
    let web_handle = spawn(|| {
        web::start(Some(RamGameCache::new()));
    });
    let web_socket_handle = spawn(|| {
        web::socket::start();
    });
    web_handle.join().unwrap();
    web_socket_handle.join().unwrap();
}
