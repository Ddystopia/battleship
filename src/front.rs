use std::io::{self, Write};
use std::{io::Read, process::Command};
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW, VMIN, VTIME};

use crate::board_api::{saturated_move, Direction};
use crate::{
    board_api::{board_get, create_ship, create_surround_mask, transpose, Orientation},
    constants::BOARD_SIZE,
    game::{Game, Player},
};

pub type OutputBuffer = [[[char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE];

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
                Orientation::Horizontal => ship >> (y * BOARD_SIZE + x),
                Orientation::Vertical => transpose(ship) >> (y * BOARD_SIZE + x),
            };

            let mask = create_surround_mask(ship);
            let players_board = game.get_board(player);

            if mask & players_board == 0 {
                game.add_ship_unchecked(player, ship, 0);
                ship_placed = true;
            } else {
                // Show red cells for invalid ship placement
            }
        }
    }
}

// board -- 10x10 array of cells
//
// cells:
//  - [ ] -- miss
//  - [*] -- hit
//  - [~] -- unknown
//  - [O] -- your ship
//  - [X] -- collided ship
//  - [o] -- new ship
//
// colors :
//   - [ ] -- \033[00m[ ]\033[0m
//   - [*] -- \033[31m[*]\033[0m
//   - [~] -- \033[34m[~]\033[0m
//   - [O] -- \033[32m[O]\033[0m
//   - [X] -- \033[33m[X]\033[0m
//   - [o] -- \033[32m[o]\033[0m
//
// Cell size: 12
//
// Buffer size: 12 * 10 * 10 = 1200

const CELL_SIZE: usize = 12; // color identifier (in rust: \u{001B}, in C: \033) + [ + color (2) + m + cell + color identifier (in rust: \u{001B}, in C: \033) + [ + 0 + m
const fn create_cell(val: &str) -> [char; CELL_SIZE] {
    let mut cell = [0 as char; CELL_SIZE];
    let mut i = 0;
    let len = val.len();
    assert!(len >= CELL_SIZE);
    unsafe {
        let raw = val as *const str as *const u8;
        while i < CELL_SIZE {
            cell[i] = *(raw.add(i)) as char;
            i += 1;
        }
    }
    cell
}
const CELL_MISS: [char; CELL_SIZE] = create_cell("\u{001B}[00m[ ]\u{001B}[0m");
const CELL_HIT: [char; CELL_SIZE] = create_cell("\u{001B}[31m[*]\u{001B}[0m");
const CELL_UNKNOWN: [char; CELL_SIZE] = create_cell("\u{001B}[34m[~]\u{001B}[0m");
const CELL_SHIP: [char; CELL_SIZE] = create_cell("\u{001B}[32m[O]\u{001B}[0m");
const CELL_COLLISION: [char; CELL_SIZE] = create_cell("\u{001B}[33m[X]\u{001B}[0m");
const CELL_NEW_SHIP: [char; CELL_SIZE] = create_cell("\u{001B}[32m[o]\u{001B}[0m");

fn copy_cell(
    cell: &[char; CELL_SIZE],
    buffer: &mut [[[char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE],
    x: usize,
    y: usize,
) {
    for i in 0..CELL_SIZE {
        buffer[y][x][i] = cell[i];
    }
}

fn clear_buffer(buffer: &mut [[[char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE]) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            for i in 0..CELL_SIZE {
                buffer[y][x][i] = 0 as char;
            }
        }
    }
}

fn render_unknown(buffer: &mut [[[char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE]) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            copy_cell(&CELL_UNKNOWN, buffer, x, y);
        }
    }
}

fn render_board_ships(
    game: &Game,
    player: Player,
    buffer: &mut [[[char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE],
) {
    let board = game.get_board(player);
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if board_get(board, x, y) {
                copy_cell(&CELL_SHIP, buffer, x, y);
            }
        }
    }
}

fn render_board_ships_n_new_ship(
    game: &Game,
    player: Player,
    buffer: &mut [[[char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE],
    new_ship: u128,
) {
    render_board_ships(game, player, buffer);

    let board = game.get_board(player);
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if board_get(new_ship, x, y) {
                copy_cell(&CELL_NEW_SHIP, buffer, x, y);
            }
        }
    }
    let collision = new_ship & create_surround_mask(board);
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if board_get(collision, x, y) {
                copy_cell(&CELL_COLLISION, buffer, x, y);
            }
        }
    }
}

fn render_board_hits(
    game: &Game,
    player: Player,
    buffer: &mut [[[char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE],
) {
    let board = game.get_hitted(player);
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if board_get(game.get_board(player), x, y) {
                copy_cell(&CELL_SHIP, buffer, x, y);
            }
        }
    }
}

fn display_board(buffer: &[[[char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE]) {
    let mut stdout = io::stdout();
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            for i in 0..CELL_SIZE {
                stdout
                    .write_all(buffer[y][x][i].encode_utf8(&mut [0; 4]).as_bytes())
                    .unwrap();
            }
        }
        stdout.write_all(b"\n").unwrap();
    }
    stdout.flush().unwrap();
}

fn read_new_ship(
    game: &Game,
    player: Player,
    buffer: &mut [[[char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE],
    ship_size: usize,
) -> u128 {
    let mut new_ship = create_ship(ship_size);
    Command::new("clear").status().unwrap();
    clear_buffer(buffer);
    render_unknown(buffer);
    render_board_ships_n_new_ship(game, player, buffer, new_ship);
    display_board(buffer);

    loop {
        let mut termios = Termios::from_fd(0).unwrap();
        termios.c_lflag &= !(ECHO | ICANON);
        termios.c_cc[VMIN] = 1;
        termios.c_cc[VTIME] = 0;
        tcsetattr(0, TCSANOW, &termios).unwrap();
        let mut buf = [0u8; 1];
        io::stdin().read_exact(&mut buf).unwrap();
        termios.c_lflag |= ECHO | ICANON;
        tcsetattr(0, TCSANOW, &termios).unwrap();
        let input = buf[0] as char;

        clear_buffer(buffer);
        render_unknown(buffer);
        if input == '\n' && game.can_place_ship(player, new_ship) {
            break;
        }

        if input == 'f' {
            new_ship = transpose(new_ship);
        }

        if input == 'k' {
            new_ship = saturated_move(new_ship, Direction::Up);
        }
        if input == 'j' {
            new_ship = saturated_move(new_ship, Direction::Down);
        }
        if input == 'h' {
            new_ship = saturated_move(new_ship, Direction::Left);
        }
        if input == 'l' {
            new_ship = saturated_move(new_ship, Direction::Right);
        }

        render_board_ships_n_new_ship(game, player, buffer, new_ship);

        Command::new("clear").status().unwrap();
        display_board(buffer);
    }
    new_ship
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_alpha_board() {
        let mut game = Game::default();
        let mut buffer = [[[0 as char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE];

        game.add_ship(
            Player::Alpha,
            read_new_ship(&game, Player::Alpha, &mut buffer, 5),
            0,
        )
        .unwrap();

        game.add_ship(
            Player::Alpha,
            read_new_ship(&game, Player::Alpha, &mut buffer, 4),
            1,
        )
        .unwrap();

        game.add_ship(
            Player::Alpha,
            read_new_ship(&game, Player::Alpha, &mut buffer, 3),
            2,
        )
        .unwrap();

        game.add_ship(
            Player::Alpha,
            read_new_ship(&game, Player::Alpha, &mut buffer, 3),
            3,
        )
        .unwrap();

        game.add_ship(
            Player::Alpha,
            read_new_ship(&game, Player::Alpha, &mut buffer, 2),
            4,
        )
        .unwrap();
    }
}
