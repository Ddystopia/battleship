#include <assert.h>

#include "board_api.h"
#include "constants.h"

inline u128 board_get(u128 board, usize x, usize y) {
  u128 lef = LEF_BORDER_MASK;
  assert(x < BOARD_SIZE);
  assert(y < BOARD_SIZE);
  assert((board & ~BOARD_MASK) == 0);

  u128 y_row = board >> (BOARD_SIZE * (BOARD_SIZE - y - 1) + GAP);
  return (y_row >> (BOARD_SIZE - x - 1)) & 1;
}

inline u128 board_set(u128 board, usize x, usize y, bool value) {
  assert(x < BOARD_SIZE);
  assert(y < BOARD_SIZE);
  assert((board & ~BOARD_MASK) == 0);

  u128 v = ONE << (BOARD_SIZE - x - 1);            // x
  v <<= (BOARD_SIZE * (BOARD_SIZE - y - 1) + GAP); // y
  if (value) {
    return board | v;
  } else {
    return board & ~v;
  }
}

inline u128 create_ship(usize size) {
  assert(size <= BIGGEST_SHIP_SIZE);
  return ((ONE << size) - 1) << (CAP - size);
}

inline u128 create_surround_mask(u128 item) {
  u128 mask_horizontal = item | item << 1 | item >> 1;

  u128 mask_up = mask_horizontal << BOARD_SIZE;
  u128 mask_down = mask_horizontal >> BOARD_SIZE;

  return mask_horizontal | mask_up | mask_down;
}

inline u128 move_board(u128 board, usize step, Direction direction) {
  switch (direction) {
  case Up:
    return board << BOARD_SIZE * step;
  case Down:
    return board >> BOARD_SIZE * step;
  case Left:
    return board << step;
  case Right:
    return board >> step;
  default:
    assert(false);
  }
}

inline u128 move_ship(u128 ship, Direction direction, Result *result_tag) {
  assert(result_tag != NULL);

  u128 mask = TOP_BORDER_MASK;
  if (direction == Down) {
    mask = BOT_BORDER_MASK;
  } else if (direction == Left) {
    mask = LEF_BORDER_MASK;
  } else if (direction == Right) {
    mask = RGT_BORDER_MASK;
  }
  *result_tag = (ship & mask) == 0 ? Ok : Err;
  return move_board(ship, 1, direction);
}

inline u128 transpose(u128 input) {
  u128 result = input;
  usize i = 1;

  while (i < BOARD_SIZE) {
    usize j = 0;
    while (j < i) {
      u128 a = board_get(input, i, j);
      u128 b = board_get(input, j, i);

      result = board_set(result, i, j, b);
      result = board_set(result, j, i, a);

      j++;
    }
    i++;
  }
  return result;
}
