use std::io::{Read, Write};
use std::fs::{File};
use std::env::args;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

fn slice_bits(n: u64, from: u8, to: u8) -> u64 {
    let mut mask = 0;

    for i in from..to {
        mask |= 0x8000000000000000 >> i;
    }

    (n & mask) >> (64 - to)
}

fn encode_13(n: u64) -> String {
    let n = n + 0xAC00;

    let a = 0b11100000u8 + slice_bits(n, 48, 52) as u8;
    let b = 0b10000000u8 + slice_bits(n, 52, 58) as u8;
    let c = 0b10000000u8 + slice_bits(n, 58, 64) as u8;

    return String::from_utf8(vec![a, b, c]).unwrap();
}

fn encode_104(buffer: &[u8]) -> String {
    let mut a = [0u8; 8];
    let mut b = [0u8; 8];

    for i in 0..8 {
        a[i] = buffer[i];
    }

    for i in 0..5 {
        b[i] = buffer[8 + i];
    }

    let a = u64::from_be_bytes(a);
    let b = u64::from_be_bytes(b);

    let mut s = String::new();

    for i in 0..4 {
        s += &encode_13(slice_bits(a, i * 13, (i + 1) * 13))[..];
    }

    s += &encode_13((slice_bits(a, 52, 64) << 1) + slice_bits(b, 0, 1))[..];

    for i in 0..3 {
        s += &encode_13(slice_bits(b, i * 13 + 1, (i + 1) * 13 + 1))[..];
    }

    s
}

fn encode(buffer: &[u8]) -> String {
    let mut encoded = String::new();
    let mut cursor = 0usize;
    let len = buffer.len();

    loop {
        let from = cursor * 13;
        let to = from + 13;

        if to > len {
            let mut remain = [0u8; 13];
            for i in 0..(len - from) {
                remain[13 - (len - from) + i] = buffer[i];
            }

            encoded.push_str(&encode_104(&remain));
            break;
        }

        encoded.push_str(&encode_104(&buffer[from..to])[..]);
        cursor += 1;
    }

    encoded
}

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() < 2 {
        println!("usage: encoder <file>");
        exit(1);
    }

    let path = args[1].clone();
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            panic!(e.to_string());
        }
    };

    let size = file.metadata().unwrap().len();

    let mut out_file = match File::create("encoded.txt") {
        Ok(f) => f,
        Err(e) => {
            panic!(e);
        }
    };

    let mut buffer = [0u8; 13 * 10000];
    let mut read = 0;

    loop {
        println!("{}{}/{}", termion::clear::All, read, size);
        match file.read(&mut buffer) {
            Ok(n) => {
                read += n;
                if n == 0 {
                    break;
                }

                out_file.write(encode(&buffer[0..n]).as_bytes()).unwrap();
            }
            Err(e) => {
                panic!(e);
            }
        }
    }
}
