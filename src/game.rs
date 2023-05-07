use crate::low_level_logic::{
    create_ship, create_surround_mask, transpose, Orientation, BOARD_SIZE,
};

#[derive(Default, Copy, Clone, Debug)]
pub struct Game {
    pub board_alpha: u128,
    pub board_beta: u128,
    pub shoots_alpha: u128,
    pub shoots_beta: u128,
    pub step: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum Player {
    Alpha = 0,
    Beta = 1,
}

impl From<usize> for Player {
    fn from(value: usize) -> Self {
        // for C use:
        // unsafe { std::mem::transmute(value as u8 & 1) }
        // (Player) (value & 1)
        if value & 1 == 0 {
            Player::Alpha
        } else {
            Player::Beta
        }
    }
}

impl Game {
    /// Create a new game with the given board sizes.
    ///
    /// Returns `true` if the game is over.
    ///
    /// * `shoot` - the shoot to be made.
    pub fn step(&mut self, shoot: u128) -> bool {
        let player: Player = self.step.into();
        self.step += 1;
        match player {
            Player::Alpha => {
                self.shoots_alpha |= shoot;
                self.board_beta & !self.shoots_alpha == 0
            }
            Player::Beta => {
                self.shoots_beta |= shoot;
                self.board_alpha & !self.shoots_beta == 0
            }
        }
    }

    pub fn can_place_ship(&self, player: Player, ship: u128) -> bool {
        let mask = create_surround_mask(ship);

        let board = match player {
            Player::Alpha => self.board_alpha,
            Player::Beta => self.board_beta,
        };

        mask & board == 0
    }

    pub fn add_ship(&mut self, player: Player, ship: u128) -> Result<(), ()> {
        if !self.can_place_ship(player, ship) {
            return Err(());
        }

        match player {
            Player::Alpha => self.board_alpha |= ship,
            Player::Beta => self.board_beta |= ship,
        };
        Ok(())
    }

    pub fn add_ship_unchecked(&mut self, player: Player, ship: u128) {
        match player {
            Player::Alpha => self.board_alpha |= ship,
            Player::Beta => self.board_beta |= ship,
        }
    }

    pub fn get_hitted(&self, player: Player) -> u128 {
        match player {
            Player::Alpha => self.shoots_alpha & self.board_beta,
            Player::Beta => self.shoots_beta & self.board_alpha,
        }
    }

    pub fn get_intact(&self, player: Player) -> u128 {
        match player {
            Player::Alpha => self.board_beta & !self.shoots_alpha,
            Player::Beta => self.board_alpha & !self.shoots_beta,
        }
    }

    pub fn get_player_to_shoot(&self) -> Player {
        self.step.into()
    }

    pub fn get_shooted_player(&self) -> Player {
        (self.step - 1).into()
    }
}

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

mod test {
    #![allow(unused_imports)]

    use super::*;
    use crate::low_level_logic::{move_board, transpose, Direction};

    #[test]
    fn cant_place_a_ship() {
        let mut game = Game::default();
        let ship = transpose(create_ship(4));
        let ship = move_board(ship, 1, Direction::Down);
        assert!(game.can_place_ship(Player::Alpha, ship));
        game.add_ship(Player::Alpha, ship).unwrap();
        let ship = create_ship(3);
        assert!(!game.can_place_ship(Player::Alpha, ship));
        assert_eq!(game.add_ship(Player::Alpha, ship), Err(()));
    }

    #[test]
    fn place_a_ship() {
        let mut game = Game::default();
        let ship = transpose(create_ship(4));
        let ship = move_board(ship, 3, Direction::Down);
        assert!(game.can_place_ship(Player::Alpha, ship));
        game.add_ship(Player::Alpha, ship).unwrap();
        let ship = create_ship(3);
        let ship = move_board(ship, 1, Direction::Right);
        assert!(game.can_place_ship(Player::Alpha, ship));
        assert_eq!(game.add_ship(Player::Alpha, ship), Ok(()));
    }

    #[test]
    fn place_a_ship_near() {
        let mut game = Game::default();
        let ship = transpose(create_ship(4));
        let ship = move_board(ship, 2, Direction::Down);
        assert!(game.can_place_ship(Player::Alpha, ship));
        game.add_ship(Player::Alpha, ship).unwrap();
        let ship = create_ship(3);
        let ship = move_board(ship, 1, Direction::Right);
        assert!(game.can_place_ship(Player::Alpha, ship));
        assert_eq!(game.add_ship(Player::Alpha, ship), Ok(()));
    }
}
