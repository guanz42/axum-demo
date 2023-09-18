mod routes;

use std::{env, net::SocketAddr, str::FromStr, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router, Server,
};
use entity::post;
use routes::hello;
use serde::Deserialize;
use service::sea_orm::{Database, DatabaseConnection};
use service::{Mutation as MutationCore, Query as QueryCore};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[tokio::main]
async fn start() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt()
        .json()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");

    let state = AppState { conn };

    let post_router = Router::new()
        .route("/", get(list_posts).post(create_post))
        .route("/:id", get(get_post).put(update_post).delete(delete_post));
    let api_posts = Router::new().nest("/posts", post_router);

    let app = Router::new()
        .route("/hello", get(hello::hello_world))
        .nest("/api", api_posts)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(state);

    let addr = SocketAddr::from_str(&server_url).unwrap();
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Params {
    page: Option<u64>,
    page_size: Option<u64>,
}

async fn list_posts(state: State<AppState>, Query(params): Query<Params>) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(5);

    let (posts, num_pages) = QueryCore::find_posts_in_page(&state.conn, page, page_size)
        .await
        .expect("Cannot find posts in page");

    Json(posts)
}

async fn create_post(state: State<AppState>, Json(body): Json<post::Model>) -> impl IntoResponse {
    MutationCore::create_post(&state.conn, body)
        .await
        .expect("cannot create post");
    StatusCode::CREATED
}

async fn update_post(
    state: State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<post::Model>,
) -> impl IntoResponse {
    MutationCore::update_post_by_id(&state.conn, id, body)
        .await
        .expect("could not update post");

    StatusCode::OK
}

async fn get_post(state: State<AppState>, Path(id): Path<i32>) -> impl IntoResponse {
    let post = QueryCore::find_post_by_id(&state.conn, id)
        .await
        .expect("could not find post");

    Json(post)
}

async fn delete_post(
    state: State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    MutationCore::delete_post(&state.conn, id)
        .await
        .expect("delete post failed");
    Ok(())
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
