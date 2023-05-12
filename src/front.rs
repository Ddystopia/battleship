use std::io::Read;
use std::io::{self, Write};
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW, VMIN, VTIME};

use crate::board_api::{saturated_move, ship_size, Direction};
use crate::constants::CELL;
use crate::game::SHIPS_COUNT;
use crate::{
    board_api::{board_get, create_ship, create_surround_mask, transpose},
    constants::BOARD_SIZE,
    game::{Game, Player}
};

// Base part of fiels. Represents something like [ ], [*], [~], [O]
pub const CELL_SIZE: usize = 12; // color identifier (\u{001B}) + [ + color (2) + m + cell + color identifier (\u{001B}) + [ + 0 + m
pub type Cell = [u8; CELL_SIZE];
pub type OutputBuffer = [[Cell; BOARD_SIZE]; BOARD_SIZE];
                                 //
// cells:
//  -     -- miss
//  - [*] -- hit
//  - [~] -- unknown
//  - [O] -- your ship
//  - [X] -- collided ship
//  - [n] -- new ship
//  - {+} -- crosshair
//
// colors :
//   -     -- \033[00m   \033[0m
//   - [*] -- \033[31m[*]\033[0m
//   - [~] -- \033[34m[~]\033[0m
//   - [O] -- \033[32m[O]\033[0m
//   - [X] -- \033[33m[X]\033[0m
//   - [n] -- \033[32m[o]\033[0m
//   - {+} -- \033[32m{+}\033[0m

const CELL_MISS: Cell = create_cell("\u{001B}[00m   \u{001B}[0m");
const CELL_HIT: Cell = create_cell("\u{001B}[31m[*]\u{001B}[0m");
const CELL_UNKNOWN: Cell = create_cell("\u{001B}[34m[~]\u{001B}[0m");
const CELL_SHIP: Cell = create_cell("\u{001B}[32m[O]\u{001B}[0m");
const CELL_COLLISION: Cell = create_cell("\u{001B}[33m[X]\u{001B}[0m");
const CELL_NEW_SHIP: Cell = create_cell("\u{001B}[32m[n]\u{001B}[0m");
const CELL_CROSSHAIR: Cell = create_cell("\u{001B}[33m{+}\u{001B}[0m");

const fn create_cell(val: &str) -> Cell {
    let mut cell = [0; CELL_SIZE];
    let bytes = val.as_bytes();

    assert!(bytes.len() == CELL_SIZE);

    let mut i = 0;
    while i < bytes.len() {
        cell[i] = bytes[i];
        i += 1;
    }
    cell
}

#[derive(Default)]
pub struct IO {
    pub lbuffer: OutputBuffer,
    pub rbuffer: OutputBuffer,
}

impl IO {
    fn display_left_board(&self) {
        let buffer = self.lbuffer;
        let mut stdout = io::stdout();

        // Allocate the temporary buffer on the stack
        let mut temp_buffer = [0u8; BOARD_SIZE * CELL_SIZE + 1]; // +1 for the newline character

        for line in buffer.iter().take(BOARD_SIZE) {
            let mut idx = 0;
            for cell in line.iter().take(BOARD_SIZE) {
                temp_buffer[idx..idx + CELL_SIZE].copy_from_slice(cell);
                idx += CELL_SIZE;
            }
            // Add a newline character at the end of the line
            temp_buffer[idx] = b'\n';
            idx += 1;

            // Print the line
            stdout.write_all(&temp_buffer[..idx]).unwrap();
        }
        stdout.flush().unwrap();
    }

    fn display_two_boards(&self) {
        let lbuffer = self.lbuffer;
        let rbuffer = self.rbuffer;
        let mut stdout = io::stdout();

        // Allocate the temporary buffer on the stack
        let mut temp_buffer = [0u8; 2 * BOARD_SIZE * CELL_SIZE + 1 + 2]; // +1 for the newline character. +2 tab

        for y in 0..BOARD_SIZE {
            let mut idx = 0;
            for x in 0..BOARD_SIZE {
                temp_buffer[idx..idx + CELL_SIZE].copy_from_slice(&lbuffer[y][x]);
                idx += CELL_SIZE;
            }

            // Add a tab characters at the end of the line
            temp_buffer[idx] = b'\t';
            idx += 1;
            temp_buffer[idx] = b'\t';
            idx += 1;

            for x in 0..BOARD_SIZE {
                temp_buffer[idx..idx + CELL_SIZE].copy_from_slice(&rbuffer[y][x]);
                idx += CELL_SIZE;
            }

            // Add a newline character at the end of the line
            temp_buffer[idx] = b'\n';
            idx += 1;

            // Print the line
            stdout.write_all(&temp_buffer[..idx]).unwrap();
        }

        stdout.flush().unwrap();
    }

