#![feature(never_type)]
#![feature(const_option)]
#![feature(let_chains)]
#![deny(unused_must_use)]
use crate::write_schema::write_types;
use anyhow::Result;
use carddb::carddb;
use common::entities::PlayerId;
use game::game::build_game::GameBuilder;
use game::player::PlayerCon;
use once_cell::sync::OnceCell;
use std::mem;
use std::sync::{Arc, Mutex};
use warp::ws::WebSocket;
use warp::Filter;
mod write_schema;
pub static CARDDB: OnceCell<carddb::CardDB> = OnceCell::new();

type Pairing = Arc<Mutex<Option<WebSocket>>>;
#[tokio::main]
async fn main() {
    write_types();
    let pairer = Pairing::default();
    let pairer = warp::any().map(move || pairer.clone());
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    let static_files = warp::path("static").and(warp::fs::dir("static"));
    let game_setup =
        warp::path("gamesetup")
            .and(warp::ws())
            .and(pairer)
            .map(|ws: warp::ws::Ws, users| {
                // This will call our function if the handshake succeeds.
                ws.on_upgrade(move |socket| user_connected(socket, users))
            });

    warp::serve(hello.or(static_files).or(game_setup))
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn user_connected(ws1: WebSocket, pair: Pairing) {
    let mut state = pair.lock().unwrap();
    let mut current = None;
    mem::swap(&mut *state, &mut current);
    current = match current {
        None => Some(ws1),
        Some(ws2) => {
            tokio::task::spawn(launch_game(vec![ws1, ws2]));
            None
        }
    };
    mem::swap(&mut *state, &mut current);
}
async fn launch_game(sockets: Vec<WebSocket>) -> Result<()> {
    let db: &carddb::CardDB = CARDDB.get_or_init(|| carddb::CardDB::new());
    let mut gamebuild = GameBuilder::new();
    let mut deck = Vec::new();
    for _ in 0..10 {
        deck.push("Anointed Chorister");
    }
    for _ in 0..20 {
        deck.push("Aven Gagglemaster");
    }
    for _ in 0..30 {
        deck.push("Plains");
    }
    sockets
        .into_iter()
        .enumerate()
        .map(|(i, socket)| {
            gamebuild.add_player(&format!("p{}", i), &db, &deck, PlayerCon::new(socket))
        })
        .collect::<Result<Vec<PlayerId>>>()?;
    let mut game = gamebuild.build(&db)?;
    println!("Launching game!");
    game.run().await;
    //Fix this to make it print the winners name
    Ok(())
}
