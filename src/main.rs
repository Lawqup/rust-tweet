#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_sync_db_pools;

use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::http::{Cookie, CookieJar};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket::serde::Serialize;
use rocket_dyn_templates::{context, Template};

mod database;
use database::{Tweet, User, UserNew};

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

#[derive(Debug, Serialize)]
struct Context {
    tweets: Vec<Tweet>,
    user: Option<User>,
}

impl Context {
    pub async fn new(conn: &DbConn) -> Self {
        Self {
            tweets: Tweet::all(conn).await,
            user: None,
        }
    }

    pub async fn with_user(mut self, user: Auth, conn: &DbConn) -> Self {
        self.user = User::get_from_id(user.0, conn).await;
        self
    }
}

#[derive(Debug)]
struct Auth(i32);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        req.cookies()
            .get("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| Auth(id))
            .or_forward(())
    }
}

#[post("/login", data = "<user>")]
async fn login(cookies: &CookieJar<'_>, user: Form<UserNew>, conn: DbConn) -> Redirect {
    let u = user.into_inner().login(&conn).await;
    cookies.add(Cookie::new("user_id", u.id.to_string()));

    Redirect::to("/")
}

#[post("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::named("user_id"));

    Redirect::to("/")
}

#[get("/")]
async fn logged_in(user: Auth, conn: DbConn) -> Template {
    Template::render(
        "logged_in",
        Context::new(&conn).await.with_user(user, &conn).await,
    )
}

// serve index when there is no authentication
#[get("/", rank = 2)]
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
        .mount("/", routes![index, tweet, login, logged_in, logout])
}
