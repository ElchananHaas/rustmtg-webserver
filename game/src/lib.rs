#![feature(never_type)]
#![feature(const_option)]
#![feature(let_chains)]
#![deny(unused_must_use)]
pub mod game;
pub mod event;
pub mod errors;
pub mod player;
pub mod ent_maps;
pub mod client_message;
pub mod actions;
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
