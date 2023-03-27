#![feature(never_type)]
#![feature(const_option)]
#![feature(let_chains)]
#![deny(unused_must_use)]

use carddb::carddb::CardDB;
use once_cell::sync::OnceCell;
pub mod actions;
pub mod client_message;
pub mod ent_maps;
pub mod errors;
pub mod event;
pub mod game;
pub mod player;
#[allow(dead_code)] //Used in unit tests
static CARDDB: OnceCell<CardDB> = OnceCell::new();
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    mod aven_gagglemaster_tests;
    mod baneslayer_angel_tests;
    mod card_tests;
    mod common_test;
    mod counter_tests;
    mod lethal_damage;
    mod mock_tests;
    mod swift_response_test;
}
