use crate::board_api::create_surround_mask;

pub const SHIP_SIZES: [usize; 5] = [5, 4, 3, 3, 2];
pub const SHIPS_COUNT: usize = SHIP_SIZES.len();

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
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Player {
    Alpha,
    Beta,
}

impl Player {
    pub fn other(&self) -> Player {
        match self {
            Player::Alpha => Player::Beta,
            Player::Beta => Player::Alpha,
        }
    }
}

impl Game {
    pub fn get_board(&self, player: Player) -> u128 {
        let ships = match player {
            Player::Alpha => self.ships_alpha,
            Player::Beta => self.ships_beta,
        };

        ships.into_iter().reduce(|acc, ship| acc | ship).unwrap_or(0)
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
        let (player_shoots, mut layers) = match player {
            Player::Alpha => (&mut self.shoots_alpha, self.ships_beta.into_iter()),
            Player::Beta => (&mut self.shoots_beta, self.ships_alpha.into_iter()),
        };

        *player_shoots |= shoot;

        let shoots = *player_shoots;

        if let Some(layer) = layers.find(move |layer| layer & !shoots == 0) {
            *player_shoots |= create_surround_mask(layer);
        }
    }

    pub fn get_winner(&self) -> Option<Player> {
        if self.get_board(Player::Beta) & !self.shoots_alpha == 0 {
            return Some(Player::Alpha);
        }
        if self.get_board(Player::Alpha) & !self.shoots_beta == 0 {
            return Some(Player::Beta);
        }
        None
    }

    pub fn is_over(&self) -> bool {
        self.get_winner().is_some()
    }

    pub fn add_ship(&mut self, player: Player, ship: u128, layer: usize) -> Result<(), ()> {
        if !self.can_place_ship(player, ship) {
            return Err(());
        }

        match player {
            Player::Alpha => self.ships_alpha[layer] |= ship,
            Player::Beta => self.ships_beta[layer] |= ship,
        };

        Ok(())
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
