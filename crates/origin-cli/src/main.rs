//! Command-line interface for the ORIGIN brand discovery engine.

mod commands;

use clap::{Parser, Subcommand, ValueEnum};
use origin_core::{
    BeamSearchOptions, BeamSearchReport, BrandReport, DesignOptions, DesignedCandidate,
    ImproveOptions, ImprovementReport, MAX_DESIGN_CANDIDATES, MarkStrength, SimilarityReport,
    TrademarkContext, TrademarkReport, analyze_brand, analyze_similarity, analyze_trademark_risk,
    beam_search, built_in_catalog, compose_builtin, design_brands, improve,
};

#[derive(Debug, Parser)]
#[command(name = "origin", version, about = "Brand discovery engine")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Screen a candidate against supported registries without direct network access.
    Availability(commands::availability::AvailabilityCommand),

    /// List the source-backed semantic roots available to the built-in composer.
    Roots {
        /// Output representation.
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },

    /// Compose two built-in semantic root identifiers into an explainable candidate.
    Compose {
        /// Left root identifier, listed by `origin roots`.
        left: String,

        /// Right root identifier, listed by `origin roots`.
        right: String,

        /// Output representation.
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },

    /// Design and rank deterministic candidate names from four strategies.
    Generate {
        /// Maximum number of candidates to produce, up to ten thousand.
        #[arg(long, default_value_t = 25, value_parser = parse_count)]
        count: usize,

        /// Seed for reproducible generation.
        #[arg(long, default_value_t = 1)]
        seed: u64,

        /// Desired meaning or theme; can be repeated or comma-separated.
        #[arg(long, value_delimiter = ',')]
        meaning: Vec<String>,

        /// Explicit semantic root identifiers; can be repeated or comma-separated.
        #[arg(long, value_delimiter = ',')]
        roots: Vec<String>,

        /// Industry cue, for example `ai`, `logistics`, or `finance`.
        #[arg(long)]
        industry: Option<String>,

        /// Live-screen this many top-ranked candidates and print clearance evidence.
        #[arg(long, value_parser = parse_finalist_count)]
        finalists: Option<usize>,

        /// Number of internally ranked candidates to fully screen before finalist ranking.
        /// Every candidate in this pool is checked against every standard target.
        #[arg(long, value_parser = parse_screening_count)]
        screen_limit: Option<usize>,

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

    /// Compare two names with the deterministic similarity engine.
    Compare {
        /// Candidate name being evaluated.
        candidate: String,

        /// Existing or reference name.
        reference: String,

        /// Output representation.
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },

    /// Screen a candidate against an earlier mark for trademark-conflict risk.
    Trademark {
        /// Candidate name being evaluated.
        candidate: String,

        /// Earlier or reference mark.
        reference: String,

        /// Treat both names as operating in the same broad industry.
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        same_industry: bool,

        /// Treat the goods or services as directly overlapping.
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        overlapping_goods: bool,

        /// Treat both marks as targeting the same market or customer group.
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        same_market: bool,

        /// Estimated commercial strength of the earlier mark.
        #[arg(long, value_enum, default_value_t = CliMarkStrength::Average)]
        prior_mark_strength: CliMarkStrength,

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

    /// Search multiple sequential phoneme improvements with beam search.
    Optimize {
        /// Existing name to optimize.
        name: String,

        /// Maximum number of final ranked results to return.
        #[arg(long, default_value_t = 10, value_parser = parse_improvement_count)]
        count: usize,

        /// Maximum number of active candidates retained per depth.
        #[arg(long, default_value_t = 12, value_parser = parse_beam_width)]
        beam_width: usize,

        /// Maximum number of sequential one-phoneme mutations.
        #[arg(long, default_value_t = 2, value_parser = parse_depth)]
        depth: usize,

        /// Seed used for reproducible search and tie-breaking.
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

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliMarkStrength {
    Weak,
    Average,
    Strong,
    Famous,
}

impl From<CliMarkStrength> for MarkStrength {
    fn from(value: CliMarkStrength) -> Self {
        match value {
            CliMarkStrength::Weak => Self::Weak,
            CliMarkStrength::Average => Self::Average,
            CliMarkStrength::Strong => Self::Strong,
            CliMarkStrength::Famous => Self::Famous,
        }
    }
}

fn parse_count(value: &str) -> Result<usize, String> {
    parse_bounded_count(value, MAX_DESIGN_CANDIDATES, "candidate")
}

fn parse_improvement_count(value: &str) -> Result<usize, String> {
    parse_bounded_count(value, 1_000, "improvement")
}

fn parse_finalist_count(value: &str) -> Result<usize, String> {
    parse_bounded_count(value, 10, "finalist")
}

fn parse_screening_count(value: &str) -> Result<usize, String> {
    parse_bounded_count(value, 100, "screening")
}

fn parse_beam_width(value: &str) -> Result<usize, String> {
    parse_bounded_count(value, 250, "beam width")
}

fn parse_depth(value: &str) -> Result<usize, String> {
    parse_bounded_count(value, 8, "search depth")
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

#[allow(clippy::too_many_lines)] // Command dispatch remains intentionally visible in one place.
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Availability(command) => run_availability(command),
        Command::Roots { format } => print_roots(format),
        Command::Compose {
            left,
            right,
            format,
        } => compose_roots(&left, &right, format),
        Command::Generate {
            count,
            seed,
            meaning,
            roots,
            industry,
            finalists,
            screen_limit,
            format,
        } => run_generate(
            count,
            seed,
            meaning,
            roots,
            industry,
            finalists,
            screen_limit,
            format,
        ),
        Command::Check { name, format } => {
            let report = analyze_brand(&name);
            match format {
                OutputFormat::Table => print_check_table(&report),
                OutputFormat::Json => print_json(&report),
            }
        }
        Command::Compare {
            candidate,
            reference,
            format,
        } => {
            let report = analyze_similarity(&candidate, &reference);
            match format {
                OutputFormat::Table => print_similarity_table(&report),
                OutputFormat::Json => print_json(&report),
            }
        }
        Command::Trademark {
            candidate,
            reference,
            same_industry,
            overlapping_goods,
            same_market,
            prior_mark_strength,
            format,
        } => {
            let report = analyze_trademark_risk(
                &candidate,
                &reference,
                TrademarkContext {
                    same_industry,
                    overlapping_goods,
                    same_market,
                    prior_mark_strength: prior_mark_strength.into(),
                },
            );
            match format {
                OutputFormat::Table => print_trademark_table(&report),
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
        Command::Optimize {
            name,
            count,
            beam_width,
            depth,
            seed,
            format,
        } => {
            let report = beam_search(
                &name,
                BeamSearchOptions {
                    count,
                    beam_width,
                    depth,
                    seed,
                },
            );
            match format {
                OutputFormat::Table => print_beam_search_table(&report),
                OutputFormat::Json => print_json(&report),
            }
        }
    }
}

fn run_availability(command: commands::availability::AvailabilityCommand) {
    match command.run() {
        Ok(report) => commands::availability::print_table(&report),
        Err(error) => {
            eprintln!("availability check failed: {error}");
            std::process::exit(2);
        }
    }
}

#[allow(clippy::too_many_arguments)] // Mirrors the explicit CLI generate arguments.
fn run_generate(
    count: usize,
    seed: u64,
    meanings: Vec<String>,
    roots: Vec<String>,
    industry: Option<String>,
    finalists: Option<usize>,
    screen_limit: Option<usize>,
    format: OutputFormat,
) {
    let options = DesignOptions {
        count,
        seed,
        meanings,
        industry,
        roots,
    };
    let candidates = design_brands(&options);
    if let Some(finalists) = finalists {
        let generated_candidates = candidates.len();
        let screening_budget = screen_limit
            .unwrap_or_else(|| finalists.saturating_mul(3))
            .max(finalists)
            .min(candidates.len());
        let reports = candidates
            .into_iter()
            .take(screening_budget)
            .map(|candidate| {
                commands::availability::live_report(&candidate.name).map(|availability| {
                    FinalistReport {
                        candidate,
                        availability,
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>();
        match reports {
            Ok(mut reports) => {
                reports.sort_by(|left, right| {
                    right
                        .recommendation_rank()
                        .cmp(&left.recommendation_rank())
                        .then_with(|| right.final_score().cmp(&left.final_score()))
                        .then_with(|| left.candidate.name.cmp(&right.candidate.name))
                });
                let finalists = reports
                    .iter()
                    .filter(|report| !report.is_rejected())
                    .take(finalists)
                    .cloned()
                    .collect::<Vec<_>>();
                let run = FinalistRun::from_reports(generated_candidates, finalists, reports);
                match format {
                    OutputFormat::Table => print_finalist_table(&run),
                    OutputFormat::Json => print_json(&run),
                }
            }
            Err(error) => {
                eprintln!("finalist clearance failed: {error}");
                std::process::exit(2);
            }
        }
        return;
    }
    match format {
        OutputFormat::Table => print_designed_candidates_table(&candidates),
        OutputFormat::Json => print_json(&candidates),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct FinalistReport {
    candidate: DesignedCandidate,
    availability: origin_core::AvailabilityReport,
}

#[derive(Debug, serde::Serialize)]
struct FinalistRun {
    generated_candidates: usize,
    screened_candidates: usize,
    clear_count: usize,
    provisional_count: usize,
    rejected_count: usize,
    finalists: Vec<FinalistReport>,
    screened_reports: Vec<FinalistReport>,
}

impl FinalistRun {
    fn from_reports(
        generated_candidates: usize,
        finalists: Vec<FinalistReport>,
        screened_reports: Vec<FinalistReport>,
    ) -> Self {
        let clear_count = screened_reports
            .iter()
            .filter(|report| {
                report.availability.recommendation() == origin_core::ClearanceRecommendation::Clear
            })
            .count();
        let provisional_count = screened_reports
            .iter()
            .filter(|report| {
                report.availability.recommendation()
                    == origin_core::ClearanceRecommendation::Provisional
            })
            .count();
        let rejected_count = screened_reports.len() - clear_count - provisional_count;
        Self {
            generated_candidates,
            screened_candidates: screened_reports.len(),
            clear_count,
            provisional_count,
            rejected_count,
            finalists,
            screened_reports,
        }
    }
}

impl FinalistReport {
    fn recommendation_rank(&self) -> u8 {
        match self.availability.recommendation() {
            origin_core::ClearanceRecommendation::Clear => 2,
            origin_core::ClearanceRecommendation::Provisional => 1,
            origin_core::ClearanceRecommendation::Reject => 0,
        }
    }

    fn is_rejected(&self) -> bool {
        self.availability.recommendation() == origin_core::ClearanceRecommendation::Reject
    }

    fn final_score(&self) -> u8 {
        let design = u16::from(self.candidate.design_score);
        let evidence = u16::from(self.availability.evidence_score());
        u8::try_from((design * 65 + evidence * 35) / 100).unwrap_or(100)
    }
}

fn print_roots(format: OutputFormat) {
    let catalog = built_in_catalog();
    match format {
        OutputFormat::Table => print_roots_table(&catalog),
        OutputFormat::Json => print_json(&catalog.iter().collect::<Vec<_>>()),
    }
}

fn compose_roots(left: &str, right: &str, format: OutputFormat) {
    match compose_builtin(left, right) {
        Ok(composition) => match format {
            OutputFormat::Table => print_composition_table(&composition),
            OutputFormat::Json => print_json(&composition),
        },
        Err(error) => {
            eprintln!("semantic composition failed: {error}");
            std::process::exit(2);
        }
    }
}

fn print_designed_candidates_table(candidates: &[DesignedCandidate]) {
    println!("rank\tdesign\toverall\ttypography\tstrategy\taccepted\tinspiration\tname");
    for (index, candidate) in candidates.iter().enumerate() {
        println!(
            "{}\t{}\t{}\t{}\t{:?}\t{}\t{}\t{}",
            index + 1,
            candidate.design_score,
            candidate.analysis.overall_score,
            candidate.typography_score,
            candidate.strategy,
            yes_no(candidate.analysis.accepted),
            candidate.inspiration.join(" + "),
            candidate.name
        );
    }
}

fn print_finalist_table(run: &FinalistRun) {
    println!(
        "generated\t{}\tscreened\t{}\tclear\t{}\tprovisional\t{}\trejected\t{}",
        run.generated_candidates,
        run.screened_candidates,
        run.clear_count,
        run.provisional_count,
        run.rejected_count
    );
    println!("rank\tfinal\tdesign\tevidence\trecommendation\tstrategy\ttaken\tunknown\tname");
    for (index, report) in run.finalists.iter().enumerate() {
        println!(
            "{}\t{}\t{}\t{}\t{:?}\t{:?}\t{}\t{}\t{}",
            index + 1,
            report.final_score(),
            report.candidate.design_score,
            report.availability.evidence_score(),
            report.availability.recommendation(),
            report.candidate.strategy,
            report.availability.taken_count(),
            report.availability.unknown_count(),
            report.candidate.name
        );
    }
}

fn print_roots_table(catalog: &origin_core::LanguageCatalog) {
    println!("id\tlanguage\troot\tmeaning\tconfidence\tsource");
    for root in catalog.iter() {
        let meaning = root
            .meanings
            .first()
            .map_or("-", |meaning| meaning.gloss.as_str());
        let source = root
            .sources
            .first()
            .map_or("-", |source| source.title.as_str());
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            root.id,
            root.language,
            root.normalized,
            meaning,
            root.confidence.score(),
            source
        );
    }
}

fn print_composition_table(composition: &origin_core::SemanticComposition) {
    println!("candidate\t{}", composition.merge.merged());
    println!(
        "roots\t{} + {}",
        composition.left_root_id, composition.right_root_id
    );
    println!("meaning\t{}", composition.meaning);
    println!("overall_score\t{}", composition.analysis.overall_score);
    println!("accepted\t{}", yes_no(composition.analysis.accepted));
    for step in composition.merge.provenance_steps() {
        println!("provenance\t{step}");
    }
    print_warnings(&composition.analysis.warnings);
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
    print_warnings(&report.warnings);
}

fn print_similarity_table(report: &SimilarityReport) {
    println!("candidate\t{}", report.candidate);
    println!("reference\t{}", report.reference);
    println!("overall_similarity\t{}", report.overall_similarity);
    println!("risk\t{:?}", report.risk);
    println!("levenshtein\t{}", report.levenshtein_score);
    println!("damerau\t{}", report.damerau_score);
    println!("bigram\t{}", report.bigram_similarity);
    println!("trigram\t{}", report.trigram_similarity);
    println!("prefix\t{}", report.prefix_similarity);
    println!("suffix\t{}", report.suffix_similarity);
    println!("shared_characters\t{}", report.shared_character_ratio);
    println!("phonetic\t{}", report.phonetic_similarity);
    println!("visual\t{}", report.visual_similarity);
    println!("keyboard\t{}", report.keyboard_similarity);
    print_warnings(&report.warnings);
}

fn print_trademark_table(report: &TrademarkReport) {
    println!("candidate\t{}", report.similarity.candidate);
    println!("reference\t{}", report.similarity.reference);
    println!("similarity\t{}", report.similarity.overall_similarity);
    println!("risk_score\t{}", report.risk_score);
    println!("risk\t{:?}", report.risk);
    println!("recommendation\t{:?}", report.recommendation);
    println!(
        "provisionally_clear\t{}",
        yes_no(report.provisionally_clear)
    );
    println!("same_industry\t{}", yes_no(report.context.same_industry));
    println!(
        "overlapping_goods\t{}",
        yes_no(report.context.overlapping_goods)
    );
    println!("same_market\t{}", yes_no(report.context.same_market));
    println!(
        "prior_mark_strength\t{:?}",
        report.context.prior_mark_strength
    );
    for factor in &report.factors {
        println!(
            "factor\t{}\t{:+}\t{}",
            factor.code, factor.impact, factor.explanation
        );
    }
    print_warnings(&report.warnings);
}

fn print_improvement_table(report: &ImprovementReport) {
    println!("original\t{}", report.original.normalized);
    println!("original_score\t{}", report.original.overall_score);
    println!("original_accepted\t{}", yes_no(report.original.accepted));
    println!();
    println!("rank\toverall\tdelta\taffinity\taccepted\tchange\tname");

    for (index, suggestion) in report.suggestions.iter().enumerate() {
        println!(
            "{}\t{}\t{:+}\t{}\t{}\t{}:{}>{}\t{}",
            index + 1,
            suggestion.score,
            suggestion.score_delta,
            suggestion.phonetic_affinity,
            yes_no(suggestion.accepted),
            suggestion.changed_position,
            suggestion.replaced,
            suggestion.replacement,
            suggestion.name
        );
    }
}

fn print_beam_search_table(report: &BeamSearchReport) {
    println!("original\t{}", report.original.normalized);
    println!("original_score\t{}", report.original.overall_score);
    println!("original_accepted\t{}", yes_no(report.original.accepted));
    println!();
    println!("rank\toverall\tdelta\tsteps\taccepted\tname\tpath");

    for (index, candidate) in report.results.iter().enumerate() {
        println!(
            "{}\t{}\t{:+}\t{}\t{}\t{}\t{}",
            index + 1,
            candidate.score,
            candidate.total_delta,
            candidate.steps.len(),
            yes_no(candidate.accepted),
            candidate.name,
            candidate.path.join(" -> ")
        );
    }
}

fn print_warnings(warnings: &[String]) {
    if warnings.is_empty() {
        println!("warnings\tnone");
    } else {
        for warning in warnings {
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
    use clap::Parser;

    use super::{
        Cli, CliMarkStrength, Command, parse_beam_width, parse_count, parse_depth,
        parse_improvement_count,
    };

    #[test]
    fn count_parser_accepts_supported_bounds() {
        assert_eq!(parse_count("1"), Ok(1));
        assert_eq!(parse_count("10000"), Ok(10_000));
    }

    #[test]
    fn count_parser_rejects_unsupported_values() {
        assert!(parse_count("0").is_err());
        assert!(parse_count("10001").is_err());
        assert!(parse_count("not-a-number").is_err());
    }

    #[test]
    fn improvement_count_parser_enforces_smaller_bound() {
        assert_eq!(parse_improvement_count("1"), Ok(1));
        assert_eq!(parse_improvement_count("1000"), Ok(1_000));
        assert!(parse_improvement_count("0").is_err());
        assert!(parse_improvement_count("1001").is_err());
    }

    #[test]
    fn beam_search_parsers_enforce_safe_bounds() {
        assert_eq!(parse_beam_width("250"), Ok(250));
        assert_eq!(parse_depth("8"), Ok(8));
        assert!(parse_beam_width("251").is_err());
        assert!(parse_depth("9").is_err());
    }

    #[test]
    fn compare_command_parses_two_names() {
        let cli = Cli::try_parse_from(["origin", "compare", "orign", "origin"])
            .expect("compare command should parse");

        assert!(matches!(
            cli.command,
            Command::Compare {
                candidate,
                reference,
                ..
            } if candidate == "orign" && reference == "origin"
        ));
    }

    #[test]
    fn availability_command_parses_all_targets() {
        let cli = Cli::try_parse_from(["origin", "availability", "qarvan", "--all"])
            .expect("availability command should parse");

        assert!(matches!(
            cli.command,
            Command::Availability(command) if command.name == "qarvan" && command.all
        ));
    }

    #[test]
    fn semantic_commands_parse() {
        assert!(matches!(
            Cli::try_parse_from(["origin", "roots"]),
            Ok(Cli {
                command: Command::Roots { .. }
            })
        ));
        assert!(matches!(
            Cli::try_parse_from(["origin", "compose", "latin-lux", "latin-via"]),
            Ok(Cli {
                command: Command::Compose { .. }
            })
        ));
    }

    #[test]
    fn trademark_command_parses_explicit_context() {
        let cli = Cli::try_parse_from([
            "origin",
            "trademark",
            "orign",
            "origin",
            "--same-industry=false",
            "--overlapping-goods=false",
            "--same-market=false",
            "--prior-mark-strength",
            "famous",
        ])
        .expect("trademark command should parse");

        assert!(matches!(
            cli.command,
            Command::Trademark {
                same_industry: false,
                overlapping_goods: false,
                same_market: false,
                prior_mark_strength: CliMarkStrength::Famous,
                ..
            }
        ));
    }
}
