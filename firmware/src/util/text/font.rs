// TODO: 9 height font idea:
// 00-7f: fill col from row 0 (top)
// 80:    mark end of character
// 81-ff: fill col from row 2

const ASCII_PRINTABLE: [Letter; 95] = [
    // SPACE
    Letter([0b00000000, Letter::END_MARKER, 0, 0, 0]),
    // EXCL
    Letter([0b01011111, Letter::END_MARKER, 0, 0, 0]),
    // "
    Letter([0b00000011, 0b00000000, 0b00000011, Letter::END_MARKER, 0]),
    // #
    Letter([0b00010100, 0b01111111, 0b00010100, 0b01111111, 0b00010100]),
    // $
    Letter([0b00100110, 0b01001001, 0b01111111, 0b01001001, 0b00110010]),
    // %
    Letter([0b00100010, 0b00010000, 0b00001000, 0b00000100, 0b00100010]),
    // &
    Letter([0b00110110, 0b01001001, 0b01010001, 0b00100010, 0b01010000]),
    // '
    Letter([0b00000011, Letter::END_MARKER, 0, 0, 0]),
    // (
    Letter([0b00111110, 0b01000001, Letter::END_MARKER, 0, 0]),
    // )
    Letter([0b01000001, 0b00111110, Letter::END_MARKER, 0, 0]),
    // *
    Letter([0b00101010, 0b00011100, 0b01111111, 0b00011100, 0b00101010]),
    // +
    Letter([0b00001000, 0b00001000, 0b00111110, 0b00001000, 0b00001000]),
    // ,
    Letter([0b10110000, Letter::END_MARKER, 0, 0, 0]),
    // -
    Letter([0b00001000, 0b00001000, 0b00001000, Letter::END_MARKER, 0]),
    // .
    Letter([0b01000000, Letter::END_MARKER, 0, 0, 0]),
    // /
    Letter([0b01100000, 0b00011100, 0b00000011, Letter::END_MARKER, 0]),
    // 0
    Letter([0b00111110, 0b01010001, 0b01001001, 0b01000101, 0b00111110]),
    // 1
    Letter([0b00000010, 0b01111111, Letter::END_MARKER, 0, 0]),
    // 2
    Letter([0b01000010, 0b01100001, 0b01010001, 0b01001001, 0b01000110]),
    // 3
    Letter([
        0b01000001,
        0b01001001,
        0b01001001,
        0b00110110,
        Letter::END_MARKER,
    ]),
    // 4
    Letter([
        0b00000111,
        0b00001000,
        0b00001000,
        0b01111111,
        Letter::END_MARKER,
    ]),
    // 5
    Letter([
        0b01001111,
        0b01001001,
        0b01001001,
        0b00110001,
        Letter::END_MARKER,
    ]),
    // 6
    Letter([
        0b00111110,
        0b01001001,
        0b01001001,
        0b00110010,
        Letter::END_MARKER,
    ]),
    // 7
    Letter([
        0b00000001,
        0b01100001,
        0b00011001,
        0b00000111,
        Letter::END_MARKER,
    ]),
    // 8
    Letter([
        0b00110110,
        0b01001001,
        0b01001001,
        0b00110110,
        Letter::END_MARKER,
    ]),
    // 9
    Letter([
        0b00100110,
        0b01001001,
        0b01001001,
        0b00111110,
        Letter::END_MARKER,
    ]),
    // :
    Letter([0b01000100, Letter::END_MARKER, 0, 0, 0]),
    // ;
    Letter([0b10110001, Letter::END_MARKER, 0, 0, 0]),
    // <
    Letter([0b00001000, 0b00010100, 0b00100010, Letter::END_MARKER, 0]),
    // =
    Letter([0b00010100, 0b00010100, 0b00010100, Letter::END_MARKER, 0]),
    // >
    Letter([0b00100010, 0b00010100, 0b00001000, Letter::END_MARKER, 0]),
    // ?
    Letter([0b00000010, 0b00000001, 0b01011001, 0b00001001, 0b00000110]),
    // @
    Letter([0b00111110, 0b01000001, 0b00011001, 0b00100101, 0b00111110]),
    // A
    Letter([
        0b01111110,
        0b00001001,
        0b00001001,
        0b01111110,
        Letter::END_MARKER,
    ]),
    // B
    Letter([
        0b01111111,
        0b01001001,
        0b01001001,
        0b00110110,
        Letter::END_MARKER,
    ]),
    // C
    Letter([
        0b00111110,
        0b01000001,
        0b01000001,
        0b00100010,
        Letter::END_MARKER,
    ]),
    // D
    Letter([
        0b01111111,
        0b01000001,
        0b01000001,
        0b00111110,
        Letter::END_MARKER,
    ]),
    // E
    Letter([0b01111111, 0b01001001, 0b01000001, Letter::END_MARKER, 0]),
    // F
    Letter([0b01111111, 0b00001001, 0b00000001, Letter::END_MARKER, 0]),
    // G
    Letter([
        0b00111110,
        0b01000001,
        0b01001001,
        0b00111010,
        Letter::END_MARKER,
    ]),
    // H
    Letter([
        0b01111111,
        0b00001000,
        0b00001000,
        0b01111111,
        Letter::END_MARKER,
    ]),
    // I
    Letter([0b01000001, 0b01111111, 0b01000001, Letter::END_MARKER, 0]),
    // J
    Letter([
        0b00110000,
        0b01000001,
        0b01000001,
        0b00111111,
        Letter::END_MARKER,
    ]),
    // K
    Letter([0b01111111, 0b00001000, 0b00010100, 0b00100010, 0b01000001]),
    // L
    Letter([
        0b01111111,
        0b01000000,
        0b01000000,
        0b01000000,
        Letter::END_MARKER,
    ]),
    // M
    Letter([0b01111111, 0b00000010, 0b00000100, 0b00000010, 0b01111111]),
    // N
    Letter([0b01111111, 0b00000100, 0b00001000, 0b00010000, 0b01111111]),
    // O
    Letter([
        0b00111110,
        0b01000001,
        0b01000001,
        0b00111110,
        Letter::END_MARKER,
    ]),
    // P
    Letter([
        0b01111111,
        0b00001001,
        0b00001001,
        0b00000110,
        Letter::END_MARKER,
    ]),
    // Q
    Letter([
        0b00111110,
        0b01000001,
        0b00100001,
        0b01011110,
        Letter::END_MARKER,
    ]),
    // R
    Letter([
        0b01111111,
        0b00001001,
        0b00001001,
        0b01110110,
        Letter::END_MARKER,
    ]),
    // S
    Letter([
        0b01000110,
        0b01001001,
        0b01001001,
        0b00110001,
        Letter::END_MARKER,
    ]),
    // T
    Letter([0b00000001, 0b00000001, 0b01111111, 0b00000001, 0b00000001]),
    // U
    Letter([
        0b00111111,
        0b01000000,
        0b01000000,
        0b00111111,
        Letter::END_MARKER,
    ]),
    // V
    Letter([0b00000111, 0b00111000, 0b01000000, 0b00111000, 0b00000111]),
    // W
    Letter([0b01111111, 0b00100000, 0b00011100, 0b00100000, 0b01111111]),
    // X
    Letter([0b01100011, 0b00010100, 0b00001000, 0b00010100, 0b01100011]),
    // Y
    Letter([0b00000011, 0b00000100, 0b01111000, 0b00000100, 0b00000011]),
    // Z
    Letter([0b01100001, 0b01010001, 0b01001001, 0b01000101, 0b01000011]),
    // [
    Letter([0b01111111, 0b01000001, Letter::END_MARKER, 0, 0]),
    // '\'
    Letter([0b00000011, 0b00011100, 0b01100000, Letter::END_MARKER, 0]),
    // ]
    Letter([0b01000001, 0b01111111, Letter::END_MARKER, 0, 0]),
    // ^
    Letter([0b00000010, 0b00000001, 0b00000010, Letter::END_MARKER, 0]),
    // _
    Letter([
        0b10100000,
        0b10100000,
        0b10100000,
        0b10100000,
        Letter::END_MARKER,
    ]),
    // `
    Letter([0b00000001, 0b00000010, Letter::END_MARKER, 0, 0]),
    // a
    Letter([
        0b00100000,
        0b01010100,
        0b01010100,
        0b01111000,
        Letter::END_MARKER,
    ]),
    // b
    Letter([
        0b01111111,
        0b01000100,
        0b01000100,
        0b00111000,
        Letter::END_MARKER,
    ]),
    // c
    Letter([0b00111000, 0b01000100, 0b01000100, Letter::END_MARKER, 0]),
    // d
    Letter([
        0b00111000,
        0b01000100,
        0b01000100,
        0b01111111,
        Letter::END_MARKER,
    ]),
    // e
    Letter([
        0b00111000,
        0b01010100,
        0b01010100,
        0b00011000,
        Letter::END_MARKER,
    ]),
    // f
    Letter([0b01111110, 0b00000101, 0b00000001, Letter::END_MARKER, 0]),
    // g
    Letter([
        0b10001110,
        0b11010001,
        0b11010001,
        0b10111110,
        Letter::END_MARKER,
    ]),
    // h
    Letter([
        0b01111111,
        0b00000100,
        0b00000100,
        0b01111000,
        Letter::END_MARKER,
    ]),
    // i
    Letter([0b01111101, Letter::END_MARKER, 0, 0, 0]),
    // j
    Letter([0b01000000, 0b10100000, 0b01111101, Letter::END_MARKER, 0]),
    // k
    Letter([
        0b01111111,
        0b00010000,
        0b00101000,
        0b01000100,
        Letter::END_MARKER,
    ]),
    // l
    Letter([0b00111111, 0b01000000, Letter::END_MARKER, 0, 0]),
    // m
    Letter([0b01111000, 0b00000100, 0b01111000, 0b00000100, 0b01111000]),
    // n
    Letter([
        0b01111000,
        0b00000100,
        0b00000100,
        0b01111000,
        Letter::END_MARKER,
    ]),
    // o
    Letter([
        0b00111000,
        0b01000100,
        0b01000100,
        0b00111000,
        Letter::END_MARKER,
    ]),
    // p
    Letter([
        0b11111111,
        0b10010001,
        0b10010001,
        0b10001110,
        Letter::END_MARKER,
    ]),
    // q
    Letter([
        0b10001110,
        0b10010001,
        0b10010001,
        0b11111110,
        Letter::END_MARKER,
    ]),
    // r
    Letter([0b01111000, 0b00000100, 0b00000100, Letter::END_MARKER, 0]),
    // s
    Letter([
        0b01001000,
        0b01010100,
        0b01010100,
        0b00100100,
        Letter::END_MARKER,
    ]),
    // t
    Letter([
        0b00000100,
        0b00111111,
        0b01000100,
        0b00100000,
        Letter::END_MARKER,
    ]),
    // u
    Letter([
        0b00111100,
        0b01000000,
        0b01000000,
        0b01111100,
        Letter::END_MARKER,
    ]),
    // v
    Letter([0b00001100, 0b00110000, 0b01000000, 0b00110000, 0b00001100]),
    // w
    Letter([0b00111100, 0b01000000, 0b00111100, 0b01000000, 0b00111100]),
    // x
    Letter([0b01000100, 0b00101000, 0b00010000, 0b00101000, 0b01000100]),
    // y
    Letter([
        0b10001111,
        0b11010000,
        0b11010000,
        0b10111111,
        Letter::END_MARKER,
    ]),
    // z
    Letter([0b01000100, 0b01100100, 0b01010100, 0b01001100, 0b01000100]),
    // {
    Letter([0b00001000, 0b00110110, 0b01000001, Letter::END_MARKER, 0]),
    // |
    Letter([0b01110111, Letter::END_MARKER, 0, 0, 0]),
    // }
    Letter([0b01000001, 0b00110110, 0b00001000, Letter::END_MARKER, 0]),
    // ~
    Letter([0b00001000, 0b00000100, 0b00001000, 0b00010000, 0b00001000]),
];

