// #![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_imports)]

use front::{
    clear, display_last_scene, display_scene_after_shoot, read_new_ship, read_shoot, wait_for_enter,
};
use game::{Game, Player};

use constants::BOARD_SIZE;
use front::{CELL_SIZE, SHIP_SIZES};

mod board_api;
mod constants;
mod front;
mod game;

fn main() {
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

    let control_table = [
        (Player::Alpha, "Now player Alpha shoots!"),
        (Player::Beta, "Now player Beta shoots!"),
    ];

    let mut step = 0;
    while !game.is_over() {
        clear();
        wait_for_enter(control_table[step % 2].1);

        let shoot = read_shoot(
            &game,
            &mut alpha_buffer,
            &mut beta_buffer,
            control_table[step % 2].0,
        );
        game.shoot(control_table[step % 2].0, shoot);
        display_scene_after_shoot(
            &game,
            &mut alpha_buffer,
            &mut beta_buffer,
            control_table[step % 2].0,
        );
        step += 1;
    }

    display_last_scene(&game, &mut alpha_buffer, &mut beta_buffer);
}
