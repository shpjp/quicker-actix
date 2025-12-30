# Twitter-Like REST API in Rust 🦀

A comprehensive Twitter-like backend API built with **Actix-web** and **Serde** in Rust. This API provides full social media functionality including user management, tweets, likes, follows, and personalized timelines.

## 🚀 Features

- **User Management**: Create users with profiles, bios, and track follower/following counts
- **Tweet Operations**: Post tweets (280 char limit), view, and delete
- **Social Interactions**: Like/unlike tweets, follow/unfollow users
- **Personalized Timeline**: Get tweets from users you follow, sorted by date
- **In-Memory Storage**: Fast data access with thread-safe Mutex-protected storage
- **RESTful Design**: Clean API endpoints with consistent JSON responses

## 📋 Tech Stack

- **Actix-web 4.4** - High-performance async web framework
- **Serde** - Serialization/deserialization for JSON
- **Chrono** - DateTime handling with timestamps
- **UUID** - Unique ID generation
- **Tokio** - Async runtime

## 🛠️ Installation & Setup

### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Cargo (comes with Rust)

### Build and Run

```bash
# Clone the repository
git clone <your-repo-url>
cd rust-31st-dec

# Build the project
cargo build

# Run the server
cargo run
```

The server will start at `http://127.0.0.1:3000`

## 📖 API Documentation

### Base URL
```
http://127.0.0.1:3000/api
```

### Response Format
All endpoints return JSON with the following structure:
```json
{
  "success": true,
  "data": <response_data>,
  "message": "Optional message"
}
```

---

## 🔌 API Endpoints

### Health Check
**GET** `/api/health`

Check if the API is running.

**Response:**
```json
{
  "success": true,
  "data": "Twitter API is running",
  "message": null
}
```

---

### 👤 User Endpoints

#### Get All Users
**GET** `/api/users`

Returns all users in the system.

#### Get User by ID
**GET** `/api/users/{id}`

Get a specific user's profile.

#### Create User
**POST** `/api/users`

**Request Body:**
```json
{
  "username": "johndoe",
  "email": "john@example.com",
  "display_name": "John Doe",
  "bio": "Software developer and coffee enthusiast"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "johndoe",
    "email": "john@example.com",
    "display_name": "John Doe",
    "bio": "Software developer and coffee enthusiast",
    "followers_count": 0,
    "following_count": 0,
    "created_at": "2025-12-31T10:30:00Z"
  },
  "message": "User created successfully"
}
```

---

### 🐦 Tweet Endpoints

#### Get All Tweets
**GET** `/api/tweets`

Returns all tweets from all users.

#### Get Tweet by ID
**GET** `/api/tweets/{id}`

Get a specific tweet.

#### Get User's Tweets
**GET** `/api/users/{id}/tweets`

Get all tweets by a specific user.

#### Create Tweet
**POST** `/api/tweets`

**Request Body:**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "Hello, Twitter! This is my first tweet! 🎉"
}
```

**Validation:**
- Content must be 1-280 characters
- User must exist

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "Hello, Twitter! This is my first tweet! 🎉",
    "likes_count": 0,
    "retweets_count": 0,
    "replies_count": 0,
    "created_at": "2025-12-31T10:35:00Z"
  },
  "message": "Tweet created successfully"
}
```

#### Delete Tweet
**DELETE** `/api/tweets/{id}`

Delete a tweet by ID.

---

### ❤️ Like Endpoints

#### Like a Tweet
**POST** `/api/likes`

**Request Body:**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "tweet_id": "660e8400-e29b-41d4-a716-446655440001"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "770e8400-e29b-41d4-a716-446655440002",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "tweet_id": "660e8400-e29b-41d4-a716-446655440001",
    "created_at": "2025-12-31T10:40:00Z"
  },
  "message": "Tweet liked successfully"
}
```

#### Unlike a Tweet
**DELETE** `/api/likes`

**Request Body:**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "tweet_id": "660e8400-e29b-41d4-a716-446655440001"
}
```

#### Get Tweet Likes
**GET** `/api/tweets/{id}/likes`

Get all likes for a specific tweet.

---

### 👥 Follow Endpoints

#### Follow a User
**POST** `/api/follows`

**Request Body:**
```json
{
  "follower_id": "550e8400-e29b-41d4-a716-446655440000",
  "following_id": "880e8400-e29b-41d4-a716-446655440003"
}
```

**Validation:**
- Both users must exist
- Cannot follow yourself
- Cannot follow the same user twice

#### Unfollow a User
**DELETE** `/api/follows`

