use std::io::Read;
use std::io::{self, Write};
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW, VMIN, VTIME};

use crate::board_api::{saturated_move, Direction, wrapping_move, ship_size};
use crate::constants::CELL;
use crate::{
    board_api::{board_get, create_ship, create_surround_mask, transpose, Orientation},
    constants::BOARD_SIZE,
    game::{Game, Player, SHIPS_COUNT},
};

// Base part of fiels. Represents something like [ ], [*], [~], [O]
pub type Cell = [char; CELL_SIZE];
pub type OutputBuffer = [[Cell; BOARD_SIZE]; BOARD_SIZE];
pub const CELL_SIZE: usize = 12; // color identifier (in rust: \u{001B}, in C: \033) + [ + color (2) + m + cell + color identifier (in rust: \u{001B}, in C: \033) + [ + 0 + m
pub const SHIP_SIZES: [usize; 5] = [5, 4, 3, 3, 2];

const CELL_MISS: Cell = create_cell("\u{001B}[00m   \u{001B}[0m");
const CELL_HIT: Cell = create_cell("\u{001B}[31m[*]\u{001B}[0m");
const CELL_UNKNOWN: Cell = create_cell("\u{001B}[34m[~]\u{001B}[0m");
const CELL_SHIP: Cell = create_cell("\u{001B}[32m[O]\u{001B}[0m");
const CELL_COLLISION: Cell = create_cell("\u{001B}[33m[X]\u{001B}[0m");
const CELL_NEW_SHIP: Cell = create_cell("\u{001B}[32m[n]\u{001B}[0m");
const CELL_CROSSHAIR: Cell = create_cell("\u{001B}[33m{+}\u{001B}[0m");

// board -- 10x10 array of cells
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

fn render_border_shoots(shoots: u128, buffer: &mut OutputBuffer) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if board_get(shoots, x, y) {
                copy_cell(&CELL_MISS, buffer, x, y);
            }
        }
    }
}

fn render_board_hits(hits: u128, buffer: &mut OutputBuffer) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if board_get(hits, x, y) {
                copy_cell(&CELL_HIT, buffer, x, y);
            }
        }
    }
}

