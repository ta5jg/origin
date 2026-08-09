use std::time::{Duration, SystemTime, UNIX_EPOCH};

use origin_core::{
    AvailabilityError, AvailabilityProvider, AvailabilityResult, AvailabilityStatus,
    AvailabilityTarget,
};

/// HTTP-backed provider for public registries that require no credentials.
#[derive(Debug)]
pub struct PublicRegistryProvider {
    agent: ureq::Agent,
}

impl Default for PublicRegistryProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl PublicRegistryProvider {
    /// Creates an HTTP provider with a short, bounded request timeout.
    #[must_use]
    pub fn new() -> Self {
        Self {
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(5))
                .timeout_read(Duration::from_secs(10))
                .user_agent("origin/0.1 availability-check")
                .build(),
        }
    }
}

impl AvailabilityProvider for PublicRegistryProvider {
    fn check(
        &self,
        target: &AvailabilityTarget,
        name: &str,
    ) -> Result<AvailabilityResult, AvailabilityError> {
        let url = target_url(target, name)?;
        let checked_at_unix_ms = now_unix_ms();
        let response = self.agent.get(&url).call();
        let status = match response {
            Ok(response) if (200..300).contains(&response.status()) => {
                if matches!(
                    target,
                    AvailabilityTarget::Company | AvailabilityTarget::Web
                ) {
                    let body = response.into_string().unwrap_or_default();
                    let exact_match = match target {
                        AvailabilityTarget::Company => company_match(&body, name),
                        AvailabilityTarget::Web => web_match(&body, name),
                        _ => false,
                    };
                    return Ok(AvailabilityResult::new(
                        target.clone(),
                        name,
                        if exact_match {
                            AvailabilityStatus::Taken
                        } else {
                            AvailabilityStatus::Available
                        },
                        url,
                    )
                    .with_evidence(
                        if exact_match {
                            "exact name found in public source"
                        } else {
                            "no exact name found in public source"
                        },
                        checked_at_unix_ms,
                    ));
                }
                AvailabilityStatus::Taken
            }
            Ok(response) if response.status() == 404 => AvailabilityStatus::Available,
            Err(ureq::Error::Status(404, _)) => AvailabilityStatus::Available,
            Ok(_) | Err(ureq::Error::Status(_, _)) => AvailabilityStatus::Unknown,
            Err(ureq::Error::Transport(error)) => {
                return Ok(AvailabilityResult::new(
                    target.clone(),
                    name,
                    AvailabilityStatus::Unknown,
                    url,
                )
                .with_evidence(format!("network error: {error}"), checked_at_unix_ms));
            }
        };

        Ok(AvailabilityResult::new(target.clone(), name, status, url)
            .with_evidence("public registry lookup completed", checked_at_unix_ms))
    }
}

fn target_url(target: &AvailabilityTarget, name: &str) -> Result<String, AvailabilityError> {
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(AvailabilityError::Provider {
            target: target.code(),
            message: "name must contain only lowercase ASCII letters, digits, or hyphens".into(),
        });
    }

    Ok(match target {
        AvailabilityTarget::GitHub => format!("https://api.github.com/users/{name}"),
        AvailabilityTarget::CratesIo => format!("https://crates.io/api/v1/crates/{name}"),
        AvailabilityTarget::Npm => format!("https://registry.npmjs.org/{name}"),
        AvailabilityTarget::PyPi => format!("https://pypi.org/pypi/{name}/json"),
        AvailabilityTarget::Company => {
            format!("https://api.opencorporates.com/v0.4/companies/search?q={name}")
        }
        AvailabilityTarget::Web => format!(
            "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={name}&srlimit=1&format=json"
        ),
        AvailabilityTarget::Domain { tld } => format!("https://rdap.org/domain/{name}.{tld}"),
    })
}

fn company_match(body: &str, name: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return false;
    };
    value
        .pointer("/results/companies")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|companies| {
            companies.iter().any(|company| {
                company
                    .pointer("/company/name")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|found| found.eq_ignore_ascii_case(name))
            })
        })
}

fn web_match(body: &str, name: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return false;
    };
    value
        .pointer("/query/search")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|results| {
            results.iter().any(|result| {
                result
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|title| title.eq_ignore_ascii_case(name))
            })
        })
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use origin_core::AvailabilityTarget;

    use super::target_url;

    #[test]
    fn target_urls_use_public_evidence_endpoints() {
        assert_eq!(
            target_url(&AvailabilityTarget::GitHub, "qarvan").expect("github URL"),
            "https://api.github.com/users/qarvan"
        );
        assert_eq!(
            target_url(&AvailabilityTarget::Company, "qarvan").expect("company URL"),
            "https://api.opencorporates.com/v0.4/companies/search?q=qarvan"
        );
        assert_eq!(
            target_url(&AvailabilityTarget::Domain { tld: "com".into() }, "qarvan")
                .expect("domain URL"),
            "https://rdap.org/domain/qarvan.com"
        );
    }

    #[test]
    fn target_urls_reject_unsafe_path_components() {
        assert!(target_url(&AvailabilityTarget::Npm, "name/slash").is_err());
    }

    #[test]
    fn exact_name_matchers_do_not_treat_partial_results_as_conflicts() {
        assert!(super::company_match(
            r#"{"results":{"companies":[{"company":{"name":"Qarvan"}}]}}"#,
            "qarvan"
        ));
        assert!(!super::company_match(
            r#"{"results":{"companies":[{"company":{"name":"Qarvan Labs"}}]}}"#,
            "qarvan"
        ));
        assert!(super::web_match(
            r#"{"query":{"search":[{"title":"Qarvan"}]}}"#,
            "qarvan"
        ));
    }
}
