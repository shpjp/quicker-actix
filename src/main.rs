mod auth;
mod db;
mod models;

use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use models::*;
use sqlx::PgPool;
use std::env;
use uuid::Uuid;
use validator::Validate;

// ============ APP STATE ============

#[derive(Clone)]
struct AppState {
    db: PgPool,
    jwt_secret: String,
}

// ============ HEALTH CHECK ============

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some("Twitter API is running with PostgreSQL"),
        message: None,
    })
}

// ============ AUTH HANDLERS ============

async fn register(state: web::Data<AppState>, req: web::Json<RegisterRequest>) -> impl Responder {
    // Validate input
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Validation error: {}", e)),
        });
    }

    // Check if user exists
    let existing = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 OR username = $2"
    )
    .bind(&req.email)
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await;

    if let Ok(Some(_)) = existing {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("User with this email or username already exists".to_string()),
        });
    }

    // Hash password
    let password_hash = match auth::hash_password(&req.password) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Failed to hash password".to_string()),
            });
        }
    };

    // Insert user
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, password_hash, display_name) 
         VALUES ($1, $2, $3, $4) 
         RETURNING *"
    )
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&req.display_name)
    .fetch_one(&state.db)
    .await;

    match user {
        Ok(user) => {
            // Create JWT token
            let token = match auth::create_jwt(user.id, user.email.clone(), &state.jwt_secret) {
                Ok(t) => t,
                Err(_) => {
                    return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                        success: false,
                        data: None,
                        message: Some("Failed to create token".to_string()),
                    });
                }
            };

            HttpResponse::Created().json(ApiResponse {
                success: true,
                data: Some(AuthResponse {
                    token,
                    user: user.into(),
                }),
                message: Some("User registered successfully".to_string()),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

async fn login(state: web::Data<AppState>, req: web::Json<LoginRequest>) -> impl Responder {
    // Validate input
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Validation error: {}", e)),
        });
    }

    // Find user by email
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await;

    match user {
        Ok(Some(user)) => {
            // Verify password
            match auth::verify_password(&req.password, &user.password_hash) {
                Ok(true) => {
                    // Create JWT token
                    let token = match auth::create_jwt(user.id, user.email.clone(), &state.jwt_secret) {
                        Ok(t) => t,
                        Err(_) => {
                            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                                success: false,
                                data: None,
                                message: Some("Failed to create token".to_string()),
                            });
                        }
                    };

                    HttpResponse::Ok().json(ApiResponse {
                        success: true,
                        data: Some(AuthResponse {
                            token,
                            user: user.into(),
                        }),
                        message: Some("Login successful".to_string()),
                    })
                }
                _ => HttpResponse::Unauthorized().json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    message: Some("Invalid credentials".to_string()),
                }),
            }
        }
        _ => HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Invalid credentials".to_string()),
        }),
    }
}

async fn get_me(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
    
    let user_id = match auth::get_user_id_from_token(auth_header, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(e),
            });
        }
    };

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await;

    match user {
        Ok(Some(user)) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(UserResponse::from(user)),
            message: None,
        }),
        _ => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("User not found".to_string()),
        }),
    }
}

// ============ USER HANDLERS ============

