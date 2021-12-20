use actix_web::{get, web, App, HttpServer, Responder};
use anyhow::{Result,bail};
//use actix_files::NamedFile;
//use actix_web::{HttpRequest, Result};
mod game; 
mod types;
mod carddb;
mod ability;
mod cost;

#[get("/{id}/{name}/index.html")]
async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}!", name,id)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(actix_files::Files::new("/static", "static").show_files_listing())
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
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
        let _game=gamebuild.build();
        Ok(())
    }
}