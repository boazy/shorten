use crate::abbrev::{Abbreviation, Abbreviator};
use crate::stop_words::STOP_WORDS;
use eyre::Context;
use itertools::Itertools;
use std::borrow::Cow;
use std::collections::HashSet;
use xdg::BaseDirectories;

pub struct Shortener {
    desired_max_length: usize,
    stop_words: HashSet<&'static str>,
    abbreviator: Abbreviator,
}

impl Shortener {
    pub fn new(desired_max_length: usize) -> eyre::Result<Shortener> {
        let base_dirs =
            BaseDirectories::with_prefix("shorten").context("Failed to get base directories")?;

        let abbrev_path = base_dirs.get_config_file("abbrev.lst");
        let abbreviator = if abbrev_path.exists() {
            Abbreviator::from_file(&abbrev_path).context("Failed to load abbreviations")?
        } else {
            Abbreviator::default()
        };

        Ok(Shortener {
            desired_max_length,
            stop_words: HashSet::from_iter(STOP_WORDS.iter().map(|s| *s)),
            abbreviator,
        })
    }

    pub fn shorten<'a>(&self, text: &'a str) -> Cow<'a, str> {
        if text.len() <= self.desired_max_length {
            return Cow::Borrowed(text);
        }

        let trimmed = text.trim();
        if trimmed.len() <= self.desired_max_length {
            return Cow::Borrowed(trimmed);
        }

        let without_stop_words = trimmed
            .split_whitespace()
            .filter(|s| !self.stop_words.contains(s))
            .join(" ");

        if without_stop_words.len() <= self.desired_max_length {
            return Cow::Owned(without_stop_words);
        };

        let words = without_stop_words.split_whitespace();
        let mut prev_word: Option<&str> = None;
        let mut abbreviated = String::with_capacity(without_stop_words.len());
        for word in words {
            let Some(found_prev_word) = prev_word else {
                prev_word = Some(word);
                continue;
            };

            let pair_of_words = format!("{} {}", found_prev_word, word);
            if self.attempt_abbrev(&mut abbreviated, &pair_of_words) {
                prev_word = None;
            } else {
                // Attempt to abbreviate the previous word and save the current word for later
                self.abbrev_or_add(&mut abbreviated, found_prev_word);
                prev_word = Some(word);
            }
        };

        // If there's a word left over, add it to the output (abbreviated or not)
        if let Some(prev_word) = prev_word {
            self.abbrev_or_add(&mut abbreviated, prev_word);
        }

        Cow::Owned(abbreviated)
    }

    fn attempt_abbrev(&self, abbreviated: &mut String, text: &str) -> bool {
        self.abbreviator.abbreviate(text)
            .map(|abbrev| abbreviated.add_abbrev(abbrev))
            .is_some()
    }

    fn abbrev_or_add(&self, abbreviated: &mut String, text: &str) {
        match self.abbreviator.abbreviate(text) {
            Some(abbrev) => abbreviated.add_abbrev(abbrev),
            None => abbreviated.add_with_space(text),
        }
    }
}

trait AddWithSpace {
    fn add_with_space(&mut self, s: &str);
    fn add_abbrev(&mut self, abbrev: Abbreviation);
}

impl AddWithSpace for String {
    fn add_with_space(&mut self, s: &str) {
        if !self.is_empty() {
            self.push(' ');
        }
        self.push_str(s);
    }

    fn add_abbrev(&mut self, abbrev: Abbreviation) {
        if abbrev.attach_to_previous {
            self.push_str(abbrev.text);
        }
        else {
            self.add_with_space(abbrev.text);
        }
    }
}
