#![allow(dead_code)]
#![allow(clippy::unusual_byte_groupings)]

pub const CAP: usize = u128::BITS as usize;
pub const BOARD_SIZE: usize = 10;

pub const BOARD_COUNT: usize = BOARD_SIZE * BOARD_SIZE;
pub const GAP: usize = CAP - BOARD_COUNT;

pub const CELL: u128 = (u128::MAX >> 1) + 1;
pub const LINE: u128 = ((1 << BOARD_SIZE) - 1) << (CAP - BOARD_SIZE);
pub const BOARD_MASK: u128 = ((1 << BOARD_COUNT) - 1) << (CAP - BOARD_COUNT);
pub const SQUARE: u128 = create_surround_mask(move_board(
    move_board(CELL, 1, Direction::Right),
    1,
    Direction::Down,
));

pub const TOP_BORDER_MASK: u128 = LINE;
pub const BOT_BORDER_MASK: u128 = move_board(TOP_BORDER_MASK, BOARD_SIZE - 1, Direction::Down);
pub const LEF_BORDER_MASK: u128 = flip(TOP_BORDER_MASK);
pub const RGT_BORDER_MASK: u128 = LEF_BORDER_MASK >> (BOARD_SIZE - 1);

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub enum Orientation {
    Horizontal,
    Vertical,
}

#[inline(always)]
const fn get_from_board(board: u128, x: usize, y: usize) -> bool {
    debug_assert!(x < BOARD_SIZE);
    debug_assert!(y < BOARD_SIZE);
    debug_assert!(board & !BOARD_MASK == 0);

    let y_row = board >> (BOARD_SIZE * (BOARD_SIZE - y - 1) + GAP);
    (y_row >> (BOARD_SIZE - x - 1)) & 1 == 1
}

#[inline(always)]
const fn set_to_board(board: u128, x: usize, y: usize, value: bool) -> u128 {
    debug_assert!(x < BOARD_SIZE);
    debug_assert!(y < BOARD_SIZE);
    debug_assert!(board & !BOARD_MASK == 0);

    let v = 1 << (BOARD_SIZE - x - 1);
    let v = v << (BOARD_SIZE * (BOARD_SIZE - y - 1) + GAP);
    if value {
        board | v
    } else {
        board & !v
    }
}
/*
 u128 f(u128 a, u128 b, char value) {
   u128 result;
   asm(
       "test %[value], %[value]\n\t"
       "cmovne %[result_or], %[result_not]\n\t"
       : [result_or] "=r" (result)
       : "0" (a | b), [result_not] "r" (a & ~b), [value] "r" (value)
       : "cc"
   );
   return result;
}
*/

/// Creates a horizontal ship of the given size.
#[inline(always)]
pub const fn create_ship(mut size: usize) -> u128 {
    assert!(size <= 4, "Ship size cannot be greater than 4!");
    let mut result = CELL;
    while size > 1 {
        result = (result >> 1) | CELL;
        size -= 1;
    }
    result
}

pub const fn add_ship(
    board: u128,
    x: usize,
    y: usize,
    size: usize,
    orientation: Orientation,
) -> Result<u128, (u128, u128)> {
    let ship = create_ship(size);
    let ship = match orientation {
        Orientation::Horizontal => ship,
        Orientation::Vertical => flip(ship),
    };
    let ship = move_board(ship, x, Direction::Right);
    let ship = move_board(ship, y, Direction::Down);
    let mask = create_surround_mask(ship);

    if mask & board != 0 {
        return Err((board | ship, mask));
    }

    Ok(board | ship)
}

#[inline(always)]
pub const fn create_surround_mask(item: u128) -> u128 {
    use Direction::*;
    let mask = item | move_board(item, 1, Right) | move_board(item, 1, Left);
    mask | move_board(mask, 1, Up) | move_board(mask, 1, Down)
}

#[inline(always)]
pub const fn move_board(board: u128, step: usize, direction: Direction) -> u128 {
    let shift = match direction {
        Direction::Up => BOARD_SIZE * step,
        Direction::Down => BOARD_SIZE * step,
        Direction::Left => step,
        Direction::Right => step,
    };
    match direction {
        Direction::Up | Direction::Left => board << shift,
        Direction::Down | Direction::Right => board >> shift,
    }
}

#[inline(always)]
pub const fn move_ship(ship: u128, step: usize, direction: Direction) -> Result<u128, u128> {
    let mask = match direction {
        Direction::Up => TOP_BORDER_MASK,
        Direction::Down => BOT_BORDER_MASK,
        Direction::Left => LEF_BORDER_MASK,
        Direction::Right => RGT_BORDER_MASK,
    };
    if ship & mask != 0 {
        return Err(move_board(ship, step, direction));
    }
    Ok(move_board(ship, step, direction))
}

#[inline(always)]
pub const fn flip(input: u128) -> u128 {
    debug_assert!(input & !BOARD_MASK == 0);
    let mut result = input;
    let mut i = 1;

    while i < BOARD_SIZE {
        let mut j = 0;
        while j < i {
            let a = get_from_board(input, i, j);
            let b = get_from_board(input, j, i);

            result = set_to_board(result, i, j, b);
            result = set_to_board(result, j, i, a);

            j += 1;
        }
        i += 1;
    }
    result
}

mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn get_set_board() {
        assert_eq!(get_from_board(1u128 << 127, 0, 0) as usize, 1);
        assert_eq!(get_from_board(1u128 << 126, 1, 0) as usize, 1);
        assert_eq!(get_from_board(1u128 << 126, 0, 1) as usize, 0);
        assert_eq!(set_to_board(0, 0, 0, true), 1 << 127);
        assert_eq!(set_to_board(0, 1, 0, true), 1 << 126);
        assert_eq!(set_to_board(1 << 127, 0, 0, false), 0);
        assert_eq!(set_to_board(1 << 126, 1, 0, false), 0);
        assert_eq!(set_to_board(1 << 127, 1, 0, false), 1 << 127);
    }

    #[test]
    fn right_duality() {
        assert_eq!(RGT_BORDER_MASK, flip(BOT_BORDER_MASK));
    }

    #[test]
    fn left_border_mask() {
        let l: u128 = 0b1000000000100000000010000000001000000000100000000010000000001000000000100000000010000000001000000000 << GAP;
        assert_eq!(LEF_BORDER_MASK, l);
    }

    #[test]
    fn flip_flip_is_id() {
        let orig: u128 = 1 << 120 | 1 << 121 | 1 << 122 | 1 << 123 | 1 << 124;
        assert_eq!(flip(flip(orig)), orig);
    }

    #[test]
    fn flip_1x1() {
        assert_eq!(flip(1u128 << 127), 1u128 << 127);
    }

    #[test]
    #[should_panic]
    fn flip_out_of_board() {
        assert_eq!(flip(1), 1);
    }

    #[test]
    fn one_ship() {
        assert_eq!(create_ship(1), 0b1u128 << 127);
    }

    #[test]
    fn horizontal_3_ship() {
        assert_eq!(create_ship(3), 0b111u128 << 125);
    }

    #[test]
    fn move_2_down() {
        assert_eq!(
            move_ship(
                0b00001_00000__00000_00000__00000_00000 << GAP,
                2,
                Direction::Down
            ),
            Ok(0b00000_00000_00000_00000_00001_00000 << GAP)
        );
    }

    #[test]
    fn move_2_left() {
        assert_eq!(
            move_ship(0b00001_00000, 2, Direction::Left),
            Ok(0b00001_00000 << 2)
        );
    }
}
