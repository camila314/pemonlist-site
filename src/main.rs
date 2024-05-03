
use axum::{routing::{get, post}, Router, response::{Redirect, Html}, http::uri::Uri, extract::Form};
use url::form_urlencoded;
use reqwest;
use dotenv::dotenv;
use axum_extra::extract::cookie::{CookieJar, Cookie};
use tower_http::services::{ServeDir, ServeFile};
use tera::{Tera, Context};
use axum::extract::{State, Path, Query};
use serde_json::Value;
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
                    setup,
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

// const BASE_URL: &str = "http://localhost:3001";
const BASE_URL: &str = "https://si8ska1o.pemonlist.com";

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
        Some(ref value) => {
            println!("token_cookie: {:?}", value);

            let token = state.database.get_info_from_token(value.value()).await;

            if !token[0]["account"].is_null() {
                ctx.insert("account", &token[0]["account"]);

                println!("{:?}", &token[0]["account"]);

                if token[0]["account"]["setup"].as_bool() == Some(false) {
                    return Err((jar, Redirect::to("/account/setup")));
                }

                return Ok(state.template.render("account.html", &ctx).unwrap().into())
            }
        }

        None => {}
    }

    match oauth2 {
        Some(ref value) => {
            println!("oauth: {:?}", value.0.code);

            let params: [(&str, &str); 5] = [
                ("client_id", &client_id),
                ("client_secret", &client_secret),
                ("code", &value.0.code),
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

            println!("access_token: {}", access_token);

            let userdata = client.get("https://www.googleapis.com/oauth2/v3/userinfo")
                .header("Authorization", format!("Bearer {}", access_token))
                .send().await.unwrap()
                .text().await.unwrap();

            let userdata_json: Value = serde_json::from_str(userdata.as_str()).unwrap();

            println!("userdata: {:?}", userdata_json);

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

            println!("fetched uuid: {:?}", account_uuid);

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
                    &value.0.code,
                    userdata_json["picture"].as_str().unwrap().strip_suffix("=s96-c").unwrap())
                ).await.unwrap().parse().unwrap();

                account_uuid = created_account[0]["id"].clone();
            }

            println!("new uuid: {:?}", account_uuid);

            state.database.execute("
                insert AuthToken {
                    token := <str>$0,
                    account := <Account><uuid><str>$1
                }
            ", &(token.as_str(), account_uuid.as_str().unwrap())).await.unwrap();

            println!("token: {}", &token);

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
        Some(ref value) => {
            println!("token_cookie: {:?}", value);

            let token = state.database.get_info_from_token(value.value()).await;

            if !token[0]["account"].is_null() && token[0]["account"]["setup"].as_bool() == Some(false) {
                ctx.insert("account", &token[0]["account"]);
                ctx.insert("token", value.value());

                println!("{:?}", &token[0]["account"]);

                return Ok(state.template.render("setup.html", &ctx).unwrap().into());
            }
        }

        None => {}
    };

    Err(Redirect::to("/account"))
}

#[derive(Debug, Deserialize)]
struct SetupInfo {
    token: String,
    username: String,
    profileshape: String,
    device: String
}

async fn setup_account(State(state): State<AppState>, Form(mut body): Form<SetupInfo>) -> Redirect {
    let exists: Value = state.database.query_json("
        select Player { id } filter .name = <str>$0
    ", &(&body.username,)).await.unwrap().parse().unwrap();

    println!("exists: {exists:#?}");

    if !exists[0]["id"].is_null() {
        return Redirect::to("/account/migrate")
    }

    let account: Value = state.database.query_json("
        select AuthToken {
            account: { id }
        } filter .token = <str>$0 and .expires > <datetime>datetime_of_statement()
    ", &(&body.token,)).await.unwrap().parse().unwrap();

    println!("account: {account:#?}");

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

    println!("player: {player:#?}");

    state.database.execute("
        update Account filter .id = <uuid><str>$0 set {
            setup := <bool>true,
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

async fn migrate(State(state): State<AppState>) -> Result<Html<String>, Redirect> {
    Err(Redirect::to("/account"))
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
        .route("/terms", get(terms))
        .route("/privacy", get(privacy))
        .route("/rules", get(rules))

        .route_service("/favicon.ico", ServeFile::new("site/meta/favicon.ico"))

        .nest_service("/src", ServeDir::new("site/src"))
        .nest_service("/meta", ServeDir::new("site/meta"))
        .with_state(state);

    // Set up 

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}