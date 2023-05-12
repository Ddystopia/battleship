// #![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_imports)]

use game::{Game, Player, SHIP_SIZES};

use front::{clear, IO, wait_for_enter};

mod board_api;
mod constants;
mod front;
mod game;

fn main() {
    let mut game = Game::default();
    let mut io = IO::default();


    clear();
    wait_for_enter("Player Alpha, place your ships!");
    for (i, size) in SHIP_SIZES.into_iter().enumerate() {
        let new_ship = io.read_new_ship(&game, Player::Alpha, size);
        game.add_ship(Player::Alpha, new_ship, i).unwrap();
    }

    clear();
    wait_for_enter("Player Beta, place your ships!");
    for (i, size) in SHIP_SIZES.into_iter().enumerate() {
        let new_ship = io.read_new_ship(&game, Player::Beta, size);
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

        let shoot = io.read_shoot(
            &game,
            control_table[step % 2].0,
        );
        game.shoot(control_table[step % 2].0, shoot);
        io.display_scene_after_shoot(
            &game,
            control_table[step % 2].0,
        );
        step += 1;
    }

    io.display_last_scene(&game);
}
