use anyhow::{bail, Result};
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use hecs::Entity;
use once_cell::sync::OnceCell;
use std::mem;
use std::sync::{Arc, Mutex};
use warp::ws::{Message, WebSocket};
use warp::Filter;
//use actix_files::NamedFile;
//use actix_web::{HttpRequest, Result};
mod ability;
mod carddb;
mod cost;
mod event;
mod game;
mod types;
mod components;

static CARDDB: OnceCell<carddb::CardDB> = OnceCell::new();
type Pairing = Arc<Mutex<Option<WebSocket>>>;
#[tokio::main]
async fn main() {
    CARDDB.set(carddb::CardDB::new()).unwrap();
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
    //let (mut user_ws_tx, mut user_ws_rx) = ws.split();
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
async fn launch_game(mut sockets: Vec<WebSocket>) -> Result<()> {
    let db: &carddb::CardDB = CARDDB.get().expect("Card database not initialized!");
    let mut gamebuild = game::GameBuilder::new();
    let mut deck = Vec::new();
    for _ in 1..60 {
        deck.push(String::from("Staunch Shieldmate"));
    }
    let users: Vec<Entity> = sockets
        .into_iter()
        .enumerate()
        .map(|(i, socket)| {
            let wrappedsocket = Box::new(Mutex::new(socket));
            gamebuild.add_player(&format!("p{}", i), &db, &deck, wrappedsocket)
        })
        .collect::<Result<Vec<Entity>>>()?;
    let mut game = gamebuild.build(&db)?;
    println!("Launching game!");
    game.run().await;
    //Fix this to make it print the winners name
    Ok(())
}
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_game_init() -> Result<()> {
        let db = carddb::CardDB::new();
        let mut gamebuild = game::GameBuilder::new();
        let mut deck = Vec::new();
        for _ in 1..60 {
            deck.push(String::from("Staunch Shieldmate"));
        }
        gamebuild.add_player("p1", &db, &deck)?;
        gamebuild.add_player("p2", &db, &deck)?;
        let _game = gamebuild.build(&db);
        Ok(())
    }
}
