use serde::Deserialize;

/// Result of an update check with different possible outcomes
#[derive(Debug, Clone)]
pub enum UpdateCheckResult {
    /// Update is available
    UpdateAvailable {
        latest_version: String,
        current_version: String,
        release_notes: String,
        download_url: String,
    },
    /// Already up to date
    UpToDate { current_version: String },
    /// Error occurred during check
    Error { message: String },
}

/// GitHub Release API response structure
#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String, // e.g., "v0.2.0"
    #[allow(dead_code)]
    name: String, // e.g., "FM Goal Musics v0.2.0" (unused, but part of API response)
    body: String,     // Markdown changelog
    html_url: String, // URL to release page
}

/// Checks for updates by querying GitHub Releases API
///
/// Returns `UpdateCheckResult` enum with the outcome (update available, up-to-date, or error).
pub fn check_for_updates() -> UpdateCheckResult {
    tracing::info!("[update-checker] Checking for updates...");

    // GitHub API endpoint for latest release
    const API_URL: &str = "https://api.github.com/repos/Mcahitcali/FMGoalMusic/releases/latest";

    // Make HTTP GET request to GitHub API
    let user_agent = format!("FMGoalMusic/{}", env!("CARGO_PKG_VERSION"));
    let response = match ureq::get(API_URL)
        .set("User-Agent", &user_agent) // GitHub requires User-Agent
        .set("Accept", "application/vnd.github+json")
        .timeout(std::time::Duration::from_secs(15)) // 15 second timeout
        .call()
    {
        Ok(resp) => resp,
        Err(_e) => {
            tracing::error!("[update-checker] Network error: {}", _e);
            return UpdateCheckResult::Error {
                message: "Network Error".to_string(),
            };
        }
    };

    // Check if response is successful
    if response.status() != 200 {
        let error_msg = format!("GitHub API returned status {}", response.status());
        tracing::error!("[update-checker] {}", error_msg);
        return UpdateCheckResult::Error { message: error_msg };
    }

    // Parse JSON response
    let release: GithubRelease = match response.into_json() {
        Ok(r) => r,
        Err(e) => {
            let error_msg = format!("Failed to parse response: {}", e);
            tracing::error!("[update-checker] {}", error_msg);
            return UpdateCheckResult::Error { message: error_msg };
        }
    };

    tracing::info!("[update-checker] Latest release: {}", release.tag_name);

    // Parse versions (remove 'v' prefix if present)
    let latest_version_str = release.tag_name.trim_start_matches('v');
    let current_version_str = env!("CARGO_PKG_VERSION");

    // Compare versions using semver
    let (latest_version, current_version) = match (
        semver::Version::parse(latest_version_str),
        semver::Version::parse(current_version_str),
    ) {
        (Ok(latest), Ok(current)) => (latest, current),
        (Err(e), _) | (_, Err(e)) => {
            let error_msg = format!("Version parsing error: {}", e);
            tracing::error!("[update-checker] {}", error_msg);
            return UpdateCheckResult::Error { message: error_msg };
        }
    };

    // Only check for major/minor version changes, ignore patch versions
    let has_major_minor_update = latest_version.major > current_version.major
        || (latest_version.major == current_version.major
            && latest_version.minor > current_version.minor);

    if has_major_minor_update {
        tracing::info!(
            "[update-checker] Update available: {} -> {} (major/minor change)",
            current_version_str,
            latest_version_str
        );
        UpdateCheckResult::UpdateAvailable {
            latest_version: latest_version_str.to_string(),
            current_version: current_version_str.to_string(),
            release_notes: release.body,
            download_url: release.html_url,
        }
    } else {
        tracing::info!(
            "[update-checker] App is up to date for major/minor versions ({})",
            current_version_str
        );
        UpdateCheckResult::UpToDate {
            current_version: current_version_str.to_string(),
        }
    }
}

/// Checks if a specific version should be skipped based on user preference
pub fn should_skip_version(latest_version: &str, skipped_version: &Option<String>) -> bool {
    if let Some(skipped) = skipped_version {
        if skipped == latest_version {
            tracing::info!(
                "[update-checker] Skipping version {} (user preference)",
                latest_version
            );
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_skip_version() {
        // Test skipping a specific version
        let skipped = Some("0.2.0".to_string());
        assert!(should_skip_version("0.2.0", &skipped));
        assert!(!should_skip_version("0.3.0", &skipped));

        // Test with no skipped version
        let no_skip = None;
        assert!(!should_skip_version("0.2.0", &no_skip));
    }

    #[test]
    fn test_version_parsing() {
        // Test that version parsing works correctly
        let v1 = semver::Version::parse("0.1.0").unwrap();
        let v2 = semver::Version::parse("0.2.0").unwrap();
        let v3 = semver::Version::parse("0.1.5").unwrap();
        let v4 = semver::Version::parse("0.2.3").unwrap();
        let v5 = semver::Version::parse("0.2.4").unwrap();

        assert!(v2 > v1); // major/minor change
        assert!(v3 > v1); // major/minor change (0.1.5 > 0.1.0)
        assert!(v2 > v3); // major/minor change (0.2.0 > 0.1.5)
        assert!(v5 > v4); // patch change only

        // Test our major/minor logic
        let has_major_minor_1_to_2 =
            v2.major > v1.major || (v2.major == v1.major && v2.minor > v1.minor);
        let has_major_minor_3_to_4 =
            v4.major > v3.major || (v4.major == v3.major && v4.minor > v3.minor);
        let has_major_minor_4_to_5 =
            v5.major > v4.major || (v5.major == v4.major && v5.minor > v4.minor);

        assert!(has_major_minor_1_to_2); // 0.1.x to 0.2.x should notify
        assert!(has_major_minor_3_to_4); // 0.1.x to 0.2.x should notify
        assert!(!has_major_minor_4_to_5); // 0.2.3 to 0.2.4 should NOT notify
    }
}