async fn get_user_by_username(state: web::Data<AppState>, username: web::Path<String>) -> impl Responder {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(username.as_str())
        .fetch_optional(&state.db)
        .await;

    match user {
        Ok(Some(user)) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(UserResponse::from(user)),
            message: None,
        }),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("User not found".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

async fn update_profile(
    state: web::Data<AppState>,
    req: HttpRequest,
    update: web::Json<UpdateProfileRequest>,
) -> impl Responder {
    let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
    
    let user_id = match auth::get_user_id_from_token(auth_header, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(e),
            });
        }
    };

    let result = sqlx::query_as::<_, User>(
        "UPDATE users 
         SET display_name = COALESCE($1, display_name),
             bio = COALESCE($2, bio),
             profile_image = COALESCE($3, profile_image),
             banner_image = COALESCE($4, banner_image)
         WHERE id = $5
         RETURNING *"
    )
    .bind(&update.display_name)
    .bind(&update.bio)
    .bind(&update.profile_image)
    .bind(&update.banner_image)
    .bind(user_id)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(user) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(UserResponse::from(user)),
            message: Some("Profile updated successfully".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

// ============ TWEET HANDLERS ============

async fn create_tweet(
    state: web::Data<AppState>,
    req: HttpRequest,
    tweet_req: web::Json<CreateTweetRequest>,
) -> impl Responder {
    if let Err(e) = tweet_req.validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Validation error: {}", e)),
        });
    }

    let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
    
    let user_id = match auth::get_user_id_from_token(auth_header, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(e),
            });
        }
    };

    let tweet = sqlx::query_as::<_, Tweet>(
        "INSERT INTO tweets (user_id, content, image_url) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(user_id)
    .bind(&tweet_req.content)
    .bind(&tweet_req.image_url)
    .fetch_one(&state.db)
    .await;

    match tweet {
        Ok(tweet) => {
            // Get user info
            let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&state.db)
                .await;

            if let Ok(user) = user {
                HttpResponse::Created().json(ApiResponse {
                    success: true,
                    data: Some(TweetResponse {
                        id: tweet.id,
                        content: tweet.content,
                        image_url: tweet.image_url,
                        likes_count: tweet.likes_count,
                        retweets_count: tweet.retweets_count,
                        replies_count: tweet.replies_count,
                        created_at: tweet.created_at,
                        user: user.into(),
                        is_liked: false,
                    }),
                    message: Some("Tweet created successfully".to_string()),
                })
            } else {
                HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    message: Some("Failed to fetch user data".to_string()),
                })
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

async fn get_timeline(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
    
    let user_id = match auth::get_user_id_from_token(auth_header, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(e),
            });
        }
    };

    // Get tweets from followed users + own tweets
    let tweets = sqlx::query_as::<_, TweetWithUser>(
        "SELECT t.id, t.user_id, t.content, t.image_url, t.likes_count, t.retweets_count, 
                t.replies_count, t.created_at,
                u.username as user_username, u.display_name as user_display_name, 
                u.email as user_email, u.bio as user_bio, 
                u.profile_image as user_profile_image, u.banner_image as user_banner_image,
                u.followers_count as user_followers_count, u.following_count as user_following_count,
                u.verified as user_verified, u.created_at as user_created_at
         FROM tweets t
         INNER JOIN users u ON t.user_id = u.id
         WHERE t.user_id IN (
             SELECT following_id FROM follows WHERE follower_id = $1
             UNION
             SELECT $1
         )
         ORDER BY t.created_at DESC
         LIMIT 50"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await;

    match tweets {
        Ok(tweets) => {
            let mut tweet_responses = Vec::new();
            
            for tweet in tweets {
                // Check if current user liked this tweet
                let is_liked = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM likes WHERE user_id = $1 AND tweet_id = $2)"
                )
                .bind(user_id)
                .bind(tweet.id)
                .fetch_one(&state.db)
                .await
                .unwrap_or(false);

                tweet_responses.push(TweetResponse {
                    id: tweet.id,
                    content: tweet.content,
                    image_url: tweet.image_url,
                    likes_count: tweet.likes_count,
                    retweets_count: tweet.retweets_count,
                    replies_count: tweet.replies_count,
                    created_at: tweet.created_at,
                    user: UserResponse {
                        id: tweet.user_id,
                        username: tweet.user_username,
                        email: tweet.user_email,
                        display_name: tweet.user_display_name,
                        bio: tweet.user_bio,
                        profile_image: tweet.user_profile_image,
                        banner_image: tweet.user_banner_image,
                        followers_count: tweet.user_followers_count,
                        following_count: tweet.user_following_count,
                        verified: tweet.user_verified,
                        created_at: tweet.user_created_at,
                    },
                    is_liked,
                });
            }

            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(tweet_responses),
                message: None,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

async fn get_user_tweets(state: web::Data<AppState>, username: web::Path<String>) -> impl Responder {
    let tweets = sqlx::query_as::<_, TweetWithUser>(
        "SELECT t.id, t.user_id, t.content, t.image_url, t.likes_count, t.retweets_count, 
                t.replies_count, t.created_at,
                u.username as user_username, u.display_name as user_display_name, 
                u.email as user_email, u.bio as user_bio, 
                u.profile_image as user_profile_image, u.banner_image as user_banner_image,
                u.followers_count as user_followers_count, u.following_count as user_following_count,
                u.verified as user_verified, u.created_at as user_created_at
         FROM tweets t
         INNER JOIN users u ON t.user_id = u.id
         WHERE u.username = $1
         ORDER BY t.created_at DESC"
    )
    .bind(username.as_str())
    .fetch_all(&state.db)
    .await;

    match tweets {
        Ok(tweets) => {
            let tweet_responses: Vec<TweetResponse> = tweets
                .into_iter()
                .map(|tweet| TweetResponse {
                    id: tweet.id,
                    content: tweet.content,
                    image_url: tweet.image_url,
                    likes_count: tweet.likes_count,
                    retweets_count: tweet.retweets_count,
                    replies_count: tweet.replies_count,
                    created_at: tweet.created_at,
                    user: UserResponse {
                        id: tweet.user_id,
                        username: tweet.user_username,
                        email: tweet.user_email,
                        display_name: tweet.user_display_name,
                        bio: tweet.user_bio,
                        profile_image: tweet.user_profile_image,
                        banner_image: tweet.user_banner_image,
                        followers_count: tweet.user_followers_count,
                        following_count: tweet.user_following_count,
                        verified: tweet.user_verified,
                        created_at: tweet.user_created_at,
                    },
                    is_liked: false,
                })
                .collect();

            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(tweet_responses),
                message: None,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

async fn delete_tweet(state: web::Data<AppState>, req: HttpRequest, tweet_id: web::Path<Uuid>) -> impl Responder {
    let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
    
    let user_id = match auth::get_user_id_from_token(auth_header, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(e),
            });
        }
    };

    let result = sqlx::query("DELETE FROM tweets WHERE id = $1 AND user_id = $2")
        .bind(tweet_id.into_inner())
        .bind(user_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some("Tweet deleted successfully"),
            message: None,
        }),
        Ok(_) => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Tweet not found or unauthorized".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

// ============ LIKE HANDLERS ============

async fn like_tweet(state: web::Data<AppState>, req: HttpRequest, tweet_id: web::Path<Uuid>) -> impl Responder {
    let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
    
    let user_id = match auth::get_user_id_from_token(auth_header, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(e),
            });
        }
    };

    let tweet_id = tweet_id.into_inner();

    // Check if already liked
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM likes WHERE user_id = $1 AND tweet_id = $2)"
    )
    .bind(user_id)
    .bind(tweet_id)
    .fetch_one(&state.db)
    .await;

    if let Ok(true) = exists {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Already liked this tweet".to_string()),
        });
    }

    // Insert like and update count
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            });
        }
    };

    let like_result = sqlx::query("INSERT INTO likes (user_id, tweet_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(tweet_id)
        .execute(&mut *tx)
        .await;

    if like_result.is_err() {
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Failed to like tweet".to_string()),
        });
    }

    let update_result = sqlx::query("UPDATE tweets SET likes_count = likes_count + 1 WHERE id = $1")
        .bind(tweet_id)
        .execute(&mut *tx)
        .await;

    match update_result {
        Ok(_) => {
            let _ = tx.commit().await;
            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some("Tweet liked successfully"),
                message: None,
            })
        }
        Err(e) => {
            let _ = tx.rollback().await;
            HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            })
        }
    }
}

