//! Command-line interface for the ORIGIN brand discovery engine.

use clap::{Parser, Subcommand, ValueEnum};
use origin_core::{
    BrandReport, GenerateOptions, ImproveOptions, ImprovementReport, MAX_CANDIDATES, analyze_brand,
    generate, improve,
};

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

    /// Generate and rank deterministic one-phoneme improvements.
    Improve {
        /// Existing name to improve.
        name: String,

        /// Maximum number of ranked suggestions to return.
        #[arg(long, default_value_t = 10, value_parser = parse_improvement_count)]
        count: usize,

        /// Seed used to break equal-score ties reproducibly.
        #[arg(long, default_value_t = 1)]
        seed: u64,

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
    parse_bounded_count(value, MAX_CANDIDATES, "candidate")
}

fn parse_improvement_count(value: &str) -> Result<usize, String> {
    parse_bounded_count(value, 1_000, "improvement")
}

fn parse_bounded_count(value: &str, maximum: usize, label: &str) -> Result<usize, String> {
    let count = value
        .parse::<usize>()
        .map_err(|error| format!("invalid {label} count: {error}"))?;

    if (1..=maximum).contains(&count) {
        Ok(count)
    } else {
        Err(format!("{label} count must be between 1 and {maximum}"))
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
        Command::Improve {
            name,
            count,
            seed,
            format,
        } => {
            let report = improve(&name, ImproveOptions { count, seed });
            match format {
                OutputFormat::Table => print_improvement_table(&report),
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
    println!("transition_quality\t{}", report.scores.transition_quality);
    println!("accepted\t{}", yes_no(report.accepted));
    if report.warnings.is_empty() {
        println!("warnings\tnone");
    } else {
        for warning in &report.warnings {
            println!("warning\t{warning}");
        }
    }
}

fn print_improvement_table(report: &ImprovementReport) {
    println!("original\t{}", report.original.normalized);
    println!("original_score\t{}", report.original.overall_score);
    println!("original_accepted\t{}", yes_no(report.original.accepted));
    println!();
    println!("rank\toverall\tdelta\taccepted\tchange\tname");

    for (index, suggestion) in report.suggestions.iter().enumerate() {
        println!(
            "{}\t{}\t{:+}\t{}\t{}:{}>{}\t{}",
            index + 1,
            suggestion.score,
            suggestion.score_delta,
            yes_no(suggestion.accepted),
            suggestion.changed_position,
            suggestion.replaced,
            suggestion.replacement,
            suggestion.name
        );
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
    use super::{parse_count, parse_improvement_count};

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

    #[test]
    fn improvement_count_parser_enforces_smaller_bound() {
        assert_eq!(parse_improvement_count("1"), Ok(1));
        assert_eq!(parse_improvement_count("1000"), Ok(1_000));
        assert!(parse_improvement_count("0").is_err());
        assert!(parse_improvement_count("1001").is_err());
    }
}
