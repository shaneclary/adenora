use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AdenoraError {
    // Auth
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("token expired")]
    TokenExpired,
    #[error("insufficient permissions")]
    InsufficientPermissions,
    #[error("account suspended")]
    AccountSuspended,
    #[error("age verification required (21+)")]
    AgeVerificationRequired,

    // Trading
    #[error("insufficient balance: need {need}, have {have}")]
    InsufficientBalance { need: String, have: String },
    #[error("market {0} is not active")]
    MarketNotActive(Uuid),
    #[error("market {0} not found")]
    MarketNotFound(Uuid),
    #[error("order {0} not found")]
    OrderNotFound(Uuid),
    #[error("invalid price: {0} (must be 0.01-0.99)")]
    InvalidPrice(String),
    #[error("invalid quantity: {0}")]
    InvalidQuantity(String),
    #[error("order would self-trade")]
    SelfTrade,

    // Bot arena
    #[error("bot {0} not registered")]
    BotNotRegistered(Uuid),
    #[error("bot name already taken: {0}")]
    BotNameTaken(String),
    #[error("cannot enter people-mode market with bot account")]
    BotInPeopleMode,
    #[error("human verification required for people-mode market")]
    HumanVerificationRequired,

    // Oracle
    #[error("dispute period has ended for market {0}")]
    DisputePeriodEnded(Uuid),
    #[error("insufficient oracle sources: need {need}, have {have}")]
    InsufficientOracleSources { need: u32, have: u32 },
    #[error("invalid jury vote: {0}")]
    InvalidJuryVote(String),

    // Lottery
    #[error("lottery {0} not found")]
    LotteryNotFound(Uuid),
    #[error("draw {0} already completed")]
    DrawAlreadyCompleted(Uuid),
    #[error("ticket purchase limit exceeded")]
    TicketLimitExceeded,

    // Responsible gambling
    #[error("deposit limit exceeded: {0}")]
    DepositLimitExceeded(String),
    #[error("self-exclusion active until {0}")]
    SelfExclusionActive(String),
    #[error("cooling-off period active")]
    CoolingOffActive,

    // Infrastructure
    #[error("database error: {0}")]
    Database(String),
    #[error("rate limit exceeded")]
    RateLimitExceeded,
    #[error("internal error: {0}")]
    Internal(String),
}

impl AdenoraError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::InvalidCredentials | Self::TokenExpired => 401,
            Self::InsufficientPermissions | Self::AccountSuspended => 403,
            Self::MarketNotFound(_) | Self::OrderNotFound(_)
            | Self::LotteryNotFound(_) | Self::BotNotRegistered(_) => 404,
            Self::SelfTrade | Self::BotInPeopleMode
            | Self::HumanVerificationRequired | Self::BotNameTaken(_) => 409,
            Self::RateLimitExceeded => 429,
            Self::InsufficientBalance { .. } | Self::InvalidPrice(_)
            | Self::InvalidQuantity(_) | Self::MarketNotActive(_)
            | Self::DisputePeriodEnded(_) | Self::InsufficientOracleSources { .. }
            | Self::DrawAlreadyCompleted(_) | Self::TicketLimitExceeded
            | Self::DepositLimitExceeded(_) | Self::SelfExclusionActive(_)
            | Self::CoolingOffActive | Self::AgeVerificationRequired
            | Self::InvalidJuryVote(_) => 400,
            Self::Database(_) | Self::Internal(_) => 500,
        }
    }
}
