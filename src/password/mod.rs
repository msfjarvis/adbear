use rand::prelude::*;

const ALL_CHARACTERS: [&str; 90] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
    "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "!", "#", "$", "%", "&",
    "(", ")", "*", "+", ",", "-", ".", "/", ":", ";", "<", "=", ">", "?", "@", "[", "]", "^", "_",
    "{", "|", "}", "~", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
];

pub fn generate() -> String {
    let mut rng = rand::thread_rng();
    (0..20)
        .map(|_| {
            let idx = rng.gen_range(0..ALL_CHARACTERS.len());
            ALL_CHARACTERS[idx]
        })
        .collect()
}
