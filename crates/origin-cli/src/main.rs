//! Command-line interface for the ORIGIN brand discovery engine.

use clap::{Parser, Subcommand, ValueEnum};
use origin_core::{BrandReport, GenerateOptions, MAX_CANDIDATES, analyze_brand, generate};

#[derive(Debug, Parser)]
#[command(name = "origin", version, about = "Brand discovery engine")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate and rank deterministic candidate names.
    Generate {
        /// Maximum number of unique candidates to produce.
        #[arg(long, default_value_t = 25, value_parser = parse_count)]
        count: usize,

        /// Seed for reproducible generation.
        #[arg(long, default_value_t = 1)]
        seed: u64,

        /// Output representation.
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },

    /// Analyze one existing name using the explainable scoring engine.
    Check {
        /// Name to evaluate.
        name: String,

        /// Output representation.
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
}

fn parse_count(value: &str) -> Result<usize, String> {
    let count = value
        .parse::<usize>()
        .map_err(|error| format!("invalid candidate count: {error}"))?;

    if (1..=MAX_CANDIDATES).contains(&count) {
        Ok(count)
    } else {
        Err(format!(
            "candidate count must be between 1 and {MAX_CANDIDATES}"
        ))
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Generate {
            count,
            seed,
            format,
        } => {
            let candidates = generate(GenerateOptions { count, seed });
            match format {
                OutputFormat::Table => print_candidate_table(&candidates),
                OutputFormat::Json => print_json(&candidates),
            }
        }
        Command::Check { name, format } => {
            let report = analyze_brand(&name);
            match format {
                OutputFormat::Table => print_check_table(&report),
                OutputFormat::Json => print_json(&report),
            }
        }
    }
}

fn print_candidate_table(candidates: &[origin_core::Candidate]) {
    println!(
        "rank\toverall\tpronounceability\trhythm\tvowels\trepetition\ttransitions\taccepted\tname"
    );
    for (index, candidate) in candidates.iter().enumerate() {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            index + 1,
            candidate.score,
            candidate.pronounceability,
            candidate.rhythm,
            candidate.vowel_balance,
            candidate.repetition,
            candidate.transition_quality,
            yes_no(candidate.accepted),
            candidate.name
        );
    }
}

fn print_check_table(report: &BrandReport) {
    println!("name\t{}", report.normalized);
    println!("profile\t{}", report.profile);
    println!("overall_score\t{}", report.overall_score);
    println!("pronounceability\t{}", report.scores.pronounceability);
    println!("rhythm\t{}", report.scores.rhythm);
    println!("vowel_balance\t{}", report.scores.vowel_balance);
    println!("repetition\t{}", report.scores.repetition);
    println!(
        "transition_quality\t{}",
        report.scores.transition_quality
    );
    println!("accepted\t{}", yes_no(report.accepted));
    if report.warnings.is_empty() {
        println!("warnings\tnone");
    } else {
        for warning in &report.warnings {
            println!("warning\t{warning}");
        }
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn print_json<T: serde::Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("failed to serialize output: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_count;

    #[test]
    fn count_parser_accepts_supported_bounds() {
        assert_eq!(parse_count("1"), Ok(1));
        assert_eq!(parse_count("1000000"), Ok(1_000_000));
    }

    #[test]
    fn count_parser_rejects_unsupported_values() {
        assert!(parse_count("0").is_err());
        assert!(parse_count("1000001").is_err());
        assert!(parse_count("not-a-number").is_err());
    }
}
