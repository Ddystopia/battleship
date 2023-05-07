use game::{Game, place_ships, Player};

mod game;
mod low_level_logic;

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

    let mut game = Game::default();

    place_ships(&mut game, Player::Alpha);
    // switch screen
    place_ships(&mut game, Player::Beta);
    // switch screen
    loop {
        let shoot = 0; // TODO:
        if game.step(shoot) { // step automaticly switches players
            break;
        }
        // switch screen
    }
}
