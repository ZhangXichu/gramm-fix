use std::io::{self, Write};

use clap::{Parser, Subcommand};

use crate::checker::{self, Correction};

#[derive(Parser)]
#[command(name = "gramm-fix", about = "German grammar checker — compiler-style feedback")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check and fix a German sentence or paragraph
    Fix {
        /// The sentence to check
        sentence: String,
    },
}

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Fix { sentence }) => process(&sentence),
        None => run_repl(),
    }
}

fn run_repl() {
    println!("gramm-fix — German grammar checker");
    println!("Enter a sentence and press Enter. Ctrl+C or Ctrl+D to exit.\n");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let sentence = input.trim().trim_matches('"');
                if !sentence.is_empty() {
                    process(sentence);
                    println!();
                }
            }
            Err(e) => {
                eprintln!("error: {e}");
                break;
            }
        }
    }
}

fn process(sentence: &str) {
    match checker::check(sentence) {
        Some(correction) => display_correction(sentence, &correction),
        None => println!("✔ {sentence}\n  No errors found."),
    }
}

fn display_correction(original: &str, c: &Correction) {
    println!("✘ {original}");
    println!("✔ {}", c.corrected);

    // Align the caret line under the corrected span in the ✔ line.
    // "✔ " is 2 display columns; then count chars before the span start.
    let prefix = 2 + c.corrected[..c.span.0].chars().count();
    let span_width = c.corrected[c.span.0..c.span.0 + c.span.1].chars().count();

    println!("{}{}", " ".repeat(prefix), "^".repeat(span_width));
    println!("\nExplanation:\n  {}", c.explanation);
}
