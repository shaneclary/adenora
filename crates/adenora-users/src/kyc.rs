use adenora_common::error::AdenoraError;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycSubmission {
    pub full_name: String,
    pub date_of_birth: NaiveDate,
    pub country_code: String,
    pub document_type: DocumentType,
    pub document_number: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Passport,
    NationalId,
    DrivingLicense,
    EId, // Estonian/Latvian/Lithuanian eID
}

pub fn verify_age(dob: NaiveDate, min_age: u8) -> Result<(), AdenoraError> {
    let today = Utc::now().date_naive();
    let age = today.years_since(dob);

    match age {
        Some(years) if years >= min_age as u32 => Ok(()),
        _ => Err(AdenoraError::AgeVerificationRequired),
    }
}
