use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RiddleError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Riddle not found: {0}")]
    RiddleNotFound(u64),
    #[error("Invalid answer")]
    InvalidAnswer,
    #[error("Already solved")]
    AlreadySolved,
    #[error("Not available yet")]
    NotAvailable,
    #[error("Time limit exceeded")]
    TimeExpired,
    #[error("Maximum attempts exceeded")]
    MaxAttemptsExceeded,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiddleType {
    Programming,
    Logic,
    Math,
    Cryptography,
    Steganography,
    Reverse,
    Web,
    Binary,
    Forensics,
    Social,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiddleDifficulty {
    Beginner,
    Easy,
    Medium,
    Hard,
    Expert,
    Insane,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiddleStatus {
    Draft,
    Active,
    Inactive,
    Archived,
    UnderReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SolveStatus {
    NotAttempted,
    InProgress,
    Solved,
    Failed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Riddle {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub content: String,
    pub riddle_type: RiddleType,
    pub difficulty: RiddleDifficulty,
    pub status: RiddleStatus,
    pub points: u32,
    pub time_limit: Option<Duration>,
    pub max_attempts: Option<u32>,
    pub answer_hash: String,
    pub hints: Vec<RiddleHint>,
    pub tags: Vec<String>,
    pub author_id: u64,
    pub author_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub solve_count: u64,
    pub attempt_count: u64,
    pub average_solve_time: Option<Duration>,
    pub is_public: bool,
    pub requires_login: bool,
    pub unlock_requirements: Vec<u64>, // Required riddle IDs to unlock this one
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiddleHint {
    pub id: u64,
    pub riddle_id: u64,
    pub content: String,
    pub cost: u32, // Points cost to unlock hint
    pub order: u32,
    pub is_free: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRiddleProgress {
    pub id: u64,
    pub user_id: u64,
    pub riddle_id: u64,
    pub status: SolveStatus,
    pub attempts: u32,
    pub started_at: DateTime<Utc>,
    pub solved_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub time_spent: Duration,
    pub hints_used: Vec<u64>,
    pub points_earned: u32,
    pub solution_submitted: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiddleAttempt {
    pub id: u64,
    pub user_id: u64,
    pub riddle_id: u64,
    pub answer: String,
    pub is_correct: bool,
    pub submitted_at: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
    pub time_taken: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiddleLeaderboard {
    pub riddle_id: u64,
    pub entries: Vec<LeaderboardEntry>,
    pub total_solvers: u64,
    pub fastest_solve_time: Option<Duration>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub user_id: u64,
    pub username: String,
    pub solve_time: Duration,
    pub solved_at: DateTime<Utc>,
    pub rank: u32,
    pub points_earned: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiddleStats {
    pub total_riddles: u64,
    pub solved_riddles: u64,
    pub total_points: u32,
    pub average_solve_time: Option<Duration>,
    pub fastest_solve: Option<Duration>,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub favorite_category: Option<RiddleType>,
    pub difficulty_breakdown: HashMap<RiddleDifficulty, u32>,
}

/// Riddle/Challenge management system ported from PHP Riddle class
/// Handles programming challenges, logic puzzles, and CTF-style problems
pub struct RiddleManager {
    current_user_id: Option<u64>,
}

impl RiddleManager {
    /// Create new RiddleManager instance
    pub fn new(current_user_id: Option<u64>) -> Self {
        Self { current_user_id }
    }

    /// Get riddle by ID
    pub fn get_riddle(&self, riddle_id: u64) -> Result<Riddle, RiddleError> {
        if riddle_id == 0 {
            return Err(RiddleError::RiddleNotFound(riddle_id));
        }

        // Mock riddle for testing
        let riddle = Riddle {
            id: riddle_id,
            title: "Binary Challenge".to_string(),
            description: "Convert the binary number to decimal".to_string(),
            content: "Binary: 1010110".to_string(),
            riddle_type: RiddleType::Programming,
            difficulty: RiddleDifficulty::Easy,
            status: RiddleStatus::Active,
            points: 100,
            time_limit: Some(Duration::minutes(30)),
            max_attempts: Some(3),
            answer_hash: self.hash_answer("86"), // Answer is 86
            hints: vec![
                RiddleHint {
                    id: 1,
                    riddle_id,
                    content: "Start from the rightmost digit".to_string(),
                    cost: 10,
                    order: 1,
                    is_free: true,
                    created_at: Utc::now(),
                },
            ],
            tags: vec!["binary".to_string(), "math".to_string()],
            author_id: 1,
            author_name: "Admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            solve_count: 42,
            attempt_count: 156,
            average_solve_time: Some(Duration::minutes(15)),
            is_public: true,
            requires_login: true,
            unlock_requirements: vec![],
        };

        // Check if user can access this riddle
        if !self.can_access_riddle(&riddle)? {
            return Err(RiddleError::PermissionDenied);
        }

        Ok(riddle)
    }

    /// Get multiple riddles with filtering
    pub fn get_riddles(
        &self,
        riddle_type: Option<RiddleType>,
        difficulty: Option<RiddleDifficulty>,
        status: Option<RiddleStatus>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Riddle>, RiddleError> {
        let limit = if limit > 100 { 100 } else { limit };
        
        // Simulate database query with filtering
        let riddles = vec![]; // Would be populated from database

        Ok(riddles)
    }

    /// Get user's progress on a riddle
    pub fn get_user_progress(&self, riddle_id: u64, user_id: u64) -> Result<Option<UserRiddleProgress>, RiddleError> {
        // Check permissions
        if let Some(current_user) = self.current_user_id {
            if current_user != user_id {
                return Err(RiddleError::PermissionDenied);
            }
        } else {
            return Err(RiddleError::PermissionDenied);
        }

        // Simulate database lookup
        Ok(None) // No progress found
    }

    /// Start working on a riddle
    pub fn start_riddle(&self, riddle_id: u64) -> Result<UserRiddleProgress, RiddleError> {
        let user_id = self.current_user_id.ok_or(RiddleError::PermissionDenied)?;
        let riddle = self.get_riddle(riddle_id)?;

        // Check if riddle is available
        if !matches!(riddle.status, RiddleStatus::Active) {
            return Err(RiddleError::NotAvailable);
        }

        // Check unlock requirements
        if !self.check_unlock_requirements(&riddle, user_id)? {
            return Err(RiddleError::PermissionDenied);
        }

        // Check if already started or solved
        if let Some(existing_progress) = self.get_user_progress(riddle_id, user_id)? {
            match existing_progress.status {
                SolveStatus::Solved => return Err(RiddleError::AlreadySolved),
                SolveStatus::InProgress | SolveStatus::NotAttempted => return Ok(existing_progress),
                _ => {}
            }
        }

        // Create new progress
        let progress = UserRiddleProgress {
            id: self.generate_id(),
            user_id,
            riddle_id,
            status: SolveStatus::InProgress,
            attempts: 0,
            started_at: Utc::now(),
            solved_at: None,
            last_attempt_at: None,
            time_spent: Duration::zero(),
            hints_used: vec![],
            points_earned: 0,
            solution_submitted: None,
        };

        // Save progress to database
        Ok(progress)
    }

    /// Submit answer for a riddle
    pub fn submit_answer(
        &self,
        riddle_id: u64,
        answer: String,
        ip_address: String,
        user_agent: String,
    ) -> Result<(bool, UserRiddleProgress), RiddleError> {
        let user_id = self.current_user_id.ok_or(RiddleError::PermissionDenied)?;
        let riddle = self.get_riddle(riddle_id)?;
        
        // Get or create progress
        let mut progress = self.get_user_progress(riddle_id, user_id)?
            .unwrap_or_else(|| {
                // Auto-start if not started
                self.start_riddle(riddle_id).unwrap()
            });

        // Check if already solved
        if matches!(progress.status, SolveStatus::Solved) {
            return Err(RiddleError::AlreadySolved);
        }

        // Check max attempts
        if let Some(max_attempts) = riddle.max_attempts {
            if progress.attempts >= max_attempts {
                return Err(RiddleError::MaxAttemptsExceeded);
            }
        }

        // Check time limit
        if let Some(time_limit) = riddle.time_limit {
            let elapsed = Utc::now() - progress.started_at;
            if elapsed > time_limit {
                progress.status = SolveStatus::Expired;
                return Err(RiddleError::TimeExpired);
            }
        }

        // Validate answer
        let answer_trimmed = answer.trim();
        if answer_trimmed.is_empty() {
            return Err(RiddleError::Validation("Answer cannot be empty".to_string()));
        }

        // Check answer
        let is_correct = self.verify_answer(&riddle, answer_trimmed);
        progress.attempts += 1;
        progress.last_attempt_at = Some(Utc::now());
        progress.solution_submitted = Some(answer_trimmed.to_string());

        // Record attempt
        let attempt = RiddleAttempt {
            id: self.generate_id(),
            user_id,
            riddle_id,
            answer: answer_trimmed.to_string(),
            is_correct,
            submitted_at: Utc::now(),
            ip_address,
            user_agent,
            time_taken: Utc::now() - progress.started_at,
        };

        if is_correct {
            // Correct answer - mark as solved
            progress.status = SolveStatus::Solved;
            progress.solved_at = Some(Utc::now());
            progress.time_spent = Utc::now() - progress.started_at;
            
            // Calculate points (reduced by hints used)
            let hint_penalty = progress.hints_used.iter()
                .map(|hint_id| self.get_hint_cost(*hint_id).unwrap_or(0))
                .sum::<u32>();
            progress.points_earned = riddle.points.saturating_sub(hint_penalty);

            // Award points to user
            self.award_points(user_id, progress.points_earned)?;
        } else {
            // Wrong answer
            if let Some(max_attempts) = riddle.max_attempts {
                if progress.attempts >= max_attempts {
                    progress.status = SolveStatus::Failed;
                }
            }
        }

        // Save progress and attempt
        self.save_progress(&progress)?;
        self.save_attempt(&attempt)?;

        Ok((is_correct, progress))
    }

    /// Get hint for a riddle
    pub fn get_hint(&self, riddle_id: u64, hint_id: u64) -> Result<RiddleHint, RiddleError> {
        let user_id = self.current_user_id.ok_or(RiddleError::PermissionDenied)?;
        let riddle = self.get_riddle(riddle_id)?;
        
        // Find hint
        let hint = riddle.hints.iter()
            .find(|h| h.id == hint_id)
            .ok_or(RiddleError::RiddleNotFound(hint_id))?;

        // Check if user can access this hint
        let mut progress = self.get_user_progress(riddle_id, user_id)?
            .ok_or(RiddleError::NotAvailable)?;

        // Check if already solved
        if matches!(progress.status, SolveStatus::Solved) {
            return Ok(hint.clone());
        }

        // Check if hint already used
        if progress.hints_used.contains(&hint_id) {
            return Ok(hint.clone());
        }

        // Check if user can afford hint
        if !hint.is_free {
            let user_points = self.get_user_points(user_id)?;
            if user_points < hint.cost {
                return Err(RiddleError::PermissionDenied);
            }
        }

        // Deduct points and mark hint as used
        if !hint.is_free {
            self.deduct_points(user_id, hint.cost)?;
        }
        
        progress.hints_used.push(hint_id);
        self.save_progress(&progress)?;

        Ok(hint.clone())
    }

    /// Get riddle leaderboard
    pub fn get_leaderboard(&self, riddle_id: u64, limit: u32) -> Result<RiddleLeaderboard, RiddleError> {
        let limit = if limit > 100 { 100 } else { limit };
        
        // Simulate database query for leaderboard
        let leaderboard = RiddleLeaderboard {
            riddle_id,
            entries: vec![], // Would be populated from database
            total_solvers: 42,
            fastest_solve_time: Some(Duration::minutes(5)),
            updated_at: Utc::now(),
        };

        Ok(leaderboard)
    }

    /// Get user's riddle statistics
    pub fn get_user_stats(&self, user_id: u64) -> Result<RiddleStats, RiddleError> {
        // Simulate database aggregation
        let stats = RiddleStats {
            total_riddles: 100,
            solved_riddles: 25,
            total_points: 2500,
            average_solve_time: Some(Duration::minutes(20)),
            fastest_solve: Some(Duration::minutes(5)),
            current_streak: 3,
            longest_streak: 8,
            favorite_category: Some(RiddleType::Programming),
            difficulty_breakdown: {
                let mut breakdown = HashMap::new();
                breakdown.insert(RiddleDifficulty::Easy, 10);
                breakdown.insert(RiddleDifficulty::Medium, 8);
                breakdown.insert(RiddleDifficulty::Hard, 5);
                breakdown.insert(RiddleDifficulty::Expert, 2);
                breakdown
            },
        };

        Ok(stats)
    }

    /// Create new riddle
    pub fn create_riddle(&self, riddle: Riddle) -> Result<Riddle, RiddleError> {
        // Check permissions (admin/moderator only)
        if self.current_user_id.is_none() {
            return Err(RiddleError::PermissionDenied);
        }

        // Validate riddle
        if riddle.title.trim().is_empty() {
            return Err(RiddleError::Validation("Title cannot be empty".to_string()));
        }

        if riddle.content.trim().is_empty() {
            return Err(RiddleError::Validation("Content cannot be empty".to_string()));
        }

        if riddle.answer_hash.is_empty() {
            return Err(RiddleError::Validation("Answer hash cannot be empty".to_string()));
        }

        // Save riddle to database
        Ok(riddle)
    }

    /// Search riddles
    pub fn search_riddles(&self, query: &str, limit: u32) -> Result<Vec<Riddle>, RiddleError> {
        if query.trim().len() < 3 {
            return Err(RiddleError::Validation("Search query must be at least 3 characters".to_string()));
        }

        let limit = if limit > 50 { 50 } else { limit };
        
        // Simulate search functionality
        let riddles = vec![]; // Would be populated from database search

        Ok(riddles)
    }

    /// Helper methods
    fn can_access_riddle(&self, riddle: &Riddle) -> Result<bool, RiddleError> {
        // Check if riddle is public or user is logged in
        if !riddle.is_public || (riddle.requires_login && self.current_user_id.is_none()) {
            return Ok(false);
        }

        // Check if riddle is active
        if !matches!(riddle.status, RiddleStatus::Active) {
            return Ok(false);
        }

        Ok(true)
    }

    fn check_unlock_requirements(&self, riddle: &Riddle, user_id: u64) -> Result<bool, RiddleError> {
        if riddle.unlock_requirements.is_empty() {
            return Ok(true);
        }

        // Check if user has solved all required riddles
        for required_riddle_id in &riddle.unlock_requirements {
            if let Some(progress) = self.get_user_progress(*required_riddle_id, user_id)? {
                if !matches!(progress.status, SolveStatus::Solved) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn verify_answer(&self, riddle: &Riddle, answer: &str) -> bool {
        let answer_hash = self.hash_answer(answer);
        answer_hash == riddle.answer_hash
    }

    fn hash_answer(&self, answer: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        answer.to_lowercase().trim().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn get_hint_cost(&self, hint_id: u64) -> Result<u32, RiddleError> {
        // Simulate database lookup for hint cost
        Ok(10) // Default cost
    }

    fn get_user_points(&self, user_id: u64) -> Result<u32, RiddleError> {
        // Simulate database lookup for user points
        Ok(1000) // Mock points
    }

    fn award_points(&self, user_id: u64, points: u32) -> Result<(), RiddleError> {
        // Update user's total points
        Ok(())
    }

    fn deduct_points(&self, user_id: u64, points: u32) -> Result<(), RiddleError> {
        // Deduct points from user's total
        Ok(())
    }

    fn save_progress(&self, progress: &UserRiddleProgress) -> Result<(), RiddleError> {
        // Save progress to database
        Ok(())
    }

    fn save_attempt(&self, attempt: &RiddleAttempt) -> Result<(), RiddleError> {
        // Save attempt to database
        Ok(())
    }

    fn generate_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_riddle_manager_creation() {
        let manager = RiddleManager::new(Some(1));
        assert_eq!(manager.current_user_id, Some(1));
    }

    #[test]
    fn test_get_riddle() {
        let manager = RiddleManager::new(Some(1));
        let result = manager.get_riddle(1);
        assert!(result.is_ok());

        let riddle = result.unwrap();
        assert_eq!(riddle.id, 1);
        assert_eq!(riddle.title, "Binary Challenge");
        assert!(matches!(riddle.riddle_type, RiddleType::Programming));
    }

    #[test]
    fn test_invalid_riddle_id() {
        let manager = RiddleManager::new(Some(1));
        let result = manager.get_riddle(0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RiddleError::RiddleNotFound(0)));
    }

    #[test]
    fn test_answer_hashing() {
        let manager = RiddleManager::new(Some(1));
        let hash1 = manager.hash_answer("86");
        let hash2 = manager.hash_answer("86");
        let hash3 = manager.hash_answer("87");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_submit_answer_without_permission() {
        let manager = RiddleManager::new(None);
        let result = manager.submit_answer(1, "86".to_string(), "127.0.0.1".to_string(), "test".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RiddleError::PermissionDenied));
    }

    #[test]
    fn test_riddle_difficulty_ordering() {
        use RiddleDifficulty::*;
        
        let difficulties = vec![Expert, Beginner, Hard, Easy, Medium, Insane];
        let mut sorted = difficulties.clone();
        sorted.sort();
        
        // Enum variants are ordered by declaration order
        assert_eq!(sorted, vec![Beginner, Easy, Medium, Hard, Expert, Insane]);
    }
}