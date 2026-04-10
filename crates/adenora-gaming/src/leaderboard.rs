use adenora_common::types::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub user_id: UserId,
    pub display_name: String,
    pub score: i64,
    pub rank: u32,
    pub games_played: u32,
    pub tournaments_won: u32,
    pub prediction_accuracy: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardScope {
    AllTime,
    Season,
    Monthly,
    Weekly,
    Daily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardCategory {
    Overall,
    Predictions,
    Gaming,
    BotBattle,
    Trivia,
    DonateAndPlay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leaderboard {
    pub scope: LeaderboardScope,
    pub category: LeaderboardCategory,
    pub entries: Vec<LeaderboardEntry>,
    pub total_participants: u32,
    pub generated_at: DateTime<Utc>,
}
