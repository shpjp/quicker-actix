use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;

// ============ DATABASE MODELS ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
    pub banner_image: Option<String>,
    pub followers_count: i32,
    pub following_count: i32,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tweet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub image_url: Option<String>,
    pub likes_count: i32,
    pub retweets_count: i32,
    pub replies_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Like {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tweet_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Follow {
    pub id: Uuid,
    pub follower_id: Uuid,
    pub following_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// Combined struct for JOIN queries
#[derive(Debug, FromRow)]
pub struct TweetWithUser {
    // Tweet fields
    pub id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub image_url: Option<String>,
    pub likes_count: i32,
    pub retweets_count: i32,
    pub replies_count: i32,
    pub created_at: DateTime<Utc>,
    // User fields
    pub user_username: String,
    pub user_display_name: String,
    pub user_email: String,
    pub user_bio: Option<String>,
    pub user_profile_image: Option<String>,
    pub user_banner_image: Option<String>,
    pub user_followers_count: i32,
    pub user_following_count: i32,
    pub user_verified: bool,
    pub user_created_at: DateTime<Utc>,
}

// ============ REQUEST MODELS ============

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 30))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub display_name: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTweetRequest {
    #[validate(length(min = 1, max = 280))]
    pub content: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
    pub banner_image: Option<String>,
}

// ============ RESPONSE MODELS ============

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
    pub banner_image: Option<String>,
    pub followers_count: i32,
    pub following_count: i32,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            bio: user.bio,
            profile_image: user.profile_image,
            banner_image: user.banner_image,
            followers_count: user.followers_count,
            following_count: user.following_count,
            verified: user.verified,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TweetResponse {
    pub id: Uuid,
    pub content: String,
    pub image_url: Option<String>,
    pub likes_count: i32,
    pub retweets_count: i32,
    pub replies_count: i32,
    pub created_at: DateTime<Utc>,
    pub user: UserResponse,
    pub is_liked: bool,
}
