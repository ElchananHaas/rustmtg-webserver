use anyhow::{Result,bail};
use warp::Filter;
use tokio::sync::{RwLock};
use warp::ws::{Message,WebSocket};
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use std::sync::{Arc};
//use actix_files::NamedFile;
//use actix_web::{HttpRequest, Result};
mod game; 
mod types;
mod carddb;
mod ability;
mod cost;
mod event;


type Pairing = Arc<RwLock<Option<warp::ws::Ws>>>;
#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let pairer=Pairing::default();
    let pairer = warp::any().map(move || pairer.clone());
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));
    let static_files = warp::path("static").and(warp::fs::dir("static"));
    let game_setup= warp::path("gamesetup").and(warp::ws()).and(pairer).map(|ws: warp::ws::Ws, users| {
        // This will call our function if the handshake succeeds.
        ws.on_upgrade(move |socket| user_connected(socket, users))
    });

    warp::serve(hello.or(static_files).or(game_setup))
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn user_connected(ws: WebSocket, pair: Pairing) {
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    user_ws_tx
    .send(Message::text("Hello!".clone()))
    .unwrap_or_else(|e| {
        eprintln!("websocket send error: {}", e);
    })
    .await;
}
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_game_init()->Result<()> {
        let db=carddb::CardDB::new();
        let mut gamebuild=game::GameBuilder::new();
        let mut deck=Vec::new();
        for _ in 1..60{
            deck.push(String::from("Staunch Shieldmate"));
        }
        gamebuild.add_player("p1",&db,&deck)?;
        gamebuild.add_player("p2",&db,&deck)?;
        let _game=gamebuild.build(&db);
        Ok(())
    }
}