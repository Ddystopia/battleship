use crate::{board_api::{
    create_ship, create_surround_mask, transpose, Orientation, BOARD_SIZE,
}, game::{Game, Player}};

// Base part of fiels. Represents something like [ ], [*], [~], [O]
pub struct Cell {}

// Ship placement logic for both players
pub fn place_ships(game: &mut Game, player: Player) {
    // Define the ships to be placed
    let ships = [
        create_ship(4),
        create_ship(2),
        create_ship(2),
        create_ship(2),
        create_ship(1),
        create_ship(1),
        create_ship(1),
        create_ship(1),
    ];

    for ship in ships {
        let mut ship_placed = false;

        while !ship_placed {
            // Get user input for ship placement (e.g., x, y, and orientation)
            // Interactive form with arrows is preferred
            // Flip ship by keybind like `f`
            let x: usize = 0; // TODO:
            let y: usize = 0; // TODO:
            let orientation: Orientation = Orientation::Horizontal; // TODO:

            let ship = match orientation {
                Orientation::Horizontal => ship << (y * BOARD_SIZE + x),
                Orientation::Vertical => transpose(ship) << (x * BOARD_SIZE + y),
            };

            let mask = create_surround_mask(ship);
            let players_board = match player {
                Player::Alpha => game.board_alpha,
                Player::Beta => game.board_beta,
            };

            if mask & players_board == 0 {
                game.add_ship_unchecked(player, ship);
                ship_placed = true;
            } else {
                // Show red cells for invalid ship placement
            }
        }
    }
}
