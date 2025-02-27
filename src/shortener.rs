use crate::abbrev::{Abbreviation, Abbreviator};
use eyre::Context;
use std::borrow::Cow;
use xdg::BaseDirectories;

pub struct Shortener {
    desired_max_length: usize,
    abbreviator: Abbreviator,
}

impl Shortener {
    pub fn new(desired_max_length: usize) -> eyre::Result<Shortener> {
        let base_dirs =
            BaseDirectories::with_prefix("shorten").context("Failed to get base directories")?;

        let abbrev_path = base_dirs.get_config_file("abbrev.lst");
        let abbreviator = if abbrev_path.exists() {
            Abbreviator::try_from_file(&abbrev_path).context("Failed to load abbreviations")?
        } else {
            Abbreviator::default()
        };

        Self::with_abbreviator(desired_max_length, abbreviator)
    }

    pub fn with_abbreviator(
        desired_max_length: usize,
        abbreviator: Abbreviator,
    ) -> eyre::Result<Shortener> {
        Ok(Shortener {
            desired_max_length,
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

        let words = trimmed.split_whitespace();
        let mut prev_word: Option<&str> = None;
        let mut abbreviated = String::with_capacity(trimmed.len());
        for word in words {
            let enclosed = process_enclosed_word(word);

            // Attempt raw enclosed abbreviation before removing enclosing
            if enclosed.is_enclosed() {
                if let Some(found_prev_word) = prev_word {
                    self.abbrev_or_add(&mut abbreviated, found_prev_word);
                    prev_word = None;
                }
                if !self.attempt_abbrev(&mut abbreviated, word) {
                    // Remove enclosing and try word individually
                    let abbrev = self.abbreviator.abbreviate(enclosed.word);
                    match abbrev {
                        Some(abbrev) => {
                            if !abbrev.text.is_empty() {
                                abbreviated.add_with_space(enclosed.openers);
                                abbreviated.add_abbrev(abbrev);
                                abbreviated.push_str(enclosed.closers);
                            }
                        }
                        None => {
                            abbreviated.add_with_space(enclosed.openers);
                            self.abbrev_or_add(&mut abbreviated, enclosed.word);
                            abbreviated.push_str(enclosed.closers);
                        }
                    }
                }
                continue;
            }

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
        }

        // If there's a word left over, add it to the output (abbreviated or not)
        if let Some(prev_word) = prev_word {
            self.abbrev_or_add(&mut abbreviated, prev_word);
        }

        Cow::Owned(abbreviated)
    }

    fn attempt_abbrev(&self, abbreviated: &mut String, text: &str) -> bool {
        self.abbreviator
            .abbreviate(text)
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
        if s.is_empty() {
            return;
        }

        let is_opener = self
            .chars()
            .last()
            .is_some_and(|c| c.is_opener());

        if !is_opener && !self.is_empty() {
            self.push(' ');
        }
        self.push_str(s);
    }

    fn add_abbrev(&mut self, abbrev: Abbreviation) {
        if abbrev.text.is_empty() {
            return;
        }
        if abbrev.attach_to_previous {
            self.push_str(abbrev.text);
        } else {
            self.add_with_space(abbrev.text);
        }
    }
}

trait Enclosing {
    fn is_opener(&self) -> bool;
    fn is_closer(&self) -> bool;
}

impl Enclosing for char {
    fn is_opener(&self) -> bool {
        matches!(
            self,
            '(' | '[' | '{' | '<' | '"' | '*'
        )
    }
    fn is_closer(&self) -> bool {
        matches!(
            self,
            ')' | ']' | '}' | '>' | '"' | '*'
        )
    }
}

struct EnclosedWord<'a> {
    word: &'a str,
    openers: &'a str,
    closers: &'a str,
}

impl EnclosedWord<'_> {
    fn is_enclosed(&self) -> bool {
        !self.openers.is_empty() || !self.closers.is_empty()
    }
}

fn process_enclosed_word(input: &str) -> EnclosedWord {
    let opener_len = input.chars().take_while(|c| c.is_opener()).count();
    let closer_len = input.chars().rev().take_while(|c| c.is_closer()).count();
    EnclosedWord {
        word: &input[opener_len..input.len() - closer_len],
        openers: &input[..opener_len],
        closers: &input[input.len() - closer_len..],
    }
}

#[cfg(test)]
mod tests {
    use std::iter::zip;
    use crate::abbrev::Abbreviator;
    use crate::shortener::Shortener;

    #[test]
    fn test_shorten() {
        let lines: Vec<&str> = r#"
            Architecture              = arch
            Learning                  = learn
            Audience                  = audn
            Session                   = sesn
            Excellence                = excl
            Section                   = <+課
            Department                = <+部
            Weekly                    = 毎週
            Monthly                   = 毎月

            One = 1

            Meeeting =
            Rescheduled =
            [Monthly] = [M]
            [Weekly] = [W]
        "#.split('\n').collect();
        let abbreviator = Abbreviator::from_lines(lines.into_iter()).unwrap();
        let shortener = Shortener::with_abbreviator(10, abbreviator).unwrap();

        let input = vec![
            "Architecture Section Learning Session",
            "*Rescheduled* [W] MPD Architecture Excellence Group Weekly Connect",
            "[Monthly] CLSD All Hands Meeting *Rescheduled*",
            "RIAM Tech Camp (Session one)",
        ];

        let expected= vec![
            "Arch課 Learn Sesn",
            "[W] MPD Arch Excl Group 毎週 Connect",
            "[M] CLSD All Hands Meeting",
            "RIAM Tech Camp (Sesn 1)",
        ];

        for (input, expected) in zip(input, expected) {
            let shortened = shortener.shorten(input);
            assert_eq!(shortened, expected);
        }
    }
}
