use std::io::{Read, Write};

use ppmd_rust::{Ppmd8Decoder, Ppmd8Encoder, RestoreMethod};

const SMALL_ORDER: u32 = 2;
const SMALL_MEM_SIZE: u32 = 2048;

const BIG_ORDER: u32 = 16;
const BIG_MEM_SIZE: u32 = 1024 * 1024;

static APACHE2_TEXT: &str = include_str!("fixtures/apache2.txt");
static APACHE2_BIN_SMALL: &[u8] = include_bytes!("fixtures/apache2_small_8.bin");
static APACHE2_BIN_BIG: &[u8] = include_bytes!("fixtures/apache2_big_8.bin");

static GPL3_TEXT: &str = include_str!("fixtures/gpl3.txt");
static GPL3_BIN_SMALL: &[u8] = include_bytes!("fixtures/gpl3_small_8.bin");
static GPL3_BIN_BIG: &[u8] = include_bytes!("fixtures/gpl3_big_8.bin");

fn encoder_test(order: u32, memory: u32, input_str: &str, expected_data: &[u8]) {
    let mut writer = Vec::new();
    {
        let mut encoder =
            Ppmd8Encoder::new(&mut writer, order, memory, RestoreMethod::Restart).unwrap();
        encoder.write_all(input_str.as_bytes()).unwrap();
        encoder.flush().unwrap();
    }

    assert_eq!(expected_data, writer.as_slice());
}

fn decoder_test(order: u32, memory: u32, input_data: &[u8], expected_string: &str) {
    let mut decoder = Ppmd8Decoder::new(input_data, order, memory, RestoreMethod::Restart).unwrap();

    let mut decoded = vec![0; expected_string.len()];
    decoder.read_exact(&mut decoded).unwrap();

    assert_eq!(decoded.as_slice(), expected_string.as_bytes());

    let decoded_data = String::from_utf8(decoded).unwrap();

    assert_eq!(decoded_data, expected_string);
}

#[test]
fn ppmd8_apache2_small_mem_encoder() {
    encoder_test(SMALL_ORDER, SMALL_MEM_SIZE, APACHE2_TEXT, APACHE2_BIN_SMALL);
}

#[test]
fn ppmd8_apache2_small_mem_decoder() {
    decoder_test(SMALL_ORDER, SMALL_MEM_SIZE, APACHE2_BIN_SMALL, APACHE2_TEXT);
}

#[test]
fn ppmd8_apache2_big_mem_encoder() {
    encoder_test(BIG_ORDER, BIG_MEM_SIZE, APACHE2_TEXT, APACHE2_BIN_BIG);
}

#[test]
fn ppmd8_apache2_big_mem_decoder() {
    decoder_test(BIG_ORDER, BIG_MEM_SIZE, APACHE2_BIN_BIG, APACHE2_TEXT);
}

#[test]
fn ppmd8_gpl3_small_mem_encoder() {
    encoder_test(SMALL_ORDER, SMALL_MEM_SIZE, GPL3_TEXT, GPL3_BIN_SMALL);
}

#[test]
fn ppmd8_gpl3_small_mem_decoder() {
    decoder_test(SMALL_ORDER, SMALL_MEM_SIZE, GPL3_BIN_SMALL, GPL3_TEXT);
}

#[test]
fn ppmd8_gpl3_big_mem_encoder() {
    encoder_test(BIG_ORDER, BIG_MEM_SIZE, GPL3_TEXT, GPL3_BIN_BIG);
}

#[test]
fn ppmd8_gpl3_big_mem_decoder() {
    decoder_test(BIG_ORDER, BIG_MEM_SIZE, GPL3_BIN_BIG, GPL3_TEXT);
}
