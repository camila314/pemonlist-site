
use axum::{routing::get, Router};
use tower_http::services::{ServeDir, ServeFile};
use tera::{Tera, Context};
use axum::extract::{State, Path};
use serde_json::Value;
use edgedb_tokio::Client as EdgeClient;

mod db;

#[derive(Clone)]
struct AppState {
    template: Tera,
    database: EdgeClient
}

async fn index(State(state): State<AppState>) -> axum::response::Html<String> {
    let mut ctx = Context::new();

    let levels: Value = state.database.query_json("select Level {
        name,
        creator,
        video_id,
        level_id,
        record := (select .entries {
            name := (select .player.name),
            time_format := (select to_str(.time, \"FMHH24:MI:SS\")),
            time_ms := (select to_str(.time, \"MS\"))
        } order by .time limit 1)
    } order by .placement", &()).await.unwrap().parse().unwrap();

    ctx.insert("levels", &levels);
    state.template.render("index.html", &ctx).unwrap().into()
}

async fn leaderboard(State(state): State<AppState>) -> axum::response::Html<String> {
    let mut ctx = Context::new();

    let players: Value = state.database.query_json("select Player {
        name,
        points,
        rank,
        device
    } filter .points > 0 order by .points desc", &()).await.unwrap().parse().unwrap();

    ctx.insert("players", &players);
    state.template.render("leaderboard.html", &ctx).unwrap().into()
}

async fn level(State(state): State<AppState>, Path(level_id): Path<u64>) -> axum::response::Html<String> {
    let mut ctx = Context::new();

    let level: Value = state.database.query_json("select Level {
        name,
        creator,
        video_id,
        verifier: { name },
        placement,
        points,
        level_id,
        records := (select .entries {
            name := .player.name,
            time_format := (select to_str(.time, \"FMHH24:MI:SS\")),
            time_ms := (select to_str(.time, \"MS\")),
            video_id,
            mobile,
            rank
        } order by .time)
    } filter .level_id = <int64>$0", &(level_id as i64,)).await.unwrap().parse().unwrap();

    ctx.insert("level", &level.as_array().unwrap()[0]);
    state.template.render("level.html", &ctx).unwrap().into()
}

async fn player(State(state): State<AppState>, Path(username): Path<String>) -> axum::response::Html<String> {
    let mut ctx = Context::new();

    let player: Value = state.database.query_json("select Player {
        name,
        points,
        verifications := (select Level { name } filter .verifier = <Player>Player.id),
        records := (select .entries {
            level: { name, level_id },
            time_format := (select to_str(.time, \"FMHH24:MI:SS\")),
            time_ms := (select to_str(.time, \"MS\"))
        } order by .rank),
        rank,
        device
    } filter .name = <str>$0", &(username,)).await.unwrap().parse().unwrap();

    ctx.insert("player", &player.as_array().unwrap()[0]);
    state.template.render("player.html", &ctx).unwrap().into()
}

async fn submit(State(state): State<AppState>) -> axum::response::Html<String> {
    let ctx = Context::new();
    state.template.render("submit.html", &ctx).unwrap().into()
}

#[tokio::main]
async fn main() {
    let state = AppState { 
        template: Tera::new("site/*.html").unwrap(),
        database: edgedb_tokio::create_client().await.unwrap(),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/level/:id", get(level))
        .route("/leaderboard", get(leaderboard))
        .route("/player/:username", get(player))
        .route("/submit", get(submit))
        .route_service("/rules", ServeFile::new("site/rules.html"))
        .nest_service("/src", ServeDir::new("site/src"))
        .nest_service("/meta", ServeDir::new("site/meta"))
        .route_service("/favicon.ico", ServeFile::new("site/meta/favicon.ico"))
        .with_state(state);

    // Set up 

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
