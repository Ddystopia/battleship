use std::io::{self, Write};
use std::{io::Read, process::Command};
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW, VMIN, VTIME};

use crate::board_api::{saturated_move, Direction};
use crate::{
    board_api::{board_get, create_ship, create_surround_mask, transpose, Orientation},
    constants::BOARD_SIZE,
    game::{Game, Player},
};

// Base part of fiels. Represents something like [ ], [*], [~], [O]
pub type Cell = [char; CELL_SIZE];
pub type OutputBuffer = [[Cell; BOARD_SIZE]; BOARD_SIZE];
pub const CELL_SIZE: usize = 12; // color identifier (in rust: \u{001B}, in C: \033) + [ + color (2) + m + cell + color identifier (in rust: \u{001B}, in C: \033) + [ + 0 + m
pub const SHIP_SIZES: [usize; 5] = [5, 4, 3, 3, 2];

const CELL_MISS: Cell = create_cell("\u{001B}[00m[ ]\u{001B}[0m");
const CELL_HIT: Cell = create_cell("\u{001B}[31m[*]\u{001B}[0m");
const CELL_UNKNOWN: Cell = create_cell("\u{001B}[34m[~]\u{001B}[0m");
const CELL_SHIP: Cell = create_cell("\u{001B}[32m[O]\u{001B}[0m");
const CELL_COLLISION: Cell = create_cell("\u{001B}[33m[X]\u{001B}[0m");
const CELL_NEW_SHIP: Cell = create_cell("\u{001B}[32m[o]\u{001B}[0m");

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

const fn create_cell(val: &str) -> Cell {
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

// void copy_cell(const Cell cell, OutputBuffer buffer, size_t x, size_t y) {
//     for (size_t i = 0; i < CELL_SIZE; ++i) {
//         buffer[y][x][i] = cell[i];
//     }
// }
fn copy_cell(cell: &Cell, buffer: &mut OutputBuffer, x: usize, y: usize) {
    for i in 0..CELL_SIZE {
        buffer[y][x][i] = cell[i];
    }
}

fn clear_buffer(buffer: &mut OutputBuffer) {
    // memset
    // memset(buffer, 0, sizeof(buffer))
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            for i in 0..CELL_SIZE {
                buffer[y][x][i] = 0 as char;
            }
        }
    }
}

fn render_unknown(buffer: &mut OutputBuffer) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            copy_cell(&CELL_UNKNOWN, buffer, x, y);
        }
    }
}

fn render_board_ships(board: u128, buffer: &mut OutputBuffer) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if board_get(board, x, y) {
                copy_cell(&CELL_SHIP, buffer, x, y);
            }
        }
    }
}

fn render_board_ships_n_new_ship(board: u128, buffer: &mut OutputBuffer, new_ship: u128) {
    render_board_ships(board, buffer);

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

fn render_board_hits(board: u128, buffer: &mut OutputBuffer) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if board_get(board, x, y) {
                copy_cell(&CELL_SHIP, buffer, x, y);
            }
        }
    }
}

fn display_board(buffer: &OutputBuffer) {
    let mut stdout = io::stdout();

    // Allocate the temporary buffer on the stack
    let mut temp_buffer = [0u8; BOARD_SIZE * CELL_SIZE + 1]; // +1 for the newline character

    for y in 0..BOARD_SIZE {
        let mut idx = 0;
        for x in 0..BOARD_SIZE {
            for i in 0..CELL_SIZE {
                // temp_buffer[idx++] = buffer[y][x][i];
                let c = buffer[y][x][i];
                let c_len = c.len_utf8();
                c.encode_utf8(&mut temp_buffer[idx..idx + c_len]);
                idx += c_len;
            }
        }
        // Add a newline character at the end of the line
        temp_buffer[idx] = b'\n';
        idx += 1;

        // Print the line
        stdout.write_all(&temp_buffer[..idx]).unwrap();
    }
    stdout.flush().unwrap();
}

pub fn read_new_ship(
    game: &Game,
    player: Player,
    buffer: &mut OutputBuffer,
    ship_size: usize,
) -> u128 {
    let mut new_ship = create_ship(ship_size);

    clear_buffer(buffer);
    render_unknown(buffer);

    loop {
        Command::new("clear").status().unwrap();

        let board = game.get_board(player);

        render_board_ships_n_new_ship(board, buffer, new_ship);
        display_board(buffer);

        let input = getchar();

        clear_buffer(buffer);
        render_unknown(buffer);

        if input == '\n' && game.can_place_ship(player, new_ship) {
            break;
        }

        if input == 'f' {
            new_ship = transpose(new_ship);
        }

        new_ship = move_by_user_input(new_ship, input);
    }

    new_ship
}

fn getchar() -> char {
    let mut termios = Termios::from_fd(0).unwrap();
    termios.c_lflag &= !(ECHO | ICANON);
    termios.c_cc[VMIN] = 1;
    termios.c_cc[VTIME] = 0;
    tcsetattr(0, TCSANOW, &termios).unwrap();
    let mut buf = [0u8; 1];
    io::stdin().read_exact(&mut buf).unwrap();
    termios.c_lflag |= ECHO | ICANON;
    tcsetattr(0, TCSANOW, &termios).unwrap();
    buf[0] as char
}

// will be reused for shooting
// you are free to rename it as you want
fn move_by_user_input(board: u128, input: char) -> u128 {
    if input == 'k' || input == 'w' {
        saturated_move(board, Direction::Up)
    } else if input == 'j' || input == 's' {
        saturated_move(board, Direction::Down)
    } else if input == 'h' || input == 'a' {
        saturated_move(board, Direction::Left)
    } else if input == 'l' || input == 'd' {
        saturated_move(board, Direction::Right)
    } else {
        board
    }
}

pub fn render_mask(mask: u128) {
    let mut buffer = [[[0 as char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE];
    render_unknown(&mut buffer);
    render_board_ships(mask, &mut buffer);
    display_board(&buffer);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_alpha_board() {
        let mut game = Game::default();
        let mut buffer = [[[0 as char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE];

        for (i, size) in SHIP_SIZES.into_iter().enumerate() {
            let new_ship = read_new_ship(&game, Player::Alpha, &mut buffer, size);
            game.add_ship(Player::Alpha, new_ship, i).unwrap();
        }

        for (i, size) in SHIP_SIZES.into_iter().enumerate() {
            let new_ship = read_new_ship(&game, Player::Beta, &mut buffer, size);
            game.add_ship(Player::Beta, new_ship, i).unwrap();
        }
    }
}
