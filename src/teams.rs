use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Represents a single team with display name and name variations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub display_name: String,
    pub variations: Vec<String>,
}

/// Database of all teams organized by league
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDatabase {
    #[serde(flatten)]
    leagues: HashMap<String, HashMap<String, Team>>,
}

impl TeamDatabase {
    /// Load team database from JSON file
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = Self::database_path()?;

        if !db_path.exists() {
            // First run: copy embedded database to user config directory
            println!("[teams] teams.json not found in user config directory, creating from embedded default");
            let database = Self::load_embedded()?;
            database.save()?;
            return Ok(database);
        }

        // Load from user config directory
        let content = fs::read_to_string(&db_path)?;
        let database: TeamDatabase = serde_json::from_str(&content)?;
        Ok(database)
    }

    /// Load embedded default database (fallback)
    fn load_embedded() -> Result<Self, Box<dyn std::error::Error>> {
        const EMBEDDED_DB: &str = include_str!("../config/teams.json");
        let database: TeamDatabase = serde_json::from_str(EMBEDDED_DB)?;
        Ok(database)
    }

    /// Save team database to user config directory
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let db_path = Self::database_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&db_path, json)?;

        println!("[teams] Saved teams database to: {}", db_path.display());
        Ok(())
    }

    /// Get the path to the teams database file in user-writable config directory
    pub fn database_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let base = dirs::config_dir()
            .ok_or("Could not determine user config directory")?;
        // Application-specific folder
        let app_dir = base.join("FMGoalMusic");
        // Ensure directory exists
        fs::create_dir_all(&app_dir)?;
        Ok(app_dir.join("teams.json"))
    }

    /// Get the teams database path for display purposes
    pub fn database_path_display() -> String {
        Self::database_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }

    /// Get list of all league names
    pub fn get_leagues(&self) -> Vec<String> {
        let mut leagues: Vec<String> = self.leagues.keys().cloned().collect();
        leagues.sort();
        leagues
    }

    /// Get all teams in a specific league
    pub fn get_teams(&self, league: &str) -> Option<Vec<(String, Team)>> {
        self.leagues.get(league).map(|teams| {
            let mut team_list: Vec<(String, Team)> = teams
                .iter()
                .map(|(key, team)| (key.clone(), team.clone()))
                .collect();
            team_list.sort_by(|a, b| a.1.display_name.cmp(&b.1.display_name));
            team_list
        })
    }

    /// Find a specific team by league and team key
    pub fn find_team(&self, league: &str, team_key: &str) -> Option<Team> {
        self.leagues
            .get(league)
            .and_then(|teams| teams.get(team_key))
            .cloned()
    }

    /// Search for a team across all leagues by display name or variation
    pub fn search_team(&self, query: &str) -> Vec<(String, String, Team)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for (league_name, teams) in &self.leagues {
            for (team_key, team) in teams {
                // Check display name
                if team.display_name.to_lowercase().contains(&query_lower) {
                    results.push((league_name.clone(), team_key.clone(), team.clone()));
                    continue;
                }

                // Check variations
                if team
                    .variations
                    .iter()
                    .any(|v| v.to_lowercase().contains(&query_lower))
                {
                    results.push((league_name.clone(), team_key.clone(), team.clone()));
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_embedded() {
        let db = TeamDatabase::load_embedded();
        assert!(db.is_ok());
        let db = db.unwrap();
        assert!(!db.get_leagues().is_empty());
    }

    #[test]
    fn test_get_leagues() {
        let db = TeamDatabase::load_embedded().unwrap();
        let leagues = db.get_leagues();
        assert!(leagues.contains(&"Premier League".to_string()));
        assert!(leagues.contains(&"La Liga".to_string()));
    }

    #[test]
    fn test_get_teams() {
        let db = TeamDatabase::load_embedded().unwrap();
        let teams = db.get_teams("Premier League");
        assert!(teams.is_some());
        let teams = teams.unwrap();
        assert!(!teams.is_empty());
    }

    #[test]
    fn test_find_team() {
        let db = TeamDatabase::load_embedded().unwrap();
        let team = db.find_team("Premier League", "manchester_united");
        assert!(team.is_some());
        let team = team.unwrap();
        assert_eq!(team.display_name, "Manchester Utd");
        assert!(!team.variations.is_empty());
    }

    #[test]
    fn test_search_team() {
        let db = TeamDatabase::load_embedded().unwrap();
        let results = db.search_team("manchester");
        assert!(!results.is_empty());
        assert!(results.iter().any(|(_, _, t)| t
            .display_name
            .to_lowercase()
            .contains("manchester")));
    }

    #[test]
    fn test_search_team_case_insensitive() {
        let db = TeamDatabase::load_embedded().unwrap();
        let results = db.search_team("LIVERPOOL");
        assert!(!results.is_empty());
    }
}