fn render_crosshair(buffer: &mut OutputBuffer, crosshair: u128) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if board_get(crosshair, x, y) {
                copy_cell(&CELL_CROSSHAIR, buffer, x, y);
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

fn display_two_boards(
    lbuffer: &OutputBuffer,
    rbuffer: &OutputBuffer
) {
    let mut stdout = io::stdout();

    // Allocate the temporary buffer on the stack
    let mut temp_buffer = [0u8; 2 * BOARD_SIZE * CELL_SIZE + 1 + 2]; // +1 for the newline character. +2 tab

    for y in 0..BOARD_SIZE {
        let mut idx = 0;
        for x in 0..BOARD_SIZE {
            for i in 0..CELL_SIZE {
                // temp_buffer[idx++] = buffer[y][x][i];
                let c = lbuffer[y][x][i];
                let c_len = c.len_utf8();
                c.encode_utf8(&mut temp_buffer[idx..idx + c_len]);
                idx += c_len;
            }
        }

        // Add a tab characters at the end of the line
        temp_buffer[idx] = b'\t';
        idx += 1;
        temp_buffer[idx] = b'\t';
        idx += 1;
        

        for x in 0..BOARD_SIZE {
            for i in 0..CELL_SIZE {
                // temp_buffer[idx++] = buffer[y][x][i];
                let c = rbuffer[y][x][i];
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

fn display_players_ships_status(game: &Game) {
    let mut stdout = io::stdout();

    // Allocate the temporary buffer on the stack
    let mut temp_buffer = [0u8; 2 * BOARD_SIZE * CELL_SIZE + 1 + 2]; // +1 for the newline character. +2 tab
    
    // Display ships under board, line by line
    for i in 0..SHIPS_COUNT {
        let mut idx = 0;
        let alpha_ship = game.ships_alpha[i];
        let alpha_ship_size = ship_size(alpha_ship);
        let alpha_ship_damage = ship_size(alpha_ship & game.shoots_beta);
        let alpha_ship_undamage = alpha_ship_size - alpha_ship_damage;
        let beta_ship = game.ships_beta[i];
        let beta_ship_size = ship_size(beta_ship);
        let beta_ship_damage = ship_size(beta_ship & game.shoots_alpha);
        let beta_ship_undamage = beta_ship_size - beta_ship_damage;

        // Display alpha side
        for _ in 0..alpha_ship_undamage {
            for i in 0..CELL_SIZE {
                // temp_buffer[idx++] = buffer[y][x][i];
                let c = CELL_SHIP[i];
                let c_len = c.len_utf8();
                c.encode_utf8(&mut temp_buffer[idx..idx + c_len]);
                idx += c_len;
            }
        }

        for _ in 0..alpha_ship_damage {
            for i in 0..CELL_SIZE {
                // temp_buffer[idx++] = buffer[y][x][i];
                let c = CELL_HIT[i];
                let c_len = c.len_utf8();
                c.encode_utf8(&mut temp_buffer[idx..idx + c_len]);
                idx += c_len;
            }
        }

        for _ in 0..(BOARD_SIZE-alpha_ship_size) {
            for i in 0..CELL_SIZE {
                // temp_buffer[idx++] = buffer[y][x][i];
                let c = CELL_MISS[i];
                let c_len = c.len_utf8();
                c.encode_utf8(&mut temp_buffer[idx..idx + c_len]);
                idx += c_len;
            }
        }

        // Add a tab characters at the end of the line
        temp_buffer[idx] = b'\t';
        idx += 1;
        temp_buffer[idx] = b'\t';
        idx += 1;

        // Display beta side
        for _ in 0..beta_ship_undamage {
            for i in 0..CELL_SIZE {
                // temp_buffer[idx++] = buffer[y][x][i];
                let c = CELL_SHIP[i];
                let c_len = c.len_utf8();
                c.encode_utf8(&mut temp_buffer[idx..idx + c_len]);
                idx += c_len;
            }
        }

        for _ in 0..beta_ship_damage {
            for i in 0..CELL_SIZE {
                // temp_buffer[idx++] = buffer[y][x][i];
                let c = CELL_HIT[i];
                let c_len = c.len_utf8();
                c.encode_utf8(&mut temp_buffer[idx..idx + c_len]);
                idx += c_len;
            }
        }

        for _ in 0..(BOARD_SIZE-beta_ship_size) {
            for i in 0..CELL_SIZE {
                // temp_buffer[idx++] = buffer[y][x][i];
                let c = CELL_MISS[i];
                let c_len = c.len_utf8();
                c.encode_utf8(&mut temp_buffer[idx..idx + c_len]);
                idx += c_len;
            }
        }
        
        temp_buffer[idx] = b'\n';
        idx += 1;

        // Print the line
        stdout.write_all(&temp_buffer[..idx]).unwrap();
    }
    
    stdout.flush().unwrap();
}

fn render_current_player_board(
    game: &Game,
    buffer: &mut OutputBuffer,
    player: Player,
) {
    let board = game.get_board(player);
    let other_shoots = game.get_shoots(player.other());
    let hits: u128 = other_shoots & board;

    render_unknown(buffer);
    render_board_ships(board, buffer);
    render_border_shoots(other_shoots, buffer);
    render_board_hits(hits, buffer);
}

fn render_enamy_player_board(
    game: &Game,
    buffer: &mut OutputBuffer,
    player: Player,
) {
    let other_player_board = game.get_board(player.other());
    let shoots = game.get_shoots(player);
    let hits: u128 = shoots & other_player_board;

    clear_buffer(buffer);
    render_unknown(buffer);
    render_border_shoots(shoots, buffer);
    render_board_hits(hits, buffer);
}

pub fn display_scene_after_shoot(
    game: &Game,
    lbuffer: &mut OutputBuffer,
    rbuffer: &mut OutputBuffer,
    player: Player
) {
    clear();
    clear_buffer(lbuffer);
    render_unknown(lbuffer);
    clear_buffer(rbuffer);
    render_unknown(rbuffer);
    
    if player == Player::Alpha {
        render_current_player_board(game, lbuffer, Player::Alpha);
        render_enamy_player_board(game, rbuffer, Player::Alpha);
    } else {
        render_current_player_board(game, rbuffer, Player::Beta);
        render_enamy_player_board(game, lbuffer, Player::Beta);
    }

    display_two_boards(lbuffer, rbuffer);
    println!();
    display_players_ships_status(game);
    println!();

    wait_for_enter("");
}

pub fn display_last_scene(
    game: &Game,
    lbuffer: &mut OutputBuffer,
    rbuffer: &mut OutputBuffer,
) {
    clear();
    clear_buffer(lbuffer);
    render_unknown(lbuffer);
    clear_buffer(rbuffer);
    render_unknown(rbuffer);
    render_current_player_board(game, lbuffer, Player::Alpha);
    render_current_player_board(game, rbuffer, Player::Beta);
    display_two_boards(lbuffer, rbuffer);
    println!();
    display_players_ships_status(game);
    println!();
    
    if game.get_winner() == 0 {
        wait_for_enter("Player Alpha wins!")
    } else if game.get_winner() == 1 {
        wait_for_enter("Player Beta wins!")
    } else {
        panic!("Invalid winner")
    }
}

pub fn read_shoot(
    game: &Game,
    alpha_buffer: &mut OutputBuffer,
    beta_buffer: &mut OutputBuffer,
    player: Player,
) -> u128 {
    let mut crosshair: u128 = CELL;

    clear_buffer(alpha_buffer);
    render_unknown(alpha_buffer);
    clear_buffer(beta_buffer);
    render_unknown(beta_buffer);

    if player == Player::Alpha {
        render_current_player_board(game, alpha_buffer, Player::Alpha);
    } else {
        render_current_player_board(game, beta_buffer, Player::Beta);
    }
    
    loop {
        clear();
        if player == Player::Alpha {
            render_enamy_player_board(game, beta_buffer, Player::Alpha);
            render_crosshair(beta_buffer, crosshair);
        } else {
            render_enamy_player_board(game, alpha_buffer, Player::Beta);
            render_crosshair(alpha_buffer, crosshair);
        }
        display_two_boards(alpha_buffer, beta_buffer);
        println!();
        display_players_ships_status(game);
        if player == Player::Alpha {
            clear_buffer(beta_buffer);
        } else {
            clear_buffer(alpha_buffer);
        }

        let input = getchar();


        if input == '\n' {
            break;
        }

        crosshair = move_by_user_input(crosshair, input);
    }

    crosshair    
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
        clear();
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

    #[test]
    fn test_display_alpha_game_scene() {
        clear();

        let mut game = Game::default();
        let mut alpha_buffer = [[[0 as char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE];
        let mut beta_buffer = [[[0 as char; CELL_SIZE]; BOARD_SIZE]; BOARD_SIZE];

        let ships_alpha: [u128; 5] = [
            0b111110000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, 
            0b11110000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, 
            0b111000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, 
            0b11100000000000000000000000000000000000000000000000000000000000000000, 
            0b1000000000100000000000000000000000000000000000000000000000000000000000, 
        ];
        let ships_beta: [u128; 5] = [
            0b1111100000000000000000000000000000000000000000000000000000000000000000000000000000000000, 
            0b111100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, 
            0b10000000001000000000100000000000000000000000000000000000, 
            0b1000000000100000000010000000000000000000000000000000000000000000000000, 
            0b11000000000000000000000000000000, 
        ];

        for (i, ship) in ships_alpha.iter().enumerate() {
            game.add_ship(Player::Alpha, *ship, i).unwrap();
        }
        for (i, ship) in ships_beta.iter().enumerate() {
            game.add_ship(Player::Beta, *ship, i).unwrap();
        }
        while !game.is_over() {
            let shoot = read_shoot(&game, &mut alpha_buffer, &mut beta_buffer, Player::Beta);
            game.shoot(Player::Beta, shoot);
        }
        display_last_scene(&game, &mut alpha_buffer, &mut beta_buffer);

    }
}
