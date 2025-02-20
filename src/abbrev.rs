use std::collections::HashMap;
use std::path::Path;
use eyre::{bail, Context, ContextCompat};
use itertools::Itertools;
use regex::{Regex, RegexBuilder};

pub struct Abbreviation<'a> {
    pub text: &'a str,
    pub attach_to_previous: bool,
}

#[derive(Default)]
pub struct Abbreviator {
    lowercase_matchers: HashMap<String, Abbrev>,
    regex_matchers: Vec<Abbrev>,
}

impl Abbreviator {
    pub fn from_file(file_path: &Path) -> eyre::Result<Abbreviator> {
        let file = std::fs::read_to_string(file_path)
            .context("Failed to read abbreviations file")?;

        let mut lowercase_matchers = HashMap::new();
        let mut regex_matchers = Vec::new();

        for line in file.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let abbrev = parse_abbrev(line)?;
            match &abbrev.matcher {
                AbbrevMatcher::Lowercase(matcher) => {
                    lowercase_matchers.insert(matcher.clone(), abbrev);
                }
                AbbrevMatcher::Regex(_) => {
                    regex_matchers.push(abbrev);
                }
            }
        }

        Ok(Abbreviator { lowercase_matchers, regex_matchers })
    }

    pub fn abbreviate(&self, text: &str) -> Option<Abbreviation> {
        if self.lowercase_matchers.is_empty() && self.regex_matchers.is_empty() {
            return None;
        }

        let lowercase = text.to_lowercase()
            .replace('-', " ")
            .split_whitespace()
            .join(" ");

        let abbrev = self.lowercase_matchers.get(&lowercase);
        if let Some(abbrev) = abbrev {
            return Some(abbrev.with_matching_case_to(text));
        }

        for abbrev in &self.regex_matchers {
            let AbbrevMatcher::Regex(re) = &abbrev.matcher else {
                continue
            };

            if re.is_match(&text) {
                return Some(abbrev.with_matching_case_to(text));
            }
        }

        None
    }
}

enum AbbrevMatcher {
    Lowercase(String),
    Regex(Regex),
}

struct Abbrev {
    pub matcher: AbbrevMatcher,
    pub abbrev: String,
    pub title_case_version: Option<String>,
    pub attach_to_previous: bool,
}

impl Abbrev {
    fn with_matching_case_to(&self, original_text: &str) -> Abbreviation {
        let is_title_case = original_text.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
        match (is_title_case, &self.title_case_version) {
            (true, Some(title_case)) => Abbreviation { text: title_case, attach_to_previous: self.attach_to_previous },
            _ => Abbreviation { text: &self.abbrev, attach_to_previous: self.attach_to_previous },
        }
    }
}

fn parse_abbrev(line: &str) -> eyre::Result<Abbrev> {
    let eq_pos = line
        .find('=')
        .context("Invalid abbreviation, no '=' found")?;
    let (matcher_def, abbrev) = line.split_at(eq_pos);
    let matcher = matcher_def.trim();
    let abbrev = abbrev[1..].trim();
    let (abbrev, attach_to_previous) = match abbrev.strip_prefix("<+") {
        Some(abbrev) => (abbrev, true),
        None => (abbrev, false),
    };

    let match_case = abbrev
        .chars()
        .next()
        .map(|c| c.is_lowercase())
        .unwrap_or(false);

    let title_case_version = if match_case {
        let mut title_case = abbrev.to_string();
        if !abbrev.is_empty() {
            title_case[0..1].make_ascii_uppercase();
            Some(title_case)
        } else {
            None
        }
    } else {
        None
    };

    if matcher.starts_with('/') {
        let matcher = &matcher[1..];
        let Some(closing_pos) = matcher.find('/') else {
            bail!("Invalid regex, no closing '/' found");
        };
        let flags = &matcher[closing_pos + 1..];
        let re = RegexBuilder::new(&matcher[..closing_pos])
            .case_insensitive(flags.contains('i'))
            .build()?;

        Ok(Abbrev {
            matcher: AbbrevMatcher::Regex(re),
            abbrev: abbrev.to_string(),
            title_case_version,
            attach_to_previous,
        })
    } else {
        Ok(Abbrev {
            matcher: AbbrevMatcher::Lowercase(matcher.to_lowercase()),
            abbrev: abbrev.to_string(),
            title_case_version,
            attach_to_previous,
        })
    }
}

