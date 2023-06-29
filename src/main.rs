use std::fs;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::io::{stdin,stdout,Write};
use colored::*;

#[derive(Copy,Clone)]
struct Word {
    letters: [char; 5]
}

impl FromStr for Word {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c: Vec<char> = s.trim().chars().collect();
        if c.len() != 5 {
            println!("-{}-", s);
            Err("Not a five-letter word.")
        } else {
            Ok(Word {
                letters: [c[0].to_ascii_lowercase(), 
                        c[1].to_ascii_lowercase(), 
                        c[2].to_ascii_lowercase(), 
                        c[3].to_ascii_lowercase(), 
                        c[4].to_ascii_lowercase()]
            })
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
    fs::read_to_string(s)
        .expect("Unable to open input file")
        .lines()
        .filter_map(|line| Word::from_str(line).ok() )
        .collect()
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
                'ö' => counts[28] += 1,
                'ä' => counts[27] += 1,
                'å' => counts[26] += 1,
                c if (c as u8) - 97 < 29 => counts[((c as u8) - 97) as usize] += 1,
                _ => (),
            }
        }
    }

    fn score_from_counts(&self, counts: &[usize;29]) -> usize {
        (0..5).map(|i| match self.letters[i] {
                'ö' => counts[28],
                'ä' => counts[27],
                'å' => counts[26],
                c if (c as u8) - 97 < 29 => counts[((c as u8) - 97) as usize],
                _ => 1,
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
        if words.len() <= 5 {
            println!("  (all remaining words: {:?})", words);
        }
        print!("Guess: ");
        let _=stdout().flush();
        let mut s = String::new();
        stdin().read_line(&mut s).expect("Did not enter a correct string");
        let word = Word::from_str(&s).unwrap();
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
