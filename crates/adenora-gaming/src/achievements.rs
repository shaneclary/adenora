use super::{Achievement, AchievementCriteria, UserAchievement};
use chrono::Utc;
use uuid::Uuid;

/// Check if a user has earned any new achievements based on their stats.
pub fn check_achievements(
    definitions: &[Achievement],
    already_unlocked: &[Uuid],
    stats: &UserGameStats,
) -> Vec<UserAchievement> {
    let mut newly_earned = Vec::new();

    for achievement in definitions {
        if already_unlocked.contains(&achievement.id) {
            continue;
        }

        let earned = match &achievement.criteria {
            AchievementCriteria::GamesPlayed(n) => stats.games_played >= *n,
            AchievementCriteria::TournamentsWon(n) => stats.tournaments_won >= *n,
            AchievementCriteria::PredictionStreak(n) => stats.best_prediction_streak >= *n,
            AchievementCriteria::TotalScore(n) => stats.total_score >= *n,
            AchievementCriteria::FirstPrediction => stats.predictions_made >= 1,
            AchievementCriteria::FirstWin => stats.wins >= 1,
            AchievementCriteria::DonateAndPlay => stats.lottery_tickets_bought >= 1,
        };

        if earned {
            newly_earned.push(UserAchievement {
                user_id: stats.user_id,
                achievement_id: achievement.id,
                unlocked_at: Utc::now(),
            });
        }
    }

    newly_earned
}

pub struct UserGameStats {
    pub user_id: Uuid,
    pub games_played: u32,
    pub tournaments_won: u32,
    pub best_prediction_streak: u32,
    pub total_score: i64,
    pub predictions_made: u32,
    pub wins: u32,
    pub lottery_tickets_bought: u32,
}

/// Generate the default achievement definitions.
pub fn default_achievements() -> Vec<Achievement> {
    vec![
        Achievement {
            id: Uuid::new_v4(),
            name: "First Steps".into(),
            description: "Make your first prediction".into(),
            icon: "target".into(),
            criteria: AchievementCriteria::FirstPrediction,
        },
        Achievement {
            id: Uuid::new_v4(),
            name: "Winner".into(),
            description: "Win your first trade".into(),
            icon: "trophy".into(),
            criteria: AchievementCriteria::FirstWin,
        },
        Achievement {
            id: Uuid::new_v4(),
            name: "Philanthropist".into(),
            description: "Buy your first lottery ticket (Donate and Play)".into(),
            icon: "heart".into(),
            criteria: AchievementCriteria::DonateAndPlay,
        },
        Achievement {
            id: Uuid::new_v4(),
            name: "Regular".into(),
            description: "Play 10 games".into(),
            icon: "gamepad".into(),
            criteria: AchievementCriteria::GamesPlayed(10),
        },
        Achievement {
            id: Uuid::new_v4(),
            name: "Veteran".into(),
            description: "Play 100 games".into(),
            icon: "star".into(),
            criteria: AchievementCriteria::GamesPlayed(100),
        },
        Achievement {
            id: Uuid::new_v4(),
            name: "Hot Streak".into(),
            description: "Get 5 correct predictions in a row".into(),
            icon: "flame".into(),
            criteria: AchievementCriteria::PredictionStreak(5),
        },
        Achievement {
            id: Uuid::new_v4(),
            name: "Oracle".into(),
            description: "Get 10 correct predictions in a row".into(),
            icon: "eye".into(),
            criteria: AchievementCriteria::PredictionStreak(10),
        },
        Achievement {
            id: Uuid::new_v4(),
            name: "Champion".into(),
            description: "Win a tournament".into(),
            icon: "crown".into(),
            criteria: AchievementCriteria::TournamentsWon(1),
        },
        Achievement {
            id: Uuid::new_v4(),
            name: "High Scorer".into(),
            description: "Reach 10,000 total points".into(),
            icon: "zap".into(),
            criteria: AchievementCriteria::TotalScore(10_000),
        },
    ]
}
