use lazy_static::lazy_static;

pub mod xml;
pub mod json;

lazy_static! {
    static ref PREPARED_INDENTS: [&'static [u8]; 12] = [
        b"\n",
        b"\n  ",
        b"\n    ",
        b"\n      ",
        b"\n        ",
        b"\n          ",
        b"\n            ",
        b"\n              ",
        b"\n                ",
        b"\n                  ",
        b"\n                    ",
        b"\n                      ",
    ];
}