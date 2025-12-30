use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ============ DATA MODELS ============

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    username: String,
    email: String,
    display_name: String,
    bio: Option<String>,
    followers_count: u32,
    following_count: u32,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Tweet {
    id: String,
    user_id: String,
    content: String,
    likes_count: u32,
    retweets_count: u32,
    replies_count: u32,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Like {
    id: String,
    user_id: String,
    tweet_id: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Follow {
    id: String,
    follower_id: String,
    following_id: String,
    created_at: DateTime<Utc>,
}

// ============ REQUEST/RESPONSE TYPES ============

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
    display_name: String,
    bio: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateTweetRequest {
    user_id: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct LikeTweetRequest {
    user_id: String,
    tweet_id: String,
}

#[derive(Debug, Deserialize)]
struct FollowUserRequest {
    follower_id: String,
    following_id: String,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

// ============ IN-MEMORY STORAGE ============

struct AppState {
    users: Mutex<Vec<User>>,
    tweets: Mutex<Vec<Tweet>>,
    likes: Mutex<Vec<Like>>,
    follows: Mutex<Vec<Follow>>,
}

// ============ API HANDLERS ============

// Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some("Twitter API is running"),
        message: None,
    })
}

// Get all users
async fn get_users(data: web::Data<AppState>) -> impl Responder {
    let users = data.users.lock().unwrap();
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(users.clone()),
        message: None,
    })
}

// Get user by ID
async fn get_user(data: web::Data<AppState>, user_id: web::Path<String>) -> impl Responder {
    let users = data.users.lock().unwrap();
    let user_id = user_id.into_inner();
    match users.iter().find(|u| u.id == user_id) {
        Some(user) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(user.clone()),
            message: None,
        }),
        None => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("User not found".to_string()),
        }),
    }
}

// Create a new user
async fn create_user(
    data: web::Data<AppState>,
    req: web::Json<CreateUserRequest>,
) -> impl Responder {
    let mut users = data.users.lock().unwrap();
    
    // Check if username or email already exists
    if users.iter().any(|u| u.username == req.username || u.email == req.email) {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Username or email already exists".to_string()),
        });
    }

    let user = User {
        id: Uuid::new_v4().to_string(),
        username: req.username.clone(),
        email: req.email.clone(),
        display_name: req.display_name.clone(),
        bio: req.bio.clone(),
        followers_count: 0,
        following_count: 0,
        created_at: Utc::now(),
    };

    users.push(user.clone());
    HttpResponse::Created().json(ApiResponse {
        success: true,
        data: Some(user),
        message: Some("User created successfully".to_string()),
    })
}

// Get all tweets
async fn get_tweets(data: web::Data<AppState>) -> impl Responder {
    let tweets = data.tweets.lock().unwrap();
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(tweets.clone()),
        message: None,
    })
}

// Get tweet by ID
async fn get_tweet(data: web::Data<AppState>, tweet_id: web::Path<String>) -> impl Responder {
    let tweets = data.tweets.lock().unwrap();
    let tweet_id = tweet_id.into_inner();
    match tweets.iter().find(|t| t.id == tweet_id) {
        Some(tweet) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(tweet.clone()),
            message: None,
        }),
        None => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Tweet not found".to_string()),
        }),
    }
}

// Get tweets by user ID
async fn get_user_tweets(data: web::Data<AppState>, user_id: web::Path<String>) -> impl Responder {
    let tweets = data.tweets.lock().unwrap();
    let user_id = user_id.into_inner();
    let user_tweets: Vec<Tweet> = tweets
        .iter()
        .filter(|t| t.user_id == user_id)
        .cloned()
        .collect();
    
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(user_tweets),
        message: None,
    })
}

// Create a new tweet
async fn create_tweet(
    data: web::Data<AppState>,
    req: web::Json<CreateTweetRequest>,
) -> impl Responder {
    let users = data.users.lock().unwrap();
    
    // Verify user exists
    if !users.iter().any(|u| u.id == req.user_id) {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("User not found".to_string()),
        });
    }

    // Validate tweet content
    if req.content.is_empty() || req.content.len() > 280 {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Tweet content must be between 1 and 280 characters".to_string()),
        });
    }

    drop(users);

    let mut tweets = data.tweets.lock().unwrap();
    let tweet = Tweet {
        id: Uuid::new_v4().to_string(),
        user_id: req.user_id.clone(),
        content: req.content.clone(),
        likes_count: 0,
        retweets_count: 0,
        replies_count: 0,
        created_at: Utc::now(),
    };

    tweets.push(tweet.clone());
    HttpResponse::Created().json(ApiResponse {
        success: true,
        data: Some(tweet),
        message: Some("Tweet created successfully".to_string()),
    })
}

// Delete a tweet
async fn delete_tweet(
    data: web::Data<AppState>,
    tweet_id: web::Path<String>,
) -> impl Responder {
    let mut tweets = data.tweets.lock().unwrap();
    let tweet_id = tweet_id.into_inner();
    
    if let Some(pos) = tweets.iter().position(|t| t.id == tweet_id) {
        tweets.remove(pos);
        HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some("Tweet deleted successfully"),
            message: None,
        })
    } else {
        HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Tweet not found".to_string()),
        })
    }
}

// Like a tweet
async fn like_tweet(
    data: web::Data<AppState>,
    req: web::Json<LikeTweetRequest>,
) -> impl Responder {
    let users = data.users.lock().unwrap();
    let mut tweets = data.tweets.lock().unwrap();
    let mut likes = data.likes.lock().unwrap();

    // Verify user and tweet exist
    if !users.iter().any(|u| u.id == req.user_id) {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("User not found".to_string()),
        });
    }

    if !tweets.iter().any(|t| t.id == req.tweet_id) {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Tweet not found".to_string()),
        });
    }

    // Check if already liked
    if likes.iter().any(|l| l.user_id == req.user_id && l.tweet_id == req.tweet_id) {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Already liked this tweet".to_string()),
        });
    }

    // Create like
    let like = Like {
        id: Uuid::new_v4().to_string(),
        user_id: req.user_id.clone(),
        tweet_id: req.tweet_id.clone(),
        created_at: Utc::now(),
    };

    // Increment like count
    if let Some(tweet) = tweets.iter_mut().find(|t| t.id == req.tweet_id) {
        tweet.likes_count += 1;
    }

    likes.push(like.clone());
    HttpResponse::Created().json(ApiResponse {
        success: true,
        data: Some(like),
        message: Some("Tweet liked successfully".to_string()),
    })
}

// Unlike a tweet
async fn unlike_tweet(
    data: web::Data<AppState>,
    req: web::Json<LikeTweetRequest>,
) -> impl Responder {
    let mut tweets = data.tweets.lock().unwrap();
    let mut likes = data.likes.lock().unwrap();

    // Find and remove like
    if let Some(pos) = likes.iter().position(|l| l.user_id == req.user_id && l.tweet_id == req.tweet_id) {
        likes.remove(pos);

        // Decrement like count
        if let Some(tweet) = tweets.iter_mut().find(|t| t.id == req.tweet_id) {
            tweet.likes_count = tweet.likes_count.saturating_sub(1);
        }

        HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some("Tweet unliked successfully"),
            message: None,
        })
    } else {
        HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Like not found".to_string()),
        })
    }
}

// Get likes for a tweet
async fn get_tweet_likes(data: web::Data<AppState>, tweet_id: web::Path<String>) -> impl Responder {
    let likes = data.likes.lock().unwrap();
    let tweet_id = tweet_id.into_inner();
    let tweet_likes: Vec<Like> = likes
        .iter()
        .filter(|l| l.tweet_id == tweet_id)
        .cloned()
        .collect();
    
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(tweet_likes),
        message: None,
    })
}

// Follow a user
async fn follow_user(
    data: web::Data<AppState>,
    req: web::Json<FollowUserRequest>,
) -> impl Responder {
    let mut users = data.users.lock().unwrap();
    let mut follows = data.follows.lock().unwrap();

    // Check if both users exist
    if !users.iter().any(|u| u.id == req.follower_id) || !users.iter().any(|u| u.id == req.following_id) {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("User not found".to_string()),
        });
    }

    // Can't follow yourself
    if req.follower_id == req.following_id {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Cannot follow yourself".to_string()),
        });
    }

    // Check if already following
    if follows.iter().any(|f| f.follower_id == req.follower_id && f.following_id == req.following_id) {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Already following this user".to_string()),
        });
    }

    // Create follow
    let follow = Follow {
        id: Uuid::new_v4().to_string(),
        follower_id: req.follower_id.clone(),
        following_id: req.following_id.clone(),
        created_at: Utc::now(),
    };

    // Update follower/following counts
    if let Some(follower) = users.iter_mut().find(|u| u.id == req.follower_id) {
        follower.following_count += 1;
    }
    if let Some(following) = users.iter_mut().find(|u| u.id == req.following_id) {
        following.followers_count += 1;
    }

    follows.push(follow.clone());
    HttpResponse::Created().json(ApiResponse {
        success: true,
        data: Some(follow),
        message: Some("User followed successfully".to_string()),
    })
}

