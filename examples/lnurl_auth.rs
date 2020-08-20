use warp;
use warp::Filter;

#[tokio::main]
async fn main() {
    let db = model::new_db();
    let api = filter::api(db).with(warp::log("api"));
    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
}

mod filter {
    use super::handler;
    use super::model::User;
    use super::model::DB;
    use warp::Filter;

    pub fn api(db: DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        users_list(db)
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
}

mod handler {
    use super::model::{Users, DB};
    use std::convert::Infallible;
    use warp::http::StatusCode;

    pub async fn list_users(db: DB) -> Result<impl warp::Reply, Infallible> {
        let users = db.lock().await;
        let list = Users {
            users: users.to_vec(),
        };
        Ok(warp::reply::json(&list))
    }
}

mod model {
    use serde_derive::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Users {
        pub users: Vec<User>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct User {
        pub pk: String,
    }

    pub type DB = Arc<Mutex<Vec<User>>>;

    pub fn new_db() -> DB {
        Arc::new(Mutex::new(Vec::new()))
    }
}
