use std::{collections::BTreeSet, path::PathBuf};

use clap::{Args, ValueEnum};
use origin_core::{
    AvailabilityChecker, AvailabilityReport, AvailabilityTarget, MockAvailabilityProvider,
};

use super::availability_provider::PublicRegistryProvider;

const DEFAULT_DOMAIN_TLDS: [&str; 8] = ["com", "net", "org", "io", "ai", "app", "dev", "co"];

/// Screens one name across Origin's standard live clearance targets.
///
/// # Errors
///
/// Returns an error when the candidate name is invalid for a target provider.
pub fn live_report(name: &str) -> Result<AvailabilityReport, origin_core::AvailabilityError> {
    AvailabilityChecker::new(PublicRegistryProvider::new()).check_all(name, default_targets())
}

fn default_targets() -> Vec<AvailabilityTarget> {
    let mut targets = vec![
        AvailabilityTarget::GitHub,
        AvailabilityTarget::CratesIo,
        AvailabilityTarget::Npm,
        AvailabilityTarget::PyPi,
        AvailabilityTarget::Company,
        AvailabilityTarget::Web,
    ];
    targets.extend(DEFAULT_DOMAIN_TLDS.map(|tld| AvailabilityTarget::Domain { tld: tld.into() }));
    targets
}

/// Offline availability screening command.
#[derive(Debug, Args)]
pub struct AvailabilityCommand {
    /// Candidate name to screen.
    pub name: String,
    /// Screen every built-in public registry and standard domain TLD.
    #[arg(long)]
    pub all: bool,
    /// Additional registry target; can be repeated.
    #[arg(long, value_enum)]
    pub target: Vec<CliAvailabilityTarget>,
    /// Screen a domain under this TLD, for example `com`; can be repeated.
    #[arg(long)]
    pub domain: Vec<String>,
    /// Write the report as pretty JSON to this file.
    #[arg(long, value_name = "PATH")]
    pub json: Option<PathBuf>,
    /// Do not issue HTTP requests; emit explicit unknown fixture results instead.
    #[arg(long)]
    pub offline: bool,
}

/// Registry targets selectable from the command line.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliAvailabilityTarget {
    /// GitHub namespace availability.
    Github,
    /// crates.io package availability.
    Crates,
    /// npm package availability.
    Npm,
    /// `PyPI` package availability.
    Pypi,
    /// Exact-name company-register search.
    Company,
    /// Exact-name public web-presence search.
    Web,
}

impl From<CliAvailabilityTarget> for AvailabilityTarget {
    fn from(value: CliAvailabilityTarget) -> Self {
        match value {
            CliAvailabilityTarget::Github => Self::GitHub,
            CliAvailabilityTarget::Crates => Self::CratesIo,
            CliAvailabilityTarget::Npm => Self::Npm,
            CliAvailabilityTarget::Pypi => Self::PyPi,
            CliAvailabilityTarget::Company => Self::Company,
            CliAvailabilityTarget::Web => Self::Web,
        }
    }
}

impl AvailabilityCommand {
    /// Runs the requested checks against public sources, or deterministic fixtures offline.
    pub fn run(self) -> Result<AvailabilityReport, origin_core::AvailabilityError> {
        let mut targets = self
            .target
            .into_iter()
            .map(AvailabilityTarget::from)
            .collect::<Vec<_>>();
        if self.all {
            targets.extend(default_targets());
        }
        targets.extend(
            self.domain
                .into_iter()
                .map(|tld| AvailabilityTarget::Domain {
                    tld: tld.to_ascii_lowercase(),
                }),
        );
        if targets.is_empty() {
            targets.push(AvailabilityTarget::GitHub);
        }

        let mut seen = BTreeSet::new();
        targets.retain(|target| seen.insert(target.clone()));

        let report = if self.offline {
            AvailabilityChecker::new(MockAvailabilityProvider::new())
                .check_all(&self.name, targets)?
        } else {
            AvailabilityChecker::new(PublicRegistryProvider::new())
                .check_all(&self.name, targets)?
        };
        if let Some(path) = self.json {
            let content = serde_json::to_string_pretty(&report).map_err(|error| {
                origin_core::AvailabilityError::Provider {
                    target: "json-report".into(),
                    message: error.to_string(),
                }
            })?;
            std::fs::write(path, format!("{content}\n")).map_err(|error| {
                origin_core::AvailabilityError::Provider {
                    target: "json-report".into(),
                    message: error.to_string(),
                }
            })?;
        }
        Ok(report)
    }
}

/// Prints a compact human-readable availability report.
pub fn print_table(report: &AvailabilityReport) {
    println!("name\t{}", report.name);
    println!("target\tstatus\tchecked_at_unix_ms\tsource\tdetail");
    for result in &report.results {
        println!(
            "{}\t{:?}\t{}\t{}\t{}",
            result.target.code(),
            result.status,
            result
                .checked_at_unix_ms
                .map_or_else(|| "-".into(), |value| value.to_string()),
            result.source,
            result.detail.as_deref().unwrap_or("-")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::AvailabilityCommand;

    #[test]
    fn all_flag_selects_every_default_target() {
        let report = AvailabilityCommand {
            name: "qarvan".into(),
            all: true,
            target: Vec::new(),
            domain: Vec::new(),
            json: None,
            offline: true,
        }
        .run()
        .expect("offline scaffold should run");

        assert_eq!(report.results.len(), 14);
        assert_eq!(report.results[3].target.code(), "pypi");
        assert_eq!(report.results[4].target.code(), "company");
        assert_eq!(report.results[5].target.code(), "web");
        assert_eq!(report.results[6].target.code(), "domain.com");
        assert_eq!(report.results[13].target.code(), "domain.co");
    }

    #[test]
    fn no_flag_defaults_to_github() {
        let report = AvailabilityCommand {
            name: "qarvan".into(),
            all: false,
            target: Vec::new(),
            domain: Vec::new(),
            json: None,
            offline: true,
        }
        .run()
        .expect("offline scaffold should run");

        assert_eq!(report.results.len(), 1);
        assert_eq!(report.results[0].target.code(), "github");
    }

    #[test]
    fn duplicate_targets_are_checked_once() {
        let report = AvailabilityCommand {
            name: "qarvan".into(),
            all: true,
            target: vec![super::CliAvailabilityTarget::Github],
            domain: vec!["com".into()],
            json: None,
            offline: true,
        }
        .run()
        .expect("offline scaffold should run");

        assert_eq!(report.results.len(), 14);
    }
}
