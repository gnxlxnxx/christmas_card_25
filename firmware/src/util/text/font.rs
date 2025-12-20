// TODO: 9 height font idea:
// 00-7f: fill col from row 0 (top)
// 80:    mark end of character
// 81-ff: fill col from row 2

const ASCII_PRINTABLE: [Letter; 95] = [
    // SPACE
    Letter([
        0b0000000,
        Letter::END_MARKER,
        0, 0, 0
    ]),
    // EXCL
    Letter([
        0b1011111,
        Letter::END_MARKER,
        0, 0, 0
    ]),
    // "
    Letter([
        0b0000011,
        0b0000000,
        0b0000011,
        Letter::END_MARKER,
        0
    ]),
    // #
    Letter([
        0b0010100,
        0b1111111,
        0b0010100,
        0b1111111,
        0b0010100,
    ]),
    // $
    Letter([
        0b0100110,
        0b1001001,
        0b1111111,
        0b1001001,
        0b0110010,
    ]),
    // %
    Letter([
        0b0100010,
        0b0010000,
        0b0001000,
        0b0000100,
        0b0100010,
    ]),
    // &
    Letter([
        0b0110110,
        0b1001001,
        0b1010001,
        0b0100010,
        0b1010000,
    ]),
    // '
    Letter([
        0b0000011,
        Letter::END_MARKER,
        0, 0, 0
    ]),
    // (
    Letter([
        0b0111110,
        0b1000001,
        Letter::END_MARKER,
        0, 0
    ]),
    // )
    Letter([
        0b1000001,
        0b0111110,
        Letter::END_MARKER,
        0, 0
    ]),
    // *
    Letter([
        0b0101010,
        0b0011100,
        0b1111111,
        0b0011100,
        0b0101010,
    ]),
    // +
    Letter([
        0b0001000,
        0b0001000,
        0b0111110,
        0b0001000,
        0b0001000,
    ]),
    // ,
    Letter([
        0b1100000,
        Letter::END_MARKER,
        0, 0, 0
    ]),
    // -
    Letter([
        0b0001000,
        0b0001000,
        0b0001000,
        Letter::END_MARKER,
        0
    ]),
    // .
    Letter([
        0b1000000,
        Letter::END_MARKER,
        0, 0, 0
    ]),
    // /
    Letter([
        0b1100000,
        0b0011100,
        0b0000011,
        Letter::END_MARKER,
        0
    ]),
    // 0
    Letter([
        0b0111110,
        0b1010001,
        0b1001001,
        0b1000101,
        0b0111110,
    ]),
    // 1
    Letter([
        0b0000010,
        0b1111111,
        Letter::END_MARKER,
        0, 0
    ]),
    // 2
    Letter([
        0b1000010,
        0b1100001,
        0b1010001,
        0b1001001,
        0b1000110,
    ]),
    // 3
    Letter([
        0b1000001,
        0b1001001,
        0b1001001,
        0b0110110,
        Letter::END_MARKER
    ]),
    // 4
    Letter([
        0b0000111,
        0b0001000,
        0b0001000,
        0b1111111,
        Letter::END_MARKER
    ]),
    // 5
    Letter([
        0b1001111,
        0b1001001,
        0b1001001,
        0b0110001,
        Letter::END_MARKER
    ]),
    // 6
    Letter([
        0b0111110,
        0b1001001,
        0b1001001,
        0b0110010,
        Letter::END_MARKER
    ]),
    // 7
    Letter([
        0b0000001,
        0b1100001,
        0b0011001,
        0b0000111,
        Letter::END_MARKER
    ]),
    // 8
    Letter([
        0b0110110,
        0b1001001,
        0b1001001,
        0b0110110,
        Letter::END_MARKER
    ]),
    // 9
    Letter([
        0b0100110,
        0b1001001,
        0b1001001,
        0b0111110,
        Letter::END_MARKER
    ]),
    // :
    Letter([
        0b1000100,
        Letter::END_MARKER,
        0, 0, 0
    ]),
    // ;
    Letter([
        0b1100100,
        Letter::END_MARKER,
        0, 0, 0
    ]),
    // <
    Letter([
        0b0001000,
        0b0010100,
        0b0100010,
        Letter::END_MARKER,
        0
    ]),
    // =
    Letter([
        0b0010100,
        0b0010100,
        0b0010100,
        Letter::END_MARKER,
        0
    ]),
    // >
    Letter([
        0b0100010,
        0b0010100,
        0b0001000,
        Letter::END_MARKER,
        0
    ]),
    // ?
    Letter([
        0b0000010,
        0b0000001,
        0b1011001,
        0b0001001,
        0b0000110,
    ]),
    // @
    Letter([
        0b0111110,
        0b1000001,
        0b0011001,
        0b0100101,
        0b0111110,
    ]),
    // A
    Letter([
        0b1111110,
        0b0001001,
        0b0001001,
        0b1111110,
        Letter::END_MARKER
    ]),
    // B
    Letter([
        0b1111111,
        0b1001001,
        0b1001001,
        0b0110110,
        Letter::END_MARKER
    ]),
    // C
    Letter([
        0b0111110,
        0b1000001,
        0b1000001,
        0b0100010,
        Letter::END_MARKER
    ]),
    // D
    Letter([
        0b1111111,
        0b1000001,
        0b1000001,
        0b0111110,
        Letter::END_MARKER
    ]),
    // E
    Letter([
        0b1111111,
        0b1001001,
        0b1000001,
        Letter::END_MARKER,
        0
    ]),
    // F
    Letter([
        0b1111111,
        0b0001001,
        0b0000001,
        Letter::END_MARKER,
        0
    ]),
    // G
    Letter([
        0b0111110,
        0b1000001,
        0b1001001,
        0b0111010,
        Letter::END_MARKER
    ]),
    // H
    Letter([
        0b1111111,
        0b0001000,
        0b0001000,
        0b1111111,
        Letter::END_MARKER
    ]),
    // I
    Letter([
        0b1000001,
        0b1111111,
        0b1000001,
        Letter::END_MARKER,
        0
    ]),
    // J
    Letter([
        0b0110000,
        0b1000001,
        0b1000001,
        0b0111111,
        Letter::END_MARKER
    ]),
    // K
    Letter([
        0b1111111,
        0b0001000,
        0b0010100,
        0b0100010,
        0b1000001,
    ]),
    // L
    Letter([
        0b1111111,
        0b1000000,
        0b1000000,
        0b1000000,
        Letter::END_MARKER
    ]),
    // M
    Letter([
        0b1111111,
        0b0000010,
        0b0000100,
        0b0000010,
        0b1111111,
    ]),
    // N
    Letter([
        0b1111111,
        0b0000100,
        0b0001000,
        0b0010000,
        0b1111111,
    ]),
    // O
    Letter([
        0b0111110,
        0b1000001,
        0b1000001,
        0b0111110,
        Letter::END_MARKER
    ]),
    // P
    Letter([
        0b1111111,
        0b0001001,
        0b0001001,
        0b0000110,
        Letter::END_MARKER
    ]),
    // Q
    Letter([
        0b0111110,
        0b1000001,
        0b0100001,
        0b1011110,
        Letter::END_MARKER
    ]),
    // R
    Letter([
        0b1111111,
        0b0001001,
        0b0001001,
        0b1110110,
        Letter::END_MARKER
    ]),
    // S
    Letter([
        0b1000110,
        0b1001001,
        0b1001001,
        0b0110001,
        Letter::END_MARKER
    ]),
    // T
    Letter([
        0b0000001,
        0b0000001,
        0b1111111,
        0b0000001,
        0b0000001,
    ]),
    // U
    Letter([
        0b0111111,
        0b1000000,
        0b1000000,
        0b0111111,
        Letter::END_MARKER
    ]),
    // V
    Letter([
        0b0000111,
        0b0111000,
        0b1000000,
        0b0111000,
        0b0000111,
    ]),
    // W
    Letter([
        0b1111111,
        0b0100000,
        0b0011100,
        0b0100000,
        0b1111111,
    ]),
    // X
    Letter([
        0b1100011,
        0b0010100,
        0b0001000,
        0b0010100,
        0b1100011,
    ]),
    // Y
    Letter([
        0b0000011,
        0b0000100,
        0b1111000,
        0b0000100,
        0b0000011,
    ]),
    // Z
    Letter([
        0b1100001,
        0b1010001,
        0b1001001,
        0b1000101,
        0b1000011,
    ]),
    // [
    Letter([
        0b1111111,
        0b1000001,
        Letter::END_MARKER,
        0, 0
    ]),
    // '\'
    Letter([
        0b0000011,
        0b0011100,
        0b1100000,
        Letter::END_MARKER,
        0
    ]),
    // ]
    Letter([
        0b1000001,
        0b1111111,
        Letter::END_MARKER,
        0, 0
    ]),
    // ^
    Letter([
        0b0000010,
        0b0000001,
        0b0000010,
        Letter::END_MARKER,
        0
    ]),
    // _
    Letter([
        0b1000000,
        0b1000000,
        0b1000000,
        0b1000000,
        Letter::END_MARKER
    ]),
    // `
    Letter([
        0b0000001,
        0b0000010,
        Letter::END_MARKER,
        0, 0
    ]),
    // a
    Letter([
        0b0100000,
        0b1010100,
        0b1010100,
        0b1111000,
        Letter::END_MARKER
    ]),
    // b
    Letter([
        0b1111111,
        0b1000100,
        0b1000100,
        0b0111000,
        Letter::END_MARKER
    ]),
    // c
    Letter([
        0b0111000,
        0b1000100,
        0b1000100,
        Letter::END_MARKER,
        0
    ]),
    // d
    Letter([
        0b0111000,
        0b1000100,
        0b1000100,
        0b1111111,
        Letter::END_MARKER
    ]),
    // e
    Letter([
        0b0111000,
        0b1010100,
        0b1010100,
        0b0011000,
        Letter::END_MARKER
    ]),
    // f
    Letter([
        0b1111110,
        0b0000101,
        0b0000001,
        Letter::END_MARKER,
        0
    ]),
    // g
    Letter([
        0b0101100,
        0b1010010,
        0b1010010,
        0b0111100,
        Letter::END_MARKER
    ]),
    // h
    Letter([
        0b1111111,
        0b0000100,
        0b0000100,
        0b1111000,
        Letter::END_MARKER
    ]),
    // i
    Letter([
        0b1111101,
        Letter::END_MARKER,
        0, 0, 0
    ]),
    // j
    Letter([
        0b0100000,
        0b1000000,
        0b0111101,
        Letter::END_MARKER,
        0
    ]),
    // k
    Letter([
        0b1111111,
        0b0010000,
        0b0101000,
        0b1000100,
        Letter::END_MARKER
    ]),
    // l
    Letter([
        0b0111111,
        0b1000000,
        Letter::END_MARKER,
        0, 0
    ]),
    // m
    Letter([
        0b1111000,
        0b0000100,
        0b1111000,
        0b0000100,
        0b1111000,
    ]),
    // n
    Letter([
        0b1111000,
        0b0000100,
        0b0000100,
        0b1111000,
        Letter::END_MARKER
    ]),
    // o
    Letter([
        0b0111000,
        0b1000100,
        0b1000100,
        0b0111000,
        Letter::END_MARKER
    ]),
    // p
    Letter([
        0b1111110,
        0b0010010,
        0b0010010,
        0b0001100,
        Letter::END_MARKER
    ]),
    // q
    Letter([
        0b0001110,
        0b0010010,
        0b0010010,
        0b1111100,
        Letter::END_MARKER
    ]),
    // r
    Letter([
        0b1111000,
        0b0000100,
        0b0000100,
        Letter::END_MARKER,
        0
    ]),
    // s
    Letter([
        0b1001000,
        0b1010100,
        0b1010100,
        0b0100100,
        Letter::END_MARKER
    ]),
    // t
    Letter([
        0b0000100,
        0b0111111,
        0b1000100,
        0b0100000,
        Letter::END_MARKER
    ]),
    // u
    Letter([
        0b0111100,
        0b1000000,
        0b1000000,
        0b1111100,
        Letter::END_MARKER
    ]),
    // v
    Letter([
        0b0001100,
        0b0110000,
        0b1000000,
        0b0110000,
        0b0001100,
    ]),
    // w
    Letter([
        0b0111100,
        0b1000000,
        0b0111100,
        0b1000000,
        0b0111100,
    ]),
    // x
    Letter([
        0b1000100,
        0b0101000,
        0b0010000,
        0b0101000,
        0b1000100,
    ]),
    // y
    Letter([
        0b0001110,
        0b1010000,
        0b1010000,
        0b0111110,
        Letter::END_MARKER
    ]),
    // z
    Letter([
        0b1000100,
        0b1100100,
        0b1010100,
        0b1001100,
        0b1000100,
    ]),
    // {
    Letter([
        0b0001000,
        0b0110110,
        0b1000001,
        Letter::END_MARKER,
        0
    ]),
    // |
    Letter([
        0b1110111,
        Letter::END_MARKER,
        0, 0, 0
    ]),
    // }
    Letter([
        0b1000001,
        0b0110110,
        0b0001000,
        Letter::END_MARKER,
        0
    ]),
    // ~
    Letter([
        0b0001000,
        0b0000100,
        0b0001000,
        0b0010000,
        0b0001000,
    ]),
];

