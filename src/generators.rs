use std::path::PathBuf;

use rand::{distributions::Alphanumeric, prelude::*};

const WORD_LIST: &str = include_str!("wordlist.txt");

pub fn gen_password(length: u32) -> String {
    const CHARS: &str = concat!(
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        "0123456789",
        r##"!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"##
    );

    let mut rng = thread_rng();
    let mut password: String;
    loop {
        password = (0..length)
            .map(|_| {
                let index = rng.gen_range(0..CHARS.len());
                CHARS
                    .chars()
                    .nth(index)
                    .expect("expected enough characters in alphabet")
            })
            .collect();

        if password.chars().any(|c| c.is_lowercase())
            && password.chars().any(|c| c.is_uppercase())
            && (password.chars().filter(|c| c.is_digit(10)).count() as f64 / length as f64) * 100.0
                >= 20.0
        {
            break;
        }
    }
    password
}

pub fn gen_token(length: u32) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| format!("{:02x}", rng.gen_range(0..=255)))
        .collect()
}

pub fn gen_key(length: u32) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(length as usize)
        .collect()
}

pub fn gen_passphrase(length: u32, wordlist: &Option<PathBuf>) -> String {
    let words: Vec<&str> = if let Some(_path) = wordlist {
        todo!("Custom wordlists are not suported yet")
    } else {
        WORD_LIST.split('\n').collect()
    };

    (0..length)
        .map(|_| {
            let dice_roll: u32 = (0..5)
                .map(|_| rand::thread_rng().gen_range(1..=6))
                .fold(0, |acc, roll| acc * 6 + roll as u32 - 1);
            words[dice_roll as usize]
        })
        .collect::<Vec<&str>>()
        .join(" ")
}