async fn unlike_tweet(state: web::Data<AppState>, req: HttpRequest, tweet_id: web::Path<Uuid>) -> impl Responder {
    let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
    
    let user_id = match auth::get_user_id_from_token(auth_header, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(e),
            });
        }
    };

    let tweet_id = tweet_id.into_inner();

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            });
        }
    };

    let delete_result = sqlx::query("DELETE FROM likes WHERE user_id = $1 AND tweet_id = $2")
        .bind(user_id)
        .bind(tweet_id)
        .execute(&mut *tx)
        .await;

    match delete_result {
        Ok(result) if result.rows_affected() > 0 => {
            let _ = sqlx::query("UPDATE tweets SET likes_count = likes_count - 1 WHERE id = $1")
                .bind(tweet_id)
                .execute(&mut *tx)
                .await;

            let _ = tx.commit().await;
            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some("Tweet unliked successfully"),
                message: None,
            })
        }
        _ => {
            let _ = tx.rollback().await;
            HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Like not found".to_string()),
            })
        }
    }
}

// ============ FOLLOW HANDLERS ============

async fn follow_user(state: web::Data<AppState>, req: HttpRequest, username: web::Path<String>) -> impl Responder {
    let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
    
    let follower_id = match auth::get_user_id_from_token(auth_header, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(e),
            });
        }
    };

    // Get user to follow
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(username.as_str())
        .fetch_optional(&state.db)
        .await;

    let following_id = match user {
        Ok(Some(user)) => user.id,
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("User not found".to_string()),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            });
        }
    };

    if follower_id == following_id {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Cannot follow yourself".to_string()),
        });
    }

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            });
        }
    };

    let follow_result = sqlx::query("INSERT INTO follows (follower_id, following_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(follower_id)
        .bind(following_id)
        .execute(&mut *tx)
        .await;

    if let Ok(result) = follow_result {
        if result.rows_affected() > 0 {
            let _ = sqlx::query("UPDATE users SET following_count = following_count + 1 WHERE id = $1")
                .bind(follower_id)
                .execute(&mut *tx)
                .await;

            let _ = sqlx::query("UPDATE users SET followers_count = followers_count + 1 WHERE id = $1")
                .bind(following_id)
                .execute(&mut *tx)
                .await;

            let _ = tx.commit().await;

            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some("User followed successfully"),
                message: None,
            })
        } else {
            let _ = tx.rollback().await;
            HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Already following this user".to_string()),
            })
        }
    } else {
        let _ = tx.rollback().await;
        HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Failed to follow user".to_string()),
        })
    }
}

