#![allow(dead_code)]
// #![allow(unused_variables)]
#![allow(unused_imports)]

use front::{
    clear, display_last_scene, display_scene_after_shoot, read_new_ship, read_shoot, render_mask,
    wait_for_enter,
};
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
    let mut alpha_buffer = [[[0 as char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE];
    let mut beta_buffer = [[[0 as char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE];

    clear();
    wait_for_enter("Player Alpha, place your ships!");
    for (i, size) in SHIP_SIZES.into_iter().enumerate() {
        let new_ship = read_new_ship(&game, Player::Alpha, &mut alpha_buffer, size);
        game.add_ship(Player::Alpha, new_ship, i).unwrap();
    }

    clear();
    wait_for_enter("Player Beta, place your ships!");
    for (i, size) in SHIP_SIZES.into_iter().enumerate() {
        let new_ship = read_new_ship(&game, Player::Beta, &mut alpha_buffer, size);
        game.add_ship(Player::Beta, new_ship, i).unwrap();
    }

    clear();
    wait_for_enter("Game starts!");

    while !game.is_over() {
        clear();
        wait_for_enter("Now player Alpha shoots!");

        let shoot = read_shoot(&game, &mut alpha_buffer, &mut beta_buffer, Player::Alpha);
        game.shoot(Player::Alpha, shoot);
        display_scene_after_shoot(&game, &mut alpha_buffer, &mut beta_buffer, Player::Alpha);

        clear();
        wait_for_enter("Now player Beta shoots!");

        let shoot = read_shoot(&game, &mut alpha_buffer, &mut beta_buffer, Player::Beta);
        game.shoot(Player::Beta, shoot);
        display_scene_after_shoot(&game, &mut alpha_buffer, &mut beta_buffer, Player::Beta);
    }

    display_last_scene(&game, &mut alpha_buffer, &mut beta_buffer);
}