// Unfollow a user
async fn unfollow_user(
    data: web::Data<AppState>,
    req: web::Json<FollowUserRequest>,
) -> impl Responder {
    let mut users = data.users.lock().unwrap();
    let mut follows = data.follows.lock().unwrap();

    // Find and remove follow
    if let Some(pos) = follows.iter().position(|f| f.follower_id == req.follower_id && f.following_id == req.following_id) {
        follows.remove(pos);

        // Update follower/following counts
        if let Some(follower) = users.iter_mut().find(|u| u.id == req.follower_id) {
            follower.following_count = follower.following_count.saturating_sub(1);
        }
        if let Some(following) = users.iter_mut().find(|u| u.id == req.following_id) {
            following.followers_count = following.followers_count.saturating_sub(1);
        }

        HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some("User unfollowed successfully"),
            message: None,
        })
    } else {
        HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Follow relationship not found".to_string()),
        })
    }
}

// Get followers of a user
async fn get_followers(data: web::Data<AppState>, user_id: web::Path<String>) -> impl Responder {
    let user_id = user_id.into_inner();
    let follows = data.follows.lock().unwrap();
    let followers: Vec<Follow> = follows
        .iter()
        .filter(|f| f.following_id == user_id)
        .cloned()
        .collect();
    
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(followers),
        message: None,
    })
}

// Get users that a user is following
async fn get_following(data: web::Data<AppState>, user_id: web::Path<String>) -> impl Responder {
    let user_id = user_id.into_inner();
    let follows = data.follows.lock().unwrap();
    let following: Vec<Follow> = follows
        .iter()
        .filter(|f| f.follower_id == user_id)
        .cloned()
        .collect();
    
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(following),
        message: None,
    })
}

// Get timeline (tweets from users you follow)
async fn get_timeline(data: web::Data<AppState>, user_id: web::Path<String>) -> impl Responder {
    let user_id = user_id.into_inner();
    let follows = data.follows.lock().unwrap();
    let tweets = data.tweets.lock().unwrap();
    
    // Get IDs of users being followed
    let following_ids: Vec<String> = follows
        .iter()
        .filter(|f| f.follower_id == user_id)
        .map(|f| f.following_id.clone())
        .collect();
    
    // Get tweets from followed users
    let mut timeline: Vec<Tweet> = tweets
        .iter()
        .filter(|t| following_ids.contains(&t.user_id))
        .cloned()
        .collect();
    
    // Sort by created_at descending (newest first)
    timeline.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(timeline),
        message: None,
    })
}

// ============ MAIN APPLICATION ============

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        users: Mutex::new(Vec::new()),
        tweets: Mutex::new(Vec::new()),
        likes: Mutex::new(Vec::new()),
        follows: Mutex::new(Vec::new()),
    });

    println!("🚀 Twitter API Server starting at http://127.0.0.1:3000");
    println!("📋 Available endpoints:");
    println!("   GET  /api/health            - Health check");
    println!("   GET  /api/users             - Get all users");
    println!("   GET  /api/users/{{id}}        - Get user by ID");
    println!("   POST /api/users             - Create user");
    println!("   GET  /api/tweets            - Get all tweets");
    println!("   GET  /api/tweets/{{id}}       - Get tweet by ID");
    println!("   GET  /api/users/{{id}}/tweets - Get user's tweets");
    println!("   POST /api/tweets            - Create tweet");
    println!("   DELETE /api/tweets/{{id}}     - Delete tweet");
    println!("   POST /api/likes             - Like a tweet");
    println!("   DELETE /api/likes           - Unlike a tweet");
    println!("   GET  /api/tweets/{{id}}/likes - Get tweet's likes");
    println!("   POST /api/follows           - Follow a user");
    println!("   DELETE /api/follows         - Unfollow a user");
    println!("   GET  /api/users/{{id}}/followers - Get user's followers");
    println!("   GET  /api/users/{{id}}/following - Get users followed");
    println!("   GET  /api/users/{{id}}/timeline  - Get user's timeline");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/api/health", web::get().to(health_check))
            // User routes
            .route("/api/users", web::get().to(get_users))
            .route("/api/users/{id}", web::get().to(get_user))
            .route("/api/users", web::post().to(create_user))
            // Tweet routes
            .route("/api/tweets", web::get().to(get_tweets))
            .route("/api/tweets/{id}", web::get().to(get_tweet))
            .route("/api/users/{id}/tweets", web::get().to(get_user_tweets))
            .route("/api/tweets", web::post().to(create_tweet))
            .route("/api/tweets/{id}", web::delete().to(delete_tweet))
            // Like routes
            .route("/api/likes", web::post().to(like_tweet))
            .route("/api/likes", web::delete().to(unlike_tweet))
            .route("/api/tweets/{id}/likes", web::get().to(get_tweet_likes))
            // Follow routes
            .route("/api/follows", web::post().to(follow_user))
            .route("/api/follows", web::delete().to(unfollow_user))
            .route("/api/users/{id}/followers", web::get().to(get_followers))
            .route("/api/users/{id}/following", web::get().to(get_following))
            // Timeline route
            .route("/api/users/{id}/timeline", web::get().to(get_timeline))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