async fn unfollow_user(state: web::Data<AppState>, req: HttpRequest, username: web::Path<String>) -> impl Responder {
    let auth_header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
    
    let follower_id = match auth::get_user_id_from_token(auth_header, &state.jwt_secret) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(e),
            });
        }
    };

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(username.as_str())
        .fetch_optional(&state.db)
        .await;

    let following_id = match user {
        Ok(Some(user)) => user.id,
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("User not found".to_string()),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            });
        }
    };

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            });
        }
    };

    let delete_result = sqlx::query("DELETE FROM follows WHERE follower_id = $1 AND following_id = $2")
        .bind(follower_id)
        .bind(following_id)
        .execute(&mut *tx)
        .await;

    match delete_result {
        Ok(result) if result.rows_affected() > 0 => {
            let _ = sqlx::query("UPDATE users SET following_count = following_count - 1 WHERE id = $1")
                .bind(follower_id)
                .execute(&mut *tx)
                .await;

            let _ = sqlx::query("UPDATE users SET followers_count = followers_count - 1 WHERE id = $1")
                .bind(following_id)
                .execute(&mut *tx)
                .await;

            let _ = tx.commit().await;

            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some("User unfollowed successfully"),
                message: None,
            })
        }
        _ => {
            let _ = tx.rollback().await;
            HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Not following this user".to_string()),
            })
        }
    }
}

// ============ MAIN ============

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("SERVER_PORT must be a valid number");

    // Create database pool
    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app_state = web::Data::new(AppState {
        db: pool,
        jwt_secret,
    });

    println!("🚀 Twitter API Server starting at http://{}:{}", host, port);
    println!("📋 Database: Connected to PostgreSQL");
    println!("🔐 Authentication: JWT enabled");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            // Serve static frontend
            .service(fs::Files::new("/", "./static").index_file("index.html"))
            // API routes
            .route("/api/health", web::get().to(health_check))
            // Auth routes
            .route("/api/auth/register", web::post().to(register))
            .route("/api/auth/login", web::post().to(login))
            .route("/api/auth/me", web::get().to(get_me))
            // User routes
            .route("/api/users/{username}", web::get().to(get_user_by_username))
            .route("/api/users/profile", web::put().to(update_profile))
            // Tweet routes
            .route("/api/tweets", web::post().to(create_tweet))
            .route("/api/tweets/timeline", web::get().to(get_timeline))
            .route("/api/tweets/{id}", web::delete().to(delete_tweet))
            .route("/api/users/{username}/tweets", web::get().to(get_user_tweets))
            // Like routes
            .route("/api/tweets/{id}/like", web::post().to(like_tweet))
            .route("/api/tweets/{id}/unlike", web::delete().to(unlike_tweet))
            // Follow routes
            .route("/api/users/{username}/follow", web::post().to(follow_user))
            .route("/api/users/{username}/unfollow", web::delete().to(unfollow_user))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
