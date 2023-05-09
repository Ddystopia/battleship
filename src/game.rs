use crate::board_api::create_surround_mask;

const SHIPS_COUNT: usize = 5;

// • авианосец - 5 ячеек(клеток);
// • крейсер - 4 ячейки;
// • разрушитель - 3 ячейки;
// • подводная лодка - 3 ячейки;
// • катер - 2 ячейки.
#[derive(Default, Copy, Clone, Debug)]
pub struct Game {
    pub ships_alpha: [u128; SHIPS_COUNT],
    pub ships_beta: [u128; SHIPS_COUNT],
    pub shoots_alpha: u128,
    pub shoots_beta: u128,
    pub step: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum Player {
    Alpha = 0,
    Beta = 1,
}

impl Player {
    pub fn other(&self) -> Player {
        // for C use:
        // unsafe { std::mem::transmute(self as u8 ^ 1) }
        // (Player) (self ^ 1)
        match self {
            Player::Alpha => Player::Beta,
            Player::Beta => Player::Alpha,
        }
    }
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
        let outher_player_board = self.get_board(player.other());
        self.step += 1;

        match player {
            Player::Alpha => {
                self.shoots_alpha |= shoot;
                outher_player_board & !self.shoots_alpha == 0
            }
            Player::Beta => {
                self.shoots_beta |= shoot;
                outher_player_board & !self.shoots_beta == 0
            }
        }
    }

    pub fn get_board(&self, player: Player) -> u128 {
        let mut board: u128 = 0;
        for i in 0..SHIPS_COUNT {
            board |= match player {
                Player::Alpha => self.ships_alpha[i],
                Player::Beta => self.ships_beta[i],
            };
        }
        board
    }

    pub fn can_place_ship(&self, player: Player, ship: u128) -> bool {
        let mask = create_surround_mask(ship);
        let board = self.get_board(player);

        mask & board == 0
    }

    pub fn add_ship(&mut self, player: Player, ship: u128, layer: usize) -> Result<(), ()> {
        if !self.can_place_ship(player, ship) {
            return Err(());
        }

        match player {
            Player::Alpha => {
                self.ships_alpha[layer] |= ship;
            }
            Player::Beta => {
                self.ships_beta[layer] |= ship;
            }
        };
        Ok(())
    }

    pub fn add_ship_unchecked(&mut self, player: Player, ship: u128, layer: usize) {
        match player {
            Player::Alpha => {
                self.ships_alpha[layer] |= ship;
            }
            Player::Beta => {
                self.ships_beta[layer] |= ship;
            }
        }
    }

    pub fn get_hitted(&self, player: Player) -> u128 {
        let other_player = player.other();

        match player {
            Player::Alpha => self.shoots_alpha & self.get_board(other_player),
            Player::Beta => self.shoots_beta & self.get_board(other_player),
        }
    }

    pub fn get_intact(&self, player: Player) -> u128 {
        let other_player = player.other();

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
