use crate::{carddb::CardDB, spawn_error::SpawnError};
use std::num::NonZeroU64;

use common::{card_entities::CardEnt, entities::PlayerId};
use once_cell::sync::OnceCell;

static CARDDB: OnceCell<CardDB> = OnceCell::new();

#[test_log::test]
fn card_tests() {
    test_card(db(), "Staunch Shieldmate");
    test_card(db(), "Plains");
    test_card(db(), "Revitalize");
    test_card(db(), "Staunch Shieldmate");
    test_card(db(), "Garruk's Gorehorn");
    test_card(db(), "Alpine Watchdog");
    test_card(db(), "Mistral Singer");
    test_card(db(), "Wishcoin Crab");
    test_card(db(), "Blood Glutton");
    test_card(db(), "Walking Corpse");
    test_card(db(), "Onakke Ogre");
    test_card(db(), "Colossal Dreadmaw");
    test_card(db(), "Concordia Pegasus");
}
#[allow(dead_code)]
fn db() -> &'static CardDB {
    CARDDB.get_or_init(|| CardDB::new())
}
#[test_log::test]
fn revitalize_test() {
    test_card(db(), "Revitalize");
}
#[test_log::test]
fn defiant_strike_test() {
    test_card(db(), "Defiant Strike");
}
#[test_log::test]
fn swift_response_test() {
    test_card(db(), "Swift Response");
}
#[test_log::test]
fn angelic_ascension_test() {
    test_card(db(), "Angelic Ascension");
}
#[test_log::test]
fn anointed_chorister_test() {
    test_card(db(), "Anointed Chorister");
}
#[allow(dead_code)]
fn test_card(db: &CardDB, card_name: &'static str) -> CardEnt {
    let spawned = db.try_spawn_card(card_name, PlayerId::from(NonZeroU64::new(1).unwrap()));
    if spawned.is_err() {
        println!("card {} failed to spawn", card_name);
    }
    if let Err(err)=&spawned
        && let SpawnError::Nom(err)=err
        && let nom::Err::Error(err)=err{
            println!("spawn error[");
            for error in &err.errors{
                println!("{:?}",error);
            } 
        }
    let spawned = spawned.unwrap();
    //println!("{:?}", spawned);
    spawned
}
