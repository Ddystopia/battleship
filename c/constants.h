#ifndef CONSTANTS_H
#define CONSTANTS_H

#include <stdbool.h>
#include <stddef.h>

typedef __uint128_t u128;
typedef size_t usize;

#define CAP ((usize)128)
#define BOARD_SIZE ((usize)10)

#define BOARD_COUNT ((usize)BOARD_SIZE * BOARD_SIZE)
#define GAP (CAP - BOARD_COUNT)

#define ONE ((u128)1)
#define CELL (ONE << (CAP - 1))
#define LINE (((ONE << BOARD_SIZE) - 1) << (CAP - BOARD_SIZE))
#define BOARD_MASK (((ONE << BOARD_COUNT) - 1) << (CAP - BOARD_COUNT))

#define BIGGEST_SHIP_SIZE (5)

#endif