const LETTER_UNKNOWN: Letter = Letter([0b01111111, 0b01000001, 0b01000001, 0b01000001, 0b01111111]);

const LETTER_UE: Letter = Letter([
    0b00111101,
    0b01000000,
    0b01000000,
    0b01111101,
    Letter::END_MARKER,
]);

const LETTER_OE: Letter = Letter([
    0b00111001,
    0b01000100,
    0b01000100,
    0b00111001,
    Letter::END_MARKER,
]);

const LETTER_AE: Letter = Letter([
    0b00100001,
    0b01010100,
    0b01010100,
    0b01111001,
    Letter::END_MARKER,
]);

const LETTER_SZ: Letter = Letter([0b01111110, 0b00000001, 0b00000101, 0b01001010, 0b00110000]);

const LETTER_UNI_STUTTGART: Letter =
    Letter([0b00001000, 0b00100010, 0b00001000, 0b00100010, 0b00001000]);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Letter(pub [u8; Self::MAX_WIDTH]);

impl Letter {
    pub const MAX_WIDTH: usize = 5;
    pub const END_MARKER: u8 = 0x80;

    pub const fn get(c: u8) -> &'static Letter {
        match c {
            c if b' ' <= c && c <= b'~' => &ASCII_PRINTABLE[(c - b' ') as usize],
            b'\xdc' | b'\xfc' => &LETTER_UE,
            b'\xd6' | b'\xf6' => &LETTER_OE,
            b'\xc4' | b'\xe4' => &LETTER_AE,
            b'\xdf' => &LETTER_SZ,
            b'\x80' => &LETTER_UNI_STUTTGART,
            _ => &LETTER_UNKNOWN,
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
