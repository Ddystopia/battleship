#![allow(dead_code)]
// #![allow(unused_variables)]
#![allow(unused_imports)]

use std::process::Command;

use front::{read_new_ship, render_mask};
use game::{Game, Player};

use constants::{BOARD_SIZE, BOARD_BORDER};
use front::{CELL_SIZE, SHIP_SIZES};

mod board_api;
mod constants;
mod front;
mod game;

fn main() {
    Command::new("clear").status().unwrap();
    render_mask(BOARD_BORDER);

    let mut game = Game::default();
    let mut buffer = [[[0 as char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE];

    for (i, size) in SHIP_SIZES.into_iter().enumerate() {
        let new_ship = read_new_ship(&game, Player::Alpha, &mut buffer, size);
        game.add_ship(Player::Alpha, new_ship, i).unwrap();
    }

    for (i, size) in SHIP_SIZES.into_iter().enumerate() {
        let new_ship = read_new_ship(&game, Player::Beta, &mut buffer, size);
        game.add_ship(Player::Beta, new_ship, i).unwrap();
    }
}
