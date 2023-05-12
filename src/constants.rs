use crate::board_api::{transpose, wrapping_move, Direction};

pub const CAP: usize = u128::BITS as usize;
pub const BOARD_SIZE: usize = 10;

pub const BOARD_COUNT: usize = BOARD_SIZE * BOARD_SIZE;
pub const GAP: usize = CAP - BOARD_COUNT;

pub const CELL: u128 = 1 << (CAP - 1);
pub const LINE: u128 = ((1 << BOARD_SIZE) - 1) << (CAP - BOARD_SIZE);
pub const BOARD_MASK: u128 = ((1 << BOARD_COUNT) - 1) << (CAP - BOARD_COUNT);

pub const TOP_BORDER_MASK: u128 = LINE;
pub const BOT_BORDER_MASK: u128 = wrapping_move(TOP_BORDER_MASK, BOARD_SIZE - 1, Direction::Down);
pub const LEF_BORDER_MASK: u128 = transpose(TOP_BORDER_MASK);
pub const RGT_BORDER_MASK: u128 = LEF_BORDER_MASK >> (BOARD_SIZE - 1);