**Request Body:**
```json
{
  "follower_id": "550e8400-e29b-41d4-a716-446655440000",
  "following_id": "880e8400-e29b-41d4-a716-446655440003"
}
```

#### Get User's Followers
**GET** `/api/users/{id}/followers`

Get all users following a specific user.

#### Get Users Followed
**GET** `/api/users/{id}/following`

Get all users that a specific user is following.

---

### 📰 Timeline Endpoint

#### Get User Timeline
**GET** `/api/users/{id}/timeline`

Get tweets from all users that the specified user follows, sorted by date (newest first).

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "tweet-id-1",
      "user_id": "followed-user-id",
      "content": "Latest tweet from followed user",
      "likes_count": 5,
      "retweets_count": 2,
      "replies_count": 1,
      "created_at": "2025-12-31T12:00:00Z"
    }
  ],
  "message": null
}
```

---

## 🧪 Testing with cURL

### Create a User
```bash
curl -X POST http://127.0.0.1:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com","display_name":"Alice","bio":"Rust enthusiast"}'
```

### Create a Tweet
```bash
curl -X POST http://127.0.0.1:3000/api/tweets \
  -H "Content-Type: application/json" \
  -d '{"user_id":"<user-id>","content":"My first tweet!"}'
```

### Like a Tweet
```bash
curl -X POST http://127.0.0.1:3000/api/likes \
  -H "Content-Type: application/json" \
  -d '{"user_id":"<user-id>","tweet_id":"<tweet-id>"}'
```

### Follow a User
```bash
curl -X POST http://127.0.0.1:3000/api/follows \
  -H "Content-Type: application/json" \
  -d '{"follower_id":"<user-id-1>","following_id":"<user-id-2>"}'
```

---

## 🏗️ Architecture

### Data Models

**User**
- `id`: Unique identifier (UUID)
- `username`: Unique username
- `email`: User email address
- `display_name`: Display name
- `bio`: Optional biography
- `followers_count`: Number of followers
- `following_count`: Number of users followed
- `created_at`: Account creation timestamp

**Tweet**
- `id`: Unique identifier (UUID)
- `user_id`: ID of the user who created the tweet
- `content`: Tweet content (max 280 chars)
- `likes_count`: Number of likes
- `retweets_count`: Number of retweets
- `replies_count`: Number of replies
- `created_at`: Tweet creation timestamp

**Like**
- `id`: Unique identifier (UUID)
- `user_id`: User who liked
- `tweet_id`: Tweet that was liked
- `created_at`: Like timestamp

**Follow**
- `id`: Unique identifier (UUID)
- `follower_id`: User who is following
- `following_id`: User being followed
- `created_at`: Follow timestamp

### Storage

Currently uses **in-memory storage** with `Mutex<Vec<T>>` for thread-safe access. Data is stored in the `AppState` struct:

```rust
struct AppState {
    users: Mutex<Vec<User>>,
    tweets: Mutex<Vec<Tweet>>,
    likes: Mutex<Vec<Like>>,
    follows: Mutex<Vec<Follow>>,
}
```

**Note:** Data is lost when the server restarts. For production, integrate a database like PostgreSQL, MongoDB, or Redis.

---

## 🚀 Future Enhancements

- [ ] Database integration (PostgreSQL/MongoDB)
- [ ] User authentication with JWT tokens
- [ ] Password hashing and security
- [ ] Retweet functionality
- [ ] Reply/comment system
- [ ] Search tweets by content/hashtags
- [ ] Pagination for large datasets
- [ ] Rate limiting
- [ ] File upload for profile pictures
- [ ] Websocket support for real-time updates
- [ ] User mentions and notifications
- [ ] Direct messaging

---

## 📝 Error Handling

The API returns appropriate HTTP status codes:

- `200 OK` - Successful GET/DELETE requests
- `201 Created` - Successful POST requests (resource created)
- `400 Bad Request` - Invalid request data or business logic violation
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server errors

Error responses include a descriptive message:
```json
{
  "success": false,
  "data": null,
  "message": "User not found"
}
```

---

## 🤝 Contributing

Contributions are welcome! Feel free to:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

---

## 📄 License

This project is open source and available under the [MIT License](LICENSE).

---

## 👨‍💻 Author

Built with ❤️ using Rust and Actix-web

---

## 🙋‍♂️ Support

If you have questions or run into issues:
- Open an issue on GitHub
- Check the Actix-web documentation: https://actix.rs/
- Rust documentation: https://doc.rust-lang.org/
