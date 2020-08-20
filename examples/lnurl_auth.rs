use std::env;
use warp;
use warp::Filter;

use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let url = env::var("SERVICE_URL").unwrap();
    let verifier = lnurl::service::AuthVerifier::new();
    let db = model::new_db();
    let api = filter::api(url, db, verifier).with(warp::log("api"));
    warp::serve(api).run(([127, 0, 0, 1], 8383)).await;
}

mod filter {
    use super::auth;
    use super::handler;
    use super::model::DB;
    use lnurl::service::AuthVerifier;
    use warp::Filter;

    pub fn api(
        url: String,
        db: DB,
        verifier: AuthVerifier,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        auth(db.clone(), verifier)
            .or(login(db.clone(), url))
            .or(users_list(db))
            .with(warp::trace::request())
    }

    /// GET /auth?sig=<sig>&key=<key>
    pub fn auth(
        db: DB,
        verifier: AuthVerifier,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("auth")
            .and(warp::get())
            .and(with_db(db))
            .and(with_verifier(verifier))
            .and(warp::query::<auth::Auth>())
            .and_then(handler::auth)
    }

    /// GET /login
    pub fn login(
        db: DB,
        url: String,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("login")
            .and(warp::get())
            .and(with_db(db))
            .and(warp::any().map(move || url.clone()))
            .and_then(handler::login)
    }

    /// GET /users
    pub fn users_list(
        db: DB,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("users")
            .and(warp::get())
            .and(with_db(db))
            .and_then(handler::list_users)
    }

    fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    fn with_verifier(
        v: AuthVerifier,
    ) -> impl Filter<Extract = (AuthVerifier,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || v.clone())
    }
}

mod auth {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub struct Auth {
        pub k1: String,
        pub sig: String,
        pub key: String,
    }
}

mod handler {
    use super::auth;
    use super::img;
    use super::model::{Sessions, User, DB};
    use hex::encode;
    use rand::random;
    use std::convert::Infallible;

    pub async fn auth(
        db: DB,
        verifier: lnurl::service::AuthVerifier,
        credentials: auth::Auth,
    ) -> Result<impl warp::Reply, Infallible> {
        let mut sessions = db.lock().await;
        if sessions.get(&credentials.k1).is_none() {
            return Ok(warp::reply::json(&lnurl::Response::Error {
                reason: format!("{} does not exist", &credentials.k1),
            }));
        }
        let res = verifier
            .verify(&credentials.k1, &credentials.sig, &credentials.key)
            .unwrap();
        if !res {
            return Ok(warp::reply::json(&lnurl::Response::Error {
                reason: format!(
                    "{}, {}, {}",
                    &credentials.k1, &credentials.sig, &credentials.key
                ),
            }));
        }
        sessions.insert(
            credentials.k1,
            Some(User {
                pk: credentials.key,
            }),
        );
        Ok(warp::reply::json(&lnurl::Response::Ok {
            event: Some(lnurl::Event::LoggedIn),
        }))
    }

    pub async fn list_users(db: DB) -> Result<impl warp::Reply, Infallible> {
        let sessions = db.lock().await;
        let users = sessions
            .values()
            .filter_map(|o| o.as_ref())
            .map(|u| u.clone())
            .collect();
        let list = Sessions { users: users };
        Ok(warp::reply::json(&list))
    }

    pub async fn login(db: DB, url: String) -> Result<impl warp::Reply, Infallible> {
        let challenge: [u8; 32] = random();
        let k1 = encode(challenge);
        let url = format!("{}/auth?tag=login&k1={}", url, &k1);
        let mut sessions = db.lock().await;
        sessions.insert(k1, None);
        Ok(warp::http::Response::builder().body(img::create_qrcode(&url)))
    }
}

mod img {
    use bech32::ToBase32;
    use image::{DynamicImage, ImageOutputFormat, Luma};
    use qrcode::QrCode;

    pub fn create_qrcode(url: &str) -> Vec<u8> {
        let encoded = bech32::encode("lnurl", url.as_bytes().to_base32()).unwrap();
        let code = QrCode::new(encoded.to_string()).unwrap();
        let mut image: Vec<u8> = Vec::new();
        let img = DynamicImage::ImageLuma8(code.render::<Luma<u8>>().build());
        img.write_to(&mut image, ImageOutputFormat::PNG).unwrap();
        image
    }
}

mod model {
    use serde_derive::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Sessions {
        pub users: Vec<User>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct User {
        pub pk: String,
    }

    pub type DB = Arc<Mutex<HashMap<String, Option<User>>>>;

    pub fn new_db() -> DB {
        Arc::new(Mutex::new(HashMap::new()))
    }
}
