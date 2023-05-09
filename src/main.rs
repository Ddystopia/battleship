mod board_api;
mod front;
mod game;
mod constants;

use front::{place_ships, Cell};
use game::{Game, Player};

fn main() {
    // Main menu
    //  - Help
    //  - Play
    //
    // Get ships placed from players
    // Player Alpha
    // While has ships to place
    //   Choose a ship
    //   Choose a valid place
    //
    // Switch screens
    //
    // Player Beta
    // While has ships to place
    //   Choose a ship
    //   Choose a valid place
    //
    // Switch screens
    //
    // Start main loop
    //   Let player choose a shell and shoot
    //   if game is over break
    //
    //   Switch screens
    //
    //  Greet the winner and quit
    //

    let mut board_buffer_alpha: [[Cell; 10]; 0] = []; // Cell[10][10]
    let mut board_buffer_beta: [[Cell; 10]; 0] = [];
    let mut shoot_buffer_alpha: [[Cell; 10]; 0] = [];
    let mut shoot_buffer_beta: [[Cell; 10]; 0] = [];

    let mut game = Game::default();

    place_ships(&mut game, Player::Alpha);
    // switch screen
    place_ships(&mut game, Player::Beta);
    // switch screen
    loop {
        let shoot = 0; // TODO:
        if game.step(shoot) {
            // step automaticly switches players
            break;
        }
        // switch screen
    }
}
