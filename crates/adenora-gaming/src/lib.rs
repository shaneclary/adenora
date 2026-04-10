pub mod achievements;
pub mod forecast;
pub mod leaderboard;
pub mod tournament;
pub mod trivia;

use adenora_common::types::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A skill-based game type available on the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: GameId,
    pub name: String,
    pub description: String,
    pub game_type: SkillGameType,
    pub min_players: u32,
    pub max_players: u32,
    pub is_free_to_play: bool,
    pub entry_fee: Option<Decimal>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillGameType {
    /// Trivia quiz with prediction-related questions
    Trivia,
    /// Forecast accuracy competition
    Forecasting,
    /// Strategy/simulation game
    Strategy,
    /// Speed-based prediction challenge
    SpeedPredict,
}

/// A game result for one player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResult {
    pub id: Uuid,
    pub game_id: GameId,
    pub user_id: UserId,
    pub score: i64,
    pub rank: u32,
    pub prize_amount: Option<Decimal>,
    pub played_at: DateTime<Utc>,
}

/// An achievement a user can unlock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub criteria: AchievementCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AchievementCriteria {
    GamesPlayed(u32),
    TournamentsWon(u32),
    PredictionStreak(u32),
    TotalScore(i64),
    FirstPrediction,
    FirstWin,
    DonateAndPlay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAchievement {
    pub user_id: UserId,
    pub achievement_id: Uuid,
    pub unlocked_at: DateTime<Utc>,
}
