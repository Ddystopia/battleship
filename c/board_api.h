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
u128 move_board(u128 board, usize step, Direction direction);
u128 transpose(u128 input);

#endif
