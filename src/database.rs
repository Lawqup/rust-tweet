use crate::DbConn;
use diesel::{self, prelude::*};
use serde::Serialize;

mod schema {
    table! {
        users {
            id -> Integer,
            username -> Text,
        }
    }

    table! {
        tweets {
            id -> Integer,
            title -> Text,
            body -> Text,
            author_id -> Integer,
        }
    }
}

use schema::tweets;
use schema::tweets::dsl::author_id as tweet_author_id;
use schema::tweets::dsl::id as tweet_id;
use schema::tweets::dsl::tweets as all_tweets;

use schema::users;
use schema::users::dsl::username as user_username;
use schema::users::dsl::users as all_users;

#[derive(Queryable, Debug, Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Insertable, Clone, FromForm)]
#[table_name = "users"]
pub struct UserNew {
    pub username: String,
}

#[derive(Queryable, Debug, Serialize)]
pub struct Tweet {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub author_id: i32,
}

#[derive(Insertable, FromForm)]
#[table_name = "tweets"]
pub struct TweetNew {
    pub title: String,
    pub body: String,
    pub author_id: i32,
}

impl UserNew {
    pub async fn login(self, conn: &DbConn) -> User {
        let _ = User::insert(self.clone(), conn).await;
        User::get_from_username(self.username, conn).await.unwrap()
    }
}

impl User {
    /// Returns all users in the database.
    pub async fn all(conn: &DbConn) -> Vec<User> {
        conn.run(|c| all_users.load::<User>(c).unwrap()).await
    }

    /// Inserts a user and returns true of insertion was successful.
    pub async fn insert(user: UserNew, conn: &DbConn) -> bool {
        conn.run(move |c| {
            diesel::insert_into(users::table)
                .values(&user)
                .execute(c)
                .is_ok()
        })
        .await
    }

    /// Returns user from username.
    pub async fn get_from_username(username: String, conn: &DbConn) -> Option<User> {
        conn.run(|c| {
            all_users
                .filter(user_username.eq(username))
                .get_result::<User>(c)
                .ok()
        })
        .await
    }

    /// Returns user from id.
    pub async fn get_from_id(id: i32, conn: &DbConn) -> Option<User> {
        conn.run(move |c| all_users.find(id).get_result::<User>(c).ok())
            .await
    }
}

impl Tweet {
    /// Returns all tweets in the database.
    pub async fn all(conn: &DbConn) -> Vec<Tweet> {
        conn.run(|c| all_tweets.order(tweet_id.desc()).load::<Tweet>(c).unwrap())
            .await
    }

    /// Returns a tweet associated with the id.
    pub async fn get(id: i32, conn: &DbConn) -> Tweet {
        conn.run(move |c| {
            all_tweets
                .filter(tweet_id.eq(id))
                .load::<Tweet>(c)
                .unwrap()
                .into_iter()
                .nth(0)
                .unwrap()
        })
        .await
    }

    /// Inserts a tweet in the DB and returns the ID if insertion was successful.
    pub async fn insert(tweet: TweetNew, conn: &DbConn) -> Option<i32> {
        conn.run(move |c| {
            if !diesel::insert_into(tweets::table)
                .values(&tweet)
                .execute(c)
                .is_ok()
            {
                None
            } else {
                Some(
                    tweets::table
                        .order(tweet_id.desc())
                        .first::<Tweet>(c)
                        .unwrap()
                        .id,
                )
            }
        })
        .await
    }

    /// Get the username associated with the author_id if that user exists.
    pub async fn author_username(author_id: i32, conn: &DbConn) -> Option<String> {
        joinable!(tweets -> users (author_id));
        allow_tables_to_appear_in_same_query!(tweets, users);

        conn.run(move |c| {
            users::table
                .inner_join(tweets::table)
                .select(user_username)
                .filter(tweet_author_id.eq(author_id))
                .get_result::<String>(c)
                .ok()
        })
        .await
    }
}
