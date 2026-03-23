use std::io::{self, Write};

use clap::{Parser, Subcommand};
use colored::{Color, Colorize};

use crate::checker::{self, CheckResult, Correction, ErrorType};

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
    dotenvy::dotenv().ok();

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
        Ok(result) if result.corrections.is_empty() => {
            println!("{} {sentence}\n  No errors found.", "✔".green());
        }
        Ok(result) => display_corrections(sentence, &result),
        Err(e) => eprintln!("{} {e}", "error:".red().bold()),
    }
}

fn error_color(t: &ErrorType) -> Color {
    match t {
        ErrorType::Grammar     => Color::Yellow,
        ErrorType::Spelling    => Color::Red,
        ErrorType::Punctuation => Color::Cyan,
        ErrorType::WordOrder   => Color::Magenta,
        ErrorType::Style       => Color::Blue,
    }
}

/// Return `text` with the first occurrence of `word` wrapped in `color`.
fn highlight_first(text: &str, word: &str, color: Color) -> String {
    if word.is_empty() {
        return text.to_string();
    }
    match text.find(word) {
        Some(pos) => format!(
            "{}{}{}",
            &text[..pos],
            text[pos..pos + word.len()].color(color).bold(),
            &text[pos + word.len()..]
        ),
        None => text.to_string(),
    }
}

fn display_corrections(original: &str, result: &CheckResult) {
    // ✘ line: highlight every wrong word in its error color.
    let mut original_line = original.to_string();
    for c in &result.corrections {
        let color = error_color(&c.error_type);
        original_line = highlight_first(&original_line, &c.wrong_word, color);
    }
    println!("{} {original_line}", "✘".red());

    // ✔ line: highlight every corrected word in green.
    let mut corrected_line = result.corrected.clone();
    for c in &result.corrections {
        let correct_word = &result.corrected[c.span.0..c.span.0 + c.span.1];
        corrected_line = highlight_first(&corrected_line, correct_word, Color::Green);
    }
    println!("{} {corrected_line}", "✔".green());

    // One caret line + explanation per correction.
    for c in &result.corrections {
        print_caret_line(&result.corrected, c);
    }

    if let Some(ref s) = result.suggested {
        println!("\n{} {}", "→".cyan().bold(), s.italic());
    }
}

fn print_caret_line(corrected: &str, c: &Correction) {
    let color = error_color(&c.error_type);
    let label = c.error_type.label().color(color).bold();

    // Align carets under the corrected span in the ✔ line.
    // "✔ " is 2 display columns; then count chars before the span start.
    let prefix     = 2 + corrected[..c.span.0].chars().count();
    let span_width = corrected[c.span.0..c.span.0 + c.span.1].chars().count();

    println!(
        "{}{}  [{label}] {}",
        " ".repeat(prefix),
        "^".repeat(span_width).color(color).bold(),
        c.explanation
    );
}
