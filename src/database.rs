use diesel::{self, prelude::*, sqlite::SqliteConnection};

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

use schema::users;
use schema::users::dsl::{username as user_username, users as all_users};

use schema::tweets;
use schema::tweets::dsl::tweets as all_tweets;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct UserNew {
    pub username: String,
}

#[derive(Queryable)]
pub struct Tweet {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub author_id: i32,
}

#[derive(Insertable)]
#[table_name = "tweets"]
pub struct TweetNew {
    pub title: String,
    pub body: String,
    pub author_id: i32,
}

impl User {
    /// Returns all users in the database.
    pub fn all(conn: &SqliteConnection) -> Vec<User> {
        all_users.load::<User>(conn).unwrap()
    }

    /// Inserts a user and returns true of insertion was successful.
    pub fn insert(user: UserNew, conn: &SqliteConnection) -> bool {
        let res = diesel::insert_into(users::table)
            .values(&user)
            .execute(conn);

        match res {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Returns true if username is available.
    pub fn username_available(username: String, conn: &SqliteConnection) -> bool {
        all_users
            .filter(user_username.eq(username))
            .load::<User>(conn)
            .unwrap()
            .is_empty()
    }
}

impl Tweet {
    /// Returns all tweets in the database.
    pub fn all(conn: &SqliteConnection) -> Vec<Tweet> {
        all_tweets.load::<Tweet>(conn).unwrap()
    }

    /// Inserts a tweet in the DB and returns true of insertion was successful.
    pub fn insert(tweet: TweetNew, conn: &SqliteConnection) -> bool {
        let res = diesel::insert_into(tweets::table)
            .values(&tweet)
            .execute(conn);

        match res {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
