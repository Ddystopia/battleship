#ifndef BOARD_API_H
#define BOARD_API_H

#include "constants.h"

typedef enum Result { Ok, Err } Result;

typedef enum Direction {
    Up,
    Down,
    Left,
    Right,
} Direction;

typedef enum Orientation {
    Horizontal,
    Vertical,
} Orientation;

u128 board_get(u128 board, usize x, usize y);
u128 board_set(u128 board, usize x, usize y, bool value);
u128 create_ship(usize size);
u128 create_surround_mask(u128 item);
u128 wrapping_move(u128 board, usize step, Direction direction);
u128 saturated_move(u128 ship, Direction direction);
u128 cutting_move(u128 ship, Direction direction);
u128 transpose(u128 input);

#endif