    fn display_players_ships_status(&self, game: &Game) {
        let mut stdout = io::stdout();

        // Allocate the temporary buffer on the stack
        let mut temp_buffer = [0; 2 * BOARD_SIZE * CELL_SIZE + 1 + 2]; // +1 for the newline character. +2 tab

        // Display ships under board, line by line
        for i in 0..SHIPS_COUNT  {
            let mut idx = 0;
            let alpha_ship = game.ships_alpha[i];
            let alpha_ship_size = ship_size(alpha_ship);
            let alpha_ship_damage = ship_size(alpha_ship & game.shoots_beta);
            let alpha_ship_undamage = alpha_ship_size - alpha_ship_damage;

            let beta_ship = game.ships_beta[i];
            let beta_ship_size = ship_size(beta_ship);
            let beta_ship_damage = ship_size(beta_ship & game.shoots_alpha);
            let beta_ship_undamage = beta_ship_size - beta_ship_damage;

            let alpha_chunks = [
                alpha_ship_undamage,
                alpha_ship_damage,
                BOARD_SIZE - alpha_ship_size,
            ];
            let beta_chunks = [
                beta_ship_undamage,
                beta_ship_damage,
                BOARD_SIZE - beta_ship_size,
            ];
            let cells = [&CELL_SHIP, &CELL_HIT, &CELL_MISS];

            for (chunk, cell) in alpha_chunks.into_iter().zip(cells.into_iter()) {
                for _ in 0..chunk {
                    temp_buffer[idx..idx + CELL_SIZE].copy_from_slice(cell);
                    idx += CELL_SIZE;
                }
            }

            // Add a tab characters at the end of the line
            temp_buffer[idx] = b'\t';
            idx += 1;
            temp_buffer[idx] = b'\t';
            idx += 1;

            for (chunk, cell) in beta_chunks.into_iter().zip(cells.into_iter()) {
                for _ in 0..chunk {
                    temp_buffer[idx..idx + CELL_SIZE].copy_from_slice(cell);
                    idx += CELL_SIZE;
                }
            }

            temp_buffer[idx] = b'\n';
            idx += 1;

            // Print the line
            stdout.write_all(&temp_buffer[..idx]).unwrap();
        }

        stdout.flush().unwrap();
    }
    pub fn display_scene_after_shoot(&mut self, game: &Game, player: Player) {
        let lbuffer = &mut self.lbuffer;
        let rbuffer = &mut self.rbuffer;
        clear();
        render_unknown(lbuffer);
        render_unknown(rbuffer);

        if player == Player::Alpha {
            render_current_player_board(lbuffer, game, Player::Alpha);
            render_enemy_player_board(rbuffer, game, Player::Alpha);
        } else {
            render_current_player_board(rbuffer, game, Player::Beta);
            render_enemy_player_board(lbuffer, game, Player::Beta);
        }

        self.display_two_boards();
        println!();
        self.display_players_ships_status(game);
        println!();

        wait_for_enter("");
    }

    pub fn display_last_scene(&mut self, game: &Game) {
        let lbuffer = &mut self.lbuffer;
        let rbuffer = &mut self.rbuffer;

        clear();
        render_unknown(lbuffer);
        render_unknown(rbuffer);
        render_current_player_board(lbuffer, game, Player::Alpha);
        render_current_player_board(rbuffer, game, Player::Beta);
        self.display_two_boards();
        println!();
        self.display_players_ships_status(game);
        println!();

        match game.get_winner() {
            Some(Player::Alpha) => wait_for_enter("Player Alpha wins!"),
            Some(Player::Beta) => wait_for_enter("Player Beta wins!"),
            _ => panic!("Invalid winner."),
        }
    }

