use clap::{Parser, Subcommand, ValueEnum};
use origin_core::{GenerateOptions, MAX_CANDIDATES, generate};

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
                OutputFormat::Table => print_table(&candidates),
                OutputFormat::Json => print_json(&candidates),
            }
        }
    }
}

fn print_table(candidates: &[origin_core::Candidate]) {
    println!("rank\tscore\tname");
    for (index, candidate) in candidates.iter().enumerate() {
        println!("{}\t{}\t{}", index + 1, candidate.score, candidate.name);
    }
}

fn print_json(candidates: &[origin_core::Candidate]) {
    match serde_json::to_string_pretty(candidates) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("failed to serialize candidates: {error}");
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
