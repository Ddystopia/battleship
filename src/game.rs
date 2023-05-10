use std::{thread::sleep, time::Duration};

use crate::{board_api::create_surround_mask, constants::BOT_BORDER_MASK, front::render_mask};

pub const SHIPS_COUNT: usize = 5;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
        self.step += 1;
        match player {
            Player::Alpha => {
                self.shoots_alpha |= shoot;
                self.get_board(Player::Beta) & !self.shoots_alpha == 0
            }
            Player::Beta => {
                self.shoots_beta |= shoot;
                self.get_board(Player::Alpha) & !self.shoots_beta == 0
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

    pub fn get_shoots(&self, player: Player) -> u128 {
        match player {
            Player::Alpha => self.shoots_alpha,
            Player::Beta => self.shoots_beta,
        }
    }

    pub fn can_place_ship(&self, player: Player, ship: u128) -> bool {
        let mask = create_surround_mask(ship);
        let board = self.get_board(player);

        mask & board == 0
    }

    pub fn shoot(&mut self, player: Player, shoot: u128) {
        match player {
            Player::Alpha => self.shoots_alpha |= shoot,
            Player::Beta => self.shoots_beta |= shoot,
        }

        let player_shoots = match player {
            Player::Alpha => &mut self.shoots_alpha,
            Player::Beta => &mut self.shoots_beta,
        };

        for layer_num in 0..SHIPS_COUNT {
            let layer = match player {
                Player::Alpha => self.ships_beta[layer_num],
                Player::Beta => self.ships_alpha[layer_num],
            };

            if layer & !*player_shoots == 0 {
                *player_shoots |= create_surround_mask(layer);
            }
        }
    }

    pub fn get_winner(&self) -> i8 {
        if self.is_over() {
            if self.get_board(Player::Beta) & !self.shoots_alpha == 0 {
                return Player::Alpha as i8;
            }
            if self.get_board(Player::Alpha) & !self.shoots_beta == 0 {
                return Player::Beta as i8;
            }
        }
        -1
    }

    pub fn is_over(&self) -> bool {
        // UN CHECKED
        let alpha_board = self.get_board(Player::Alpha);
        let beta_board = self.get_board(Player::Beta);
        let alpha_shoots = self.shoots_alpha;
        let beta_shoots = self.shoots_beta;

        if (beta_board & !alpha_shoots) == 0 {
            return true;
        }
        if (alpha_board & !beta_shoots) == 0 {
            return true;
        }
        false
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
        match player {
            Player::Alpha => self.shoots_alpha & self.get_board(Player::Beta),
            Player::Beta => self.shoots_beta & self.get_board(Player::Alpha),
        }
    }

    pub fn get_intact(&self, player: Player) -> u128 {
        match player {
            Player::Alpha => self.get_board(Player::Beta) & !self.shoots_alpha,
            Player::Beta => self.get_board(Player::Alpha) & !self.shoots_beta,
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
    use crate::board_api::{create_ship, transpose, wrapping_move, Direction};

    #[test]
    fn cant_place_a_ship() {
        let mut game = Game::default();
        let ship = transpose(create_ship(4));
        let ship = wrapping_move(ship, 1, Direction::Down);
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
        let ship = wrapping_move(ship, 3, Direction::Down);
        assert!(game.can_place_ship(Player::Alpha, ship));
        game.add_ship(Player::Alpha, ship, 1).unwrap();
        let ship = create_ship(3);
        let ship = wrapping_move(ship, 1, Direction::Right);
        assert!(game.can_place_ship(Player::Alpha, ship));
        assert_eq!(game.add_ship(Player::Alpha, ship, 1), Ok(()));
    }

    #[test]
    fn place_a_ship_near() {
        let mut game = Game::default();
        let ship = transpose(create_ship(4));
        let ship = wrapping_move(ship, 2, Direction::Down);
        assert!(game.can_place_ship(Player::Alpha, ship));
        game.add_ship(Player::Alpha, ship, 2).unwrap();
        let ship = create_ship(3);
        let ship = wrapping_move(ship, 1, Direction::Right);
        assert!(game.can_place_ship(Player::Alpha, ship));
        assert_eq!(game.add_ship(Player::Alpha, ship, 2), Ok(()));
    }
}
