
use axum::{routing::{get, post}, Router, response::{Redirect, Html}, http::uri::Uri, extract::Form};
use url::form_urlencoded;
use reqwest;
use dotenv::dotenv;
use axum_extra::extract::cookie::{CookieJar, Cookie};
use tower_http::services::{ServeDir, ServeFile};
use tera::{Tera, Context};
use axum::extract::{State, Path, Query};
use serde_json::{json, Value};
use serde::Deserialize;
use edgedb_tokio::Client as EdgeClient;
use time::OffsetDateTime;
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};

mod db;

#[derive(Clone)]
struct AppState {
    template: Tera,
    database: EdgeClient
}

// const BASE_URL: &str = "http://localhost:3001";
const BASE_URL: &str = "https://si8ska1o.pemonlist.com";

async fn index(State(state): State<AppState>) -> Html<String> {
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

async fn leaderboard(State(state): State<AppState>) -> Html<String> {
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

async fn level(State(state): State<AppState>, Path(level_id): Path<u64>) -> Html<String> {
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

async fn player(State(state): State<AppState>, Path(username): Path<String>) -> Html<String> {
    let mut ctx = Context::new();

    let player: Value = state.database.query_json("select Player {
        name,
        points,
        verifications := (select Level { name } filter .verifier = <Player>Player.id),
        records := (select .entries {
            level: { name, level_id, placement },
            time_format := (select to_str(.time, \"FMHH24:MI:SS\")),
            time_ms := (select to_str(.time, \"MS\")),
            video_id,
            rank
        } order by .level.placement),
        rank,
        device
    } filter .name = <str>$0", &(username,)).await.unwrap().parse().unwrap();

    ctx.insert("player", &player.as_array().unwrap()[0]);
    state.template.render("player.html", &ctx).unwrap().into()
}

async fn submit(State(state): State<AppState>) -> Html<String> {
    let ctx = Context::new();

    state.template.render("submit.html", &ctx).unwrap().into()
}

async fn terms(State(state): State<AppState>) -> Html<String> {
    state.template.render("terms.html", &Context::new()).unwrap().into()
}

async fn privacy(State(state): State<AppState>) -> Html<String> {
    state.template.render("privacy.html", &Context::new()).unwrap().into()
}

async fn rules(State(state): State<AppState>) -> Html<String> {
    state.template.render("rules.html", &Context::new()).unwrap().into()
}

async fn oauth(State(state): State<AppState>) -> Html<String> {
    state.template.render("oauth.html", &Context::new()).unwrap().into()
}

#[derive(Deserialize)]
struct Oauth2 {
    code: String
}

trait Token {
    async fn get_info_from_token(&self, token_string: &str) -> Value;
}

impl Token for edgedb_tokio::Client {
    async fn get_info_from_token(&self, token_string: &str) -> Value {
        self.query_json("
            select AuthToken {
                account: {
                    id,
                    image,
                    profile_shape,
                    status,
                    player: {
                        id, 
                        name,
                        points,
                        verifications := (select Level { name } filter .verifier = <Player>AuthToken.account.player.id),
                        records := (select .entries {
                            level: { name, level_id, placement },
                            time_format := (select to_str(.time, \"FMHH24:MI:SS\")),
                            time_ms := (select to_str(.time, \"MS\")),
                            video_id,
                            rank
                        } order by .level.placement),
                        rank,
                        device
                    }
                }
            } filter .token = <str>$0 and .expires > <datetime>datetime_of_statement()
        ", &(token_string,)).await.unwrap().parse().unwrap()
    }
}

async fn account(State(state): State<AppState>, oauth2: Option<Query<Oauth2>>, mut jar: CookieJar) -> Result<Html<String>, (CookieJar, Redirect)> {
    let mut ctx = Context::new();

    let client_id = std::env::var("GOOGLE_OAUTH2_CLIENT_ID").expect("Must set GOOGLE_OAUTH2_CLIENT_ID");
    let client_secret = std::env::var("GOOGLE_OAUTH2_CLIENT_SECRET").expect("Must set GOOGLE_OAUTH2_CLIENT_SECRET");

    let query = form_urlencoded::Serializer::new(String::new())
    .append_pair("scope", "email")
    .append_pair("access_type", "offline")
    .append_pair("response_type", "code")
    .append_pair("redirect_uri", &format!("{BASE_URL}/account"))
    .append_pair("client_id", &client_id)
    .finish();

    let uri = format!("https://accounts.google.com/o/oauth2/v2/auth?{query}").parse::<Uri>().unwrap();

    match jar.get("token") {
        Some(ref cookie) => {
            let token = state.database.get_info_from_token(cookie.value()).await;

            if !token[0]["account"].is_null() {
                ctx.insert("account", &token[0]["account"]);

                if token[0]["account"]["status"].as_str() == Some("None") {
                    return Err((jar, Redirect::to("/account/setup")));
                }

                let migration: Value = state.database.query_json("
                    select MigrationRequest {
                        id,
                        requested := (select to_str(.created_at, \"FMDD FMMonth, HH24:MI\")),
                        discord: { global_name, user_id, username, avatar, accent_color, banner }
                    } filter .account.id = <uuid><str>$0
                ", &(token[0]["account"]["id"].as_str().unwrap(),)).await.unwrap().parse().unwrap();

                println!("{migration:#?}");

                ctx.insert("migration", &migration[0]);

                return Ok(state.template.render("account.html", &ctx).unwrap().into())
            }
        }

        None => {}
    }

    match oauth2 {
        Some(ref oauth2) => {
            let params: [(&str, &str); 5] = [
                ("client_id", &client_id),
                ("client_secret", &client_secret),
                ("code", &oauth2.code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", &format!("{BASE_URL}/account"))
            ];

            let client = reqwest::Client::new();
            let token = client.post("https://oauth2.googleapis.com/token")
                .form(&params)
                .send().await.unwrap()
                .text().await.unwrap();

            let token_json: Value = serde_json::from_str(token.as_str()).unwrap();

            if token_json["access_token"].is_null() {
                return Err((jar, Redirect::to(&uri.to_string())));
            }

            let access_token = token_json["access_token"].as_str().unwrap();

            let userdata = client.get("https://www.googleapis.com/oauth2/v3/userinfo")
                .header("Authorization", format!("Bearer {}", access_token))
                .send().await.unwrap()
                .text().await.unwrap();

            let userdata_json: Value = serde_json::from_str(userdata.as_str()).unwrap();

            let token = rand::thread_rng().sample_iter(&Alphanumeric)
                .take(64)
                .map(char::from)
                .collect::<String>();            

            jar = jar.add(
                Cookie::build((
                    "token",
                    token.to_string()
                ))
                .expires(OffsetDateTime::from_unix_timestamp(Utc::now().timestamp() + (7 * 24 * 60 * 60)).unwrap()) 
            );

            let account: Value = state.database.query_json("
                select Account { id } filter .email = <str>$0
            ", &(userdata_json["email"].as_str().unwrap(),)).await.unwrap().parse().unwrap();

                
            let mut account_uuid = account[0]["id"].clone();

            if account_uuid.is_null() {
                let created_account: Value = state.database.query_json("
                    insert Account {
                        email :=  <str>$0,
                        oauth2 :=  <str>$1,
                        image :=  <str>$2,
                        player := <default::Player>{}
                    };
                ", &(
                    userdata_json["email"].as_str().unwrap(),
                    &oauth2.code,
                    userdata_json["picture"].as_str().unwrap().strip_suffix("=s96-c").unwrap())
                ).await.unwrap().parse().unwrap();

                account_uuid = created_account[0]["id"].clone();
            }

            state.database.execute("
                insert AuthToken {
                    token := <str>$0,
                    account := <Account><uuid><str>$1
                }
            ", &(token.as_str(), account_uuid.as_str().unwrap())).await.unwrap();

            Err((jar, Redirect::to(&"/account")))
        }

        None => {
            Err((jar, Redirect::to(&uri.to_string())))
        }
    }
}

async fn setup(State(state): State<AppState>, jar: CookieJar) -> Result<Html<String>, Redirect> {
    let mut ctx = Context::new();

    match jar.get("token") {
        Some(ref cookie) => {
            let token = state.database.get_info_from_token(cookie.value()).await;

            if !token[0]["account"].is_null() && token[0]["account"]["status"].as_str() == Some("None") {
                ctx.insert("account", &token[0]["account"]);
                ctx.insert("token", cookie.value());

                return Ok(state.template.render("setup.html", &ctx).unwrap().into());
            }
        }

        None => {}
    };

    Err(Redirect::to("/account"))
}

#[derive(Deserialize)]
struct SetupInfo {
    token: String,
    username: String,
    profileshape: String,
    device: String
}

async fn setup_account(State(state): State<AppState>, Form(mut body): Form<SetupInfo>) -> Redirect {
    let player: Value = state.database.query_json("
        select Player { id } filter .name = <str>$0
    ", &(&body.username,)).await.unwrap().parse().unwrap();

    if !player[0]["id"].is_null() {
        return Redirect::to(&format!("/account/migrate?username={}&profileshape={}&device={}", &body.username, &body.profileshape, &body.device))
    }

    let account: Value = state.database.query_json("
        select AuthToken {
            account: { id }
        } filter .token = <str>$0 and .expires > <datetime>datetime_of_statement()
    ", &(&body.token,)).await.unwrap().parse().unwrap();

    if account[0]["account"]["id"].is_null() {
        return Redirect::to("/account")
    }

    let player: Value = state.database.query_json("
        insert Player {
            name := <str>$0,
            device := <Device><str>$1
        }
    ", &(
        &body.username,
        format!("{}{}", &body.device.remove(0).to_uppercase(), &body.device)
    )).await.unwrap().parse().unwrap();

    state.database.execute("
        update Account filter .id = <uuid><str>$0 set {
            status := AccountStatus.Done,
            player := <Player><uuid><str>$1,
            profile_shape := <ProfileShape><str>$2
        }
    ", &(
        account[0]["account"]["id"].as_str(),
        player[0]["id"].as_str(),
        format!("{}{}", &body.profileshape.remove(0).to_uppercase(), &body.profileshape)
    )).await.unwrap();

    Redirect::to("/account")
}

#[derive(Debug, Deserialize)]
struct SetupInfoQuery {
    username: String,
    profileshape: String,
    device: String
}

async fn migrate(State(state): State<AppState>, info: Option<Query<SetupInfoQuery>>, jar: CookieJar) -> Result<Html<String>, Redirect> {
    let mut ctx = Context::new();

    match info {
        Some(ref query) => {
            ctx.insert("setupinfo", &json!({
                "username": &query.username,
                "profileshape": &query.profileshape,
                "device": &query.device
            }))
        }

        None => {}
    };

    match jar.get("token") {
        Some(ref cookie) => {
            let token = state.database.get_info_from_token(cookie.value()).await;

            if !token[0]["account"].is_null() && token[0]["account"]["status"].as_str() == Some("None") {
                ctx.insert("account", &token[0]["account"]);
                ctx.insert("token", cookie.value());

                return Ok(state.template.render("migrate.html", &ctx).unwrap().into());
            }
        }

        None => {}
    }

    Err(Redirect::to("/account"))
}

#[derive(Deserialize)]
struct MigrateInfo {
    token: String,
    username: String,
    profileshape: String,
    device: String,
    discord: String
}

async fn migrate_account(State(state): State<AppState>, Form(mut body): Form<MigrateInfo>) -> Redirect {
    let player: Value = state.database.query_json("
        select Player { id } filter .name = <str>$0
    ", &(&body.username,)).await.unwrap().parse().unwrap();

    if player[0]["id"].is_null() {
        return Redirect::to("/account/setup")
    }

    let account: Value = state.database.query_json("
        select AuthToken {
            account: { id }
        } filter .token = <str>$0 and .expires > <datetime>datetime_of_statement()
    ", &(&body.token,)).await.unwrap().parse().unwrap();

    if account[0]["account"]["id"].is_null() {
        return Redirect::to("/account")
    }

    let client_id = std::env::var("DISCORD_OAUTH2_CLIENT_ID").expect("Must set DISCORD_OAUTH2_CLIENT_ID");
    let client_secret = std::env::var("DISCORD_OAUTH2_CLIENT_SECRET").expect("Must set DISCORD_OAUTH2_CLIENT_SECRET");

    let params: [(&str, &str); 5] = [
        ("client_id", &client_id),
        ("client_secret", &client_secret),
        ("grant_type", "authorization_code"),
        ("code", &body.discord),
        ("redirect_uri", &format!("{BASE_URL}/oauth"))
    ];

    let client = reqwest::Client::new();

    let token = client.post("https://discord.com/api/v10/oauth2/token")
        .form(&params)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send().await.unwrap()
        .text().await.unwrap();

    let token_json: Value = serde_json::from_str(&token).unwrap();

    if token_json["access_token"].is_null() {
        return Redirect::to("/account/migrate")
    }

    let user = client.get("https://discord.com/api/v10/users/@me")
        .header("Authorization", format!("Bearer {}", token_json["access_token"].as_str().unwrap()))
        .send().await.unwrap()
        .text().await.unwrap();

    let user_json: Value = serde_json::from_str(&user).unwrap();

    if user_json["id"].is_null() {
        return Redirect::to("/account/migrate")
    }

    let mut accent_color = "000000".to_string();
    let mut banner = "";

    if !user_json["accent_color"].is_null() {
        accent_color = format!("{:06X}", user_json["accent_color"].as_u64().unwrap())
    }

    if !user_json["banner"].is_null() {
        banner = user_json["banner"].as_str().unwrap()
    }

    let discord: Value = state.database.query_required_single_json("
        insert Discord {
            user_id := <str>$0,
            username := <str>$1,
            global_name := <str>$2,
            avatar := <str>$3,
            accent_color := <str>$4,
            banner := <str>$5
        }
    ", &(
        user_json["id"].as_str().unwrap(),
        user_json["username"].as_str().unwrap(),
        user_json["global_name"].as_str().unwrap(),
        user_json["avatar"].as_str().unwrap(),
        accent_color.as_str(),
        banner
    )).await.unwrap().parse().unwrap();

    state.database.execute("
        insert MigrationRequest {
            discord := <Discord><uuid><str>$0,
            account := <Account><uuid><str>$1,
            player := <Player><uuid><str>$2
        }
    ", &(
        discord["id"].as_str().unwrap(),
        account[0]["account"]["id"].as_str().unwrap(),
        player[0]["id"].as_str().unwrap()
    )).await.unwrap();

    state.database.execute("
        update Account filter .id = <uuid><str>$0 set {
            status := AccountStatus.Migrating,
            profile_shape := <ProfileShape><str>$1
        }
    ", &(
        account[0]["account"]["id"].as_str().unwrap(),
        format!("{}{}", &body.profileshape.remove(0).to_uppercase(), &body.profileshape)
    )).await.unwrap();

    state.database.execute("
        update Player filter .id = <uuid><str>$0 set {
            device := <Device><str>$1
        }
    ", &(
        player[0]["id"].as_str().unwrap(),
        format!("{}{}", &body.device.remove(0).to_uppercase(), &body.device)
    )).await.unwrap();

    Redirect::to("/account")
}

#[tokio::main]
async fn main() {
    dotenv().ok();

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
        .route("/account", get(account))
        .route("/account/setup", get(setup))
        .route("/account/setup", post(setup_account))
        .route("/account/migrate", get(migrate))
        .route("/account/migrate", post(migrate_account))
        .route("/terms", get(terms))
        .route("/privacy", get(privacy))
        .route("/rules", get(rules))
        .route("/oauth", get(oauth))

        .route_service("/favicon.ico", ServeFile::new("site/meta/favicon.ico"))

        .nest_service("/src", ServeDir::new("site/src"))
        .nest_service("/meta", ServeDir::new("site/meta"))
        .with_state(state);

    // Set up 

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}