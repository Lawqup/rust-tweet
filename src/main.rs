#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_sync_db_pools;

use rocket::fs::{relative, FileServer};
use rocket::serde::Serialize;
use rocket_dyn_templates::{context, Template};

mod database;
use database::Tweet;

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

#[derive(Debug, Serialize)]
struct Context {
    tweets: Vec<Tweet>,
}

impl Context {
    pub async fn new(conn: &DbConn) -> Self {
        Self {
            tweets: Tweet::all(conn).await,
        }
    }
}

#[get("/")]
async fn index(conn: DbConn) -> Template {
    Template::render("index", Context::new(&conn).await)
}

#[get("/tweet/<id>")]
async fn tweet(id: i32, conn: DbConn) -> Template {
    let tweet = Tweet::get(id, &conn).await;
    Template::render(
        "tweet",
        context! {
            tweet: &tweet,
            author: Tweet::author_username(tweet.author_id, &conn).await,
        },
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![index, tweet])
}