const LETTER_UNKNOWN: Letter = Letter([
    0b1111111,
    0b1000001,
    0b1000001,
    0b1000001,
    0b1111111,
]);

const LETTER_UE: Letter = Letter([
    0b0111101,
    0b1000000,
    0b1000000,
    0b1111101,
    Letter::END_MARKER
]);

const LETTER_OE: Letter = Letter([
    0b0111001,
    0b1000100,
    0b1000100,
    0b0111001,
    Letter::END_MARKER
]);

const LETTER_AE: Letter = Letter([
    0b0100001,
    0b1010100,
    0b1010100,
    0b1111001,
    Letter::END_MARKER
]);

const LETTER_SZ: Letter = Letter([
    0b1111110,
    0b0000001,
    0b0000101,
    0b1001010,
    0b0110000,
]);

const LETTER_UNI_STUTTGART: Letter = Letter([
    0b0001000,
    0b0100010,
    0b0001000,
    0b0100010,
    0b0001000,
]);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Letter(pub [u8; Self::MAX_WIDTH]);

impl Letter {
    pub const MAX_WIDTH: usize = 5;
    pub const END_MARKER: u8 = 0x80;

    pub const fn get(c: u8) -> &'static Letter {
        match c {
            c if b' ' <= c && c <= b'~' => &ASCII_PRINTABLE[(c - b' ') as usize],
            b'\xdc'|b'\xfc' => &LETTER_UE,
            b'\xd6'|b'\xf6' => &LETTER_OE,
            b'\xc4'|b'\xe4' => &LETTER_AE,
            b'\x80' => &LETTER_UNI_STUTTGART,
            _ => &LETTER_UNKNOWN
        }
    }

    pub const fn downshift(row: u8) -> Option<usize> {
        if row < Self::END_MARKER {
            Some(0)
        } else if row > Self::END_MARKER {
            Some(2)
        } else {
        None
        }
    }
}
