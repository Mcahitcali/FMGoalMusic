use serde::Deserialize;
use std::error::Error;

/// Result of an update check
#[derive(Debug, Clone)]
pub struct UpdateCheckResult {
    pub update_available: bool,
    pub latest_version: String,
    pub current_version: String,
    pub release_notes: String,
    pub download_url: String,
}

/// GitHub Release API response structure
#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,      // e.g., "v0.2.0"
    name: String,          // e.g., "FM Goal Musics v0.2.0"
    body: String,          // Markdown changelog
    html_url: String,      // URL to release page
}

/// Checks for updates by querying GitHub Releases API
///
/// Returns `UpdateCheckResult` with information about the latest version.
/// If no update is available, `update_available` will be false.
pub fn check_for_updates() -> Result<UpdateCheckResult, Box<dyn Error>> {
    log::info!("[update-checker] Checking for updates...");

    // GitHub API endpoint for latest release
    const API_URL: &str = "https://api.github.com/repos/Mcahitcali/FMGoalMusic/releases/latest";

    // Make HTTP GET request to GitHub API
    let response = ureq::get(API_URL)
        .set("User-Agent", "FMGoalMusic/0.1.0")  // GitHub requires User-Agent
        .set("Accept", "application/vnd.github+json")
        .timeout(std::time::Duration::from_secs(10))  // 10 second timeout
        .call()?;

    // Check if response is successful
    if response.status() != 200 {
        let error_msg = format!("GitHub API returned status {}", response.status());
        log::error!("[update-checker] {}", error_msg);
        return Err(error_msg.into());
    }

    // Parse JSON response
    let release: GithubRelease = response.into_json()?;
    log::info!("[update-checker] Latest release: {}", release.tag_name);

    // Parse versions (remove 'v' prefix if present)
    let latest_version_str = release.tag_name.trim_start_matches('v');
    let current_version_str = env!("CARGO_PKG_VERSION");

    // Compare versions using semver
    let latest_version = semver::Version::parse(latest_version_str)?;
    let current_version = semver::Version::parse(current_version_str)?;

    let update_available = latest_version > current_version;

    if update_available {
        log::info!(
            "[update-checker] Update available: {} -> {}",
            current_version_str,
            latest_version_str
        );
    } else {
        log::info!("[update-checker] App is up to date ({})", current_version_str);
    }

    Ok(UpdateCheckResult {
        update_available,
        latest_version: latest_version_str.to_string(),
        current_version: current_version_str.to_string(),
        release_notes: release.body,
        download_url: release.html_url,
    })
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
