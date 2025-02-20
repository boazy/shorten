use eyre::Context;
use itertools::Itertools;
use std::env::args;
use std::io::stdin;

mod abbrev;
mod shortener;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let desired_max_length = match args().skip(1).exactly_one() {
        Ok(arg) => arg,
        Err(_) => {
            eprintln!("Usage: shortener <desired_max_length>");
            std::process::exit(1);
        }
    };

    let desired_max_length = desired_max_length
        .parse::<usize>()
        .context("Failed to parse desired max length as an integer")?;

    let shortener = shortener::Shortener::new(desired_max_length)?;
    let input = stdin().lines();
    for line in input {
        let line = line?;
        let shortened = shortener.shorten(&line);
        println!("{}", shortened);
    }

    Ok(())
}