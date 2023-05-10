use std::io::{BufWriter, Write};

use crate::{
    board_api::{board_get, create_ship, create_surround_mask, transpose, Orientation},
    constants::BOARD_SIZE,
    game::{Game, Player},
};

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
//
// colors :
//   - [ ] -- \033[00m[ ]\033[0m
//   - [*] -- \033[31m[*]\033[0m
//   - [~] -- \033[34m[~]\033[0m
//   - [O] -- \033[32m[O]\033[0m
//   - [X] -- \033[33m[X]\033[0m
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

fn copy_cell(
    cell: &[char; CELL_SIZE],
    buffer: &mut [[[char; CELL_SIZE]; 10]; 10],
    x: usize,
    y: usize,
) {
    for i in 0..CELL_SIZE {
        buffer[y][x][i] = cell[i];
    }
}

fn clear_buffer(buffer: &mut [[[char; CELL_SIZE]; 10]; 10]) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            for i in 0..CELL_SIZE {
                buffer[y][x][i] = 0 as char;
            }
        }
    }
}

fn render_unknown(buffer: &mut [[[char; CELL_SIZE]; 10]; 10]) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            copy_cell(&CELL_UNKNOWN, buffer, x, y);
        }
    }
}

fn render_board_ships(game: &Game, player: Player, buffer: &mut [[[char; CELL_SIZE]; 10]; 10]) {
    let board = game.get_board(player);
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if board_get(board, x, y) {
                copy_cell(&CELL_SHIP, buffer, x, y);
            }
        }
    }
}

fn render_board_hits(game: &Game, player: Player, buffer: &mut [[[char; CELL_SIZE]; 10]; 10]) {
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
    let stdout = std::io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());

    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            for i in 0..CELL_SIZE {
                stdout.write(&[buffer[y][x][i] as u8]);
            }
        }
        stdout.write(b"\n");
    }
}

#[cfg(test)]
mod tests {
    use std::{io, process::Command, thread::sleep, time::Duration};

    use crate::board_api::{move_ship, Direction};

    use super::*;

    #[test]
    fn test_display_alpha_board() {
        let mut game = Game::default();
        let mut buffer = [[[0 as char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE];
        for i in 1..30 {
            println!("Step: {}", i);
            let mut ship = create_ship(4);
            let step = i;
            match move_ship(ship, step, Direction::Right) {
                Err(_) => {}
                Ok(new_ship) => {
                    ship = new_ship;
                    game.add_ship(Player::Alpha, ship, 0);
                    clear_buffer(&mut buffer);
                    render_unknown(&mut buffer);
                    render_board_ships(&game, Player::Alpha, &mut buffer);
                    display_board(&buffer);
                }
            }
            sleep(Duration::from_secs(1));
            Command::new("clear").status();

            // print!("{}[2J{}[1;1H", 27 as char, 27 as char);
            // io::stdout().flush().unwrap();
        }
    }
}
