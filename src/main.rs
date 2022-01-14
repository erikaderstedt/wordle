use std::fs;
use std::error::Error;
use std::fmt;
use std::io::{stdin,stdout,Write};
use std::io::prelude::*;
use colored::*;
use encoding::{Encoding, EncoderTrap};
use encoding::all::ISO_8859_1;

#[derive(Copy,Clone)]
struct Word {
    letters: [u8; 5]
}

fn is_letter(a: u8) -> bool {
    (a >= b'a' && a <= b'z') || a == 246 || a == 228 || a == 229
}

impl Word {
    fn from_bytes(a: &[u8]) -> Option<Word> {
        if is_letter(a[0]) && is_letter(a[1]) && is_letter(a[2]) && is_letter(a[3]) && is_letter(a[4]) {
            Some(Word { letters: [a[0], a[1], a[2], a[3], a[4]] })
        } else {
            None
        }
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}{}{}{}{}", self.letters[0] as char,
            self.letters[1] as char,
            self.letters[2] as char,
            self.letters[3] as char,
            self.letters[4] as char)
    }
}

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

fn load_words(s: &str) -> Vec<Word> {
    let mut a = [0u8;6];
    let mut f = fs::File::open(s).expect("Unable to open input file");
    let mut v: Vec<Word> = Vec::with_capacity(5000);
    while f.read_exact(&mut a).is_ok() {
        if let Some(w) = Word::from_bytes(&a) {
            v.push(w)
        }
    }
    v
}

#[derive(Copy,Clone)]
enum Reply {
    Correct,
    WrongLocation,
    NotInWord,
}

impl Word {

    fn update_counts(&self, counts: &mut [usize;29]) {
        for i in 0..5 {
            match self.letters[i] {
                246 => counts[28] += 1,
                228 => counts[27] += 1,
                229 => counts[26] += 1,
                c => counts[(c - 97) as usize] += 1,
            }
        }
    }

    fn score_from_counts(&self, counts: &[usize;29]) -> usize {
        (0..5).map(|i| match self.letters[i] {
                246 => counts[28],
                228 => counts[27],
                229 => counts[26],
                c => counts[(c - 97) as usize],
            }).product()
    }

    fn execute_guess(&self, result: [Reply;5], words: Vec<Word>) -> Vec<Word> {
        words.into_iter()
            .filter(|w| 
            (0..5).all(|i| match result[i] {
                Reply::Correct =>       w.letters[i] == self.letters[i],
                Reply::NotInWord =>     !w.letters.contains(&self.letters[i]),
                Reply::WrongLocation => w.letters[i] != self.letters[i] && w.letters.contains(&self.letters[i]),
            }))
        .collect()
    }

    fn has_duplicate_letter(&self) -> bool {
        self.letters[0] == self.letters[1] ||
        self.letters[0] == self.letters[2] ||
        self.letters[0] == self.letters[3] ||
        self.letters[0] == self.letters[4] ||
        self.letters[1] == self.letters[2] ||
        self.letters[1] == self.letters[3] ||
        self.letters[1] == self.letters[4] ||
        self.letters[2] == self.letters[3] ||
        self.letters[2] == self.letters[4] ||
        self.letters[3] == self.letters[4]
    }

    fn suggest(words: &Vec<Word>) -> Vec<Word> {
        let mut counts = [0usize;29];
        for w in words.iter() {
            w.update_counts(&mut counts);
        }
    
        let mut scores: Vec<(Word,usize)> = words.iter()
            .filter(|w| !w.has_duplicate_letter())
            .map(|w| (w.clone(), w.score_from_counts(&counts)))
            .collect();
        scores.sort_unstable_by_key(|a| a.1);
        scores.reverse();
        scores.into_iter().take(3).map(|a| a.0).collect()
    }
}


fn main() -> Result<(), Box<dyn Error>> {

    let mut words = load_words(&std::env::args().skip(1).next().unwrap());
    let mut num_guesses = 0;
    let mut replies = [Reply::NotInWord; 5];

    println!("Initial word list contains {} words.", words.len());
    loop {
        println!("{} guesses made, {} 5-letter words remaining", num_guesses, words.len());
        println!("Suggested guesses: {:?}", Word::suggest(&words));
        print!("Guess: ");
        let _=stdout().flush();
        let mut s = String::new();
        stdin().read_line(&mut s).expect("Did not enter a correct string");
        let a = ISO_8859_1.encode(&s, EncoderTrap::Ignore).expect("Invalid ISO 8859-1 string");
        let word = Word::from_bytes(&a).unwrap();
        for i in 0..5 {
            loop {
                print!("Letter {} ('{}'): (C)orrect, (N)ot in word, (W)rong Location? ", i+1, word.letters[i] as char);
                let _=stdout().flush();
                let mut s = String::new();
                stdin().read_line(&mut s).expect("Did not enter a correct string");
                match s.trim() {
                    "C" | "c" => replies[i] = Reply::Correct,
                    "W" | "w" => replies[i] = Reply::WrongLocation,
                    "N" | "n" => replies[i] = Reply::NotInWord,
                    _ => continue,
                }
                break;
            }
        }
        for i in 0..5 {
            print!("{} ",
            match replies[i] {
                Reply::Correct => (word.letters[i] as char).to_string().on_green().black(),
                Reply::WrongLocation => (word.letters[i] as char).to_string().on_bright_yellow().black(),
                Reply::NotInWord => (word.letters[i] as char).to_string().on_black().white(),
            });
        }
        println!("");
        
        words = word.execute_guess(replies, words);
        num_guesses += 1;
        if words.len() == 1 {
            println!("Answer should be {}", words[0]);
            break;
        }
    }


    Ok(())
}
