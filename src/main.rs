#![allow(dead_code)]
// #![allow(unused_variables)]
#![allow(unused_imports)]

use front::{clear, read_new_ship, render_mask, wait_for_enter};
use game::{Game, Player};

use constants::{BOARD_BORDER, BOARD_SIZE};
use front::{CELL_SIZE, SHIP_SIZES};

mod board_api;
mod constants;
mod front;
mod game;

fn main() {
    clear();
    render_mask(BOARD_BORDER);

    let mut game = Game::default();
    let mut buffer = [[[0 as char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE];

    wait_for_enter("Player Alpha, place your ships!");
    for (i, size) in SHIP_SIZES.into_iter().enumerate() {
        let new_ship = read_new_ship(&game, Player::Alpha, &mut buffer, size);
        game.add_ship(Player::Alpha, new_ship, i).unwrap();
    }

    wait_for_enter("Player Beta, place your ships!");
    for (i, size) in SHIP_SIZES.into_iter().enumerate() {
        let new_ship = read_new_ship(&game, Player::Beta, &mut buffer, size);
        game.add_ship(Player::Beta, new_ship, i).unwrap();
    }

    wait_for_enter("Game starts!");
}