    pub fn read_shoot(&mut self, game: &Game, player: Player) -> u128 {
        let mut crosshair: u128 = CELL;

        render_unknown(&mut self.lbuffer);
        render_unknown(&mut self.rbuffer);

        if player == Player::Alpha {
            render_current_player_board(&mut self.lbuffer, game, Player::Alpha);
        } else {
            render_current_player_board(&mut self.rbuffer, game, Player::Beta);
        }

        loop {
            clear();
            if player == Player::Alpha {
                render_enemy_player_board(&mut self.rbuffer, game, Player::Alpha);
                render(&mut self.rbuffer, crosshair, CELL_CROSSHAIR);
            } else {
                render_enemy_player_board(&mut self.lbuffer, game, Player::Beta);
                render(&mut self.lbuffer, crosshair, CELL_CROSSHAIR);
            }
            self.display_two_boards();
            println!();
            self.display_players_ships_status(game);

            let input = getchar();

            if input == '\n' {
                break;
            }

            crosshair = move_by_user_input(crosshair, input);
        }

        crosshair
    }

    pub fn read_new_ship(&mut self, game: &Game, player: Player, ship_size: usize) -> u128 {
        let mut new_ship = create_ship(ship_size);

        render_unknown(&mut self.lbuffer);

        loop {
            clear();
            let board = game.get_board(player);

            render_board_ships_n_new_ship(&mut self.lbuffer, board, new_ship);
            self.display_left_board();

            let input = getchar();

            render_unknown(&mut self.lbuffer);

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

    #[allow(dead_code)]
    pub fn print_mask(&mut self, mask: u128) {
        render_unknown(&mut self.lbuffer);
        render(&mut self.lbuffer, mask, CELL_SHIP);
        self.display_left_board();
    }
}

#[inline(always)]
fn copy_cell(cell: &Cell, buffer: &mut OutputBuffer, x: usize, y: usize) {
    buffer[y][x][..CELL_SIZE].copy_from_slice(cell)
}

fn render_unknown(buffer: &mut OutputBuffer) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            copy_cell(&CELL_UNKNOWN, buffer, x, y);
        }
    }
}

fn render(buffer: &mut OutputBuffer, mask: u128, cell: Cell) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if board_get(mask, x, y) {
                copy_cell(&cell, buffer, x, y);
            }
        }
    }
}

fn render_board_ships_n_new_ship(buffer: &mut OutputBuffer, board: u128, new_ship: u128) {
    let collision = new_ship & create_surround_mask(board);

    render(buffer, board, CELL_SHIP);
    render(buffer, new_ship, CELL_NEW_SHIP);
    render(buffer, collision, CELL_COLLISION);
}

fn render_current_player_board(buffer: &mut OutputBuffer, game: &Game, player: Player) {
    let board = game.get_board(player);
    let other_shoots = game.get_shoots(player.other());
    let hits: u128 = other_shoots & board;

    render_unknown(buffer);
    render(buffer, board, CELL_SHIP);
    render(buffer, other_shoots, CELL_MISS);
    render(buffer, hits, CELL_HIT);
}

fn render_enemy_player_board(buffer: &mut OutputBuffer, game: &Game, player: Player) {
    let other_player_board = game.get_board(player.other());
    let shoots = game.get_shoots(player);
    let hits: u128 = shoots & other_player_board;

    render_unknown(buffer);
    render(buffer, shoots, CELL_MISS);
    render(buffer, hits, CELL_HIT);
}

pub fn clear() {
    print!("{}[2J{}[1;1H", 27 as char, 27 as char);
}

pub fn wait_for_enter(text: &str) {
    println!("{}", text);
    println!("Press enter to continue...");
    while getchar() != '\n' {}
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
    match input {
        'k' | 'w' => saturated_move(board, Direction::Up),
        'j' | 's' => saturated_move(board, Direction::Down),
        'h' | 'a' => saturated_move(board, Direction::Left),
        'l' | 'd' => saturated_move(board, Direction::Right),
        _ => board,
    }
}
