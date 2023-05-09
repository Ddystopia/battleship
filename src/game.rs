use crate::board_api::create_surround_mask;

const SHIPS_COUNT: usize = 5;

// • авианосец - 5 ячеек(клеток);
// • крейсер - 4 ячейки;
// • разрушитель - 3 ячейки;
// • подводная лодка - 3 ячейки;
// • катер - 2 ячейки.
#[derive(Default, Copy, Clone, Debug)]
pub struct Game {
    pub alpha_ships: [u128; SHIPS_COUNT],
    pub beta_ships: [u128; SHIPS_COUNT],
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
        let other_player = self.other_player(player);
        self.step += 1;

        match player {
            Player::Alpha => {
                self.shoots_alpha |= shoot;
                self.get_board(other_player) & !self.shoots_alpha == 0
            }
            Player::Beta => {
                self.shoots_beta |= shoot;
                self.get_board(other_player) & !self.shoots_beta == 0
            }
        }
    }

    fn other_player(&self, player: Player) -> Player {
        match player {
            Player::Alpha => Player::Beta,
            Player::Beta => Player::Alpha,
        }
    }

    pub fn get_board(&self, player: Player) -> u128 {
        let mut board: u128 = 0;
        for i in 0..SHIPS_COUNT {
            board |= match player {
                Player::Alpha => self.alpha_ships[i],
                Player::Beta => self.beta_ships[i],
            };
        }
        board
    }

    pub fn can_place_ship(&self, player: Player, ship: u128) -> bool {
        let mask = create_surround_mask(ship);
        let board = self.get_board(player);

        mask & board == 0
    }

    pub fn add_ship(&mut self, player: Player, ship: u128, layer: u8) -> Result<(), ()> {
        if !self.can_place_ship(player, ship) {
            return Err(());
        }

        match player {
            Player::Alpha => {
                self.alpha_ships[layer as usize] |= ship;
            }
            Player::Beta => {
                self.beta_ships[layer as usize] |= ship;
            }
        };
        Ok(())
    }

    pub fn add_ship_unchecked(&mut self, player: Player, ship: u128, layer: u8) {
        match player {
            Player::Alpha => {
                self.alpha_ships[layer as usize] |= ship;
            }
            Player::Beta => {
                self.beta_ships[layer as usize] |= ship;
            }
        }
    }

    pub fn get_hitted(&self, player: Player) -> u128 {
        let other_player = self.other_player(player);

        match player {
            Player::Alpha => self.shoots_alpha & self.get_board(other_player),
            Player::Beta => self.shoots_beta & self.get_board(other_player),
        }
    }

    pub fn get_intact(&self, player: Player) -> u128 {
        let other_player = self.other_player(player);

        match player {
            Player::Alpha => self.get_board(other_player) & !self.shoots_alpha,
            Player::Beta => self.get_board(other_player) & !self.shoots_beta,
        }
    }

    pub fn get_player_to_shoot(&self) -> Player {
        self.step.into()
    }

    pub fn get_shooted_player(&self) -> Player {
        (self.step - 1).into()
    }
}

mod test {
    #![allow(unused_imports)]

    use super::*;
    use crate::board_api::{create_ship, move_board, transpose, Direction};

    #[test]
    fn cant_place_a_ship() {
        let mut game = Game::default();
        let ship = transpose(create_ship(4));
        let ship = move_board(ship, 1, Direction::Down);
        assert!(game.can_place_ship(Player::Alpha, ship));
        game.add_ship(Player::Alpha, ship, 0).unwrap();
        let ship = create_ship(3);
        assert!(!game.can_place_ship(Player::Alpha, ship));
        assert_eq!(game.add_ship(Player::Alpha, ship, 0), Err(()));
    }

    #[test]
    fn place_a_ship() {
        let mut game = Game::default();
        let ship = transpose(create_ship(4));
        let ship = move_board(ship, 3, Direction::Down);
        assert!(game.can_place_ship(Player::Alpha, ship));
        game.add_ship(Player::Alpha, ship, 1).unwrap();
        let ship = create_ship(3);
        let ship = move_board(ship, 1, Direction::Right);
        assert!(game.can_place_ship(Player::Alpha, ship));
        assert_eq!(game.add_ship(Player::Alpha, ship, 1), Ok(()));
    }

    #[test]
    fn place_a_ship_near() {
        let mut game = Game::default();
        let ship = transpose(create_ship(4));
        let ship = move_board(ship, 2, Direction::Down);
        assert!(game.can_place_ship(Player::Alpha, ship));
        game.add_ship(Player::Alpha, ship, 2).unwrap();
        let ship = create_ship(3);
        let ship = move_board(ship, 1, Direction::Right);
        assert!(game.can_place_ship(Player::Alpha, ship));
        assert_eq!(game.add_ship(Player::Alpha, ship, 2), Ok(()));
    }
}
