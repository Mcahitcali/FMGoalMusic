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
    UpToDate {
        current_version: String,
    },
    /// Error occurred during check
    Error {
        message: String,
    },
}

/// GitHub Release API response structure
#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,      // e.g., "v0.2.0"
    #[allow(dead_code)]
    name: String,          // e.g., "FM Goal Musics v0.2.0" (unused, but part of API response)
    body: String,          // Markdown changelog
    html_url: String,      // URL to release page
}

/// Checks for updates by querying GitHub Releases API
///
/// Returns `UpdateCheckResult` enum with the outcome (update available, up-to-date, or error).
pub fn check_for_updates() -> UpdateCheckResult {
    log::info!("[update-checker] Checking for updates...");

    // GitHub API endpoint for latest release
    const API_URL: &str = "https://api.github.com/repos/Mcahitcali/FMGoalMusic/releases/latest";

    // Make HTTP GET request to GitHub API
    let user_agent = format!("FMGoalMusic/{}", env!("CARGO_PKG_VERSION"));
    let response = match ureq::get(API_URL)
        .set("User-Agent", &user_agent)  // GitHub requires User-Agent
        .set("Accept", "application/vnd.github+json")
        .timeout(std::time::Duration::from_secs(10))  // 10 second timeout
        .call()
    {
        Ok(resp) => resp,
        Err(e) => {
            let error_msg = format!("Network error: {}", e);
            log::error!("[update-checker] {}", error_msg);
            return UpdateCheckResult::Error { message: error_msg };
        }
    };

    // Check if response is successful
    if response.status() != 200 {
        let error_msg = format!("GitHub API returned status {}", response.status());
        log::error!("[update-checker] {}", error_msg);
        return UpdateCheckResult::Error { message: error_msg };
    }

    // Parse JSON response
    let release: GithubRelease = match response.into_json() {
        Ok(r) => r,
        Err(e) => {
            let error_msg = format!("Failed to parse response: {}", e);
            log::error!("[update-checker] {}", error_msg);
            return UpdateCheckResult::Error { message: error_msg };
        }
    };

    log::info!("[update-checker] Latest release: {}", release.tag_name);

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
            log::error!("[update-checker] {}", error_msg);
            return UpdateCheckResult::Error { message: error_msg };
        }
    };

    if latest_version > current_version {
        log::info!(
            "[update-checker] Update available: {} -> {}",
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
        log::info!("[update-checker] App is up to date ({})", current_version_str);
        UpdateCheckResult::UpToDate {
            current_version: current_version_str.to_string(),
        }
    }
}

/// Checks if a specific version should be skipped based on user preference
pub fn should_skip_version(latest_version: &str, skipped_version: &Option<String>) -> bool {
    if let Some(skipped) = skipped_version {
        if skipped == latest_version {
            log::info!("[update-checker] Skipping version {} (user preference)", latest_version);
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

        assert!(v2 > v1);
        assert!(v3 > v1);
        assert!(v2 > v3);
    }
}
