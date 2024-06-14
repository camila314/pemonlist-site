use axum::{
    extract::{Form, Host, Path, Query, State},
    http::uri::Uri,
    response::{Html, Redirect},
    routing::{get, post},
    Router
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use chrono::Utc;
use dotenv::dotenv;
use edgedb_tokio::Client as EdgeClient;
use futures::executor::block_on;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{self, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::RwLock;
use std::thread;
use tera::{Context, Tera};
use time::OffsetDateTime;
use std::time::Instant;
use tower_http::services::{ServeDir, ServeFile};
use url::form_urlencoded;

#[derive(Clone)]
struct AppState {
    template: Tera,
    database: EdgeClient
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
                    mod,
                    player: {
                        id, 
                        name,
                        points,
                        verifications: { name, level_id, placement, video_id },
                        records := (select .entries {
                            level: { name, level_id, placement },
                            time_format := (select to_str(.time, \"FMHH24:MI:SS\")),
                            time_ms := (select to_str(.time, \"MS\")),
                            video_id,
                            rank
                        } order by .level.placement),
                        unverified_records := (select .unverified_entries {
                            id,
                            level: { name, level_id, placement },
                            time_format := (select to_str(.time, \"FMHH24:MI:SS\")),
                            time_ms := (select to_str(.time, \"MS\")),
                            video_id,
                            status,
                            reason
                        } order by .level.placement),
                        rank,
                        device
                    }
                }
            } filter .token = <str>$0 and .expires > <datetime>datetime_of_statement()
        ", &(token_string,)).await.unwrap().parse().unwrap()
    }
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
        } filter .status = Status.Approved  order by .time limit 1),
        placement
    } order by .placement", &()).await.unwrap().parse().unwrap();

    ctx.insert("levels", &levels);
    state.template.render("index.html", &ctx).unwrap().into()
}

static LEADERBOARD_CACHE: RwLock<Value> = RwLock::new(json!([]));
static RECORDS: RwLock<i64> = RwLock::new(0);

async fn leaderboard(State(state): State<AppState>) -> Html<String> {
    let mut ctx = Context::new();

    let mut players = LEADERBOARD_CACHE.read().unwrap().clone();

    if players.clone().as_array().unwrap().is_empty() {
        players = state.database.query_json("select Player {
            id,
            rank,
            points
        } filter .points > 0 order by .points desc", &()).await.unwrap().parse().unwrap();

        let mut guard = LEADERBOARD_CACHE.write().unwrap();
        *guard = players.clone();
    } else {
        let records: i64 = state.database.query_required_single_json("
            select count((select Entry filter .status = Status.Approved))
        ", &()).await.unwrap().parse::<Value>().unwrap().as_i64().unwrap();

        if records != RECORDS.read().unwrap().clone() {
            let threadstate = state.clone();
            thread::spawn(move || {
                let players = threadstate.database.query_json("select Player {
                    id,
                    rank,
                    points
                } filter .points > 0 order by .points desc", &());

                let new_players = block_on(players).unwrap().parse::<Value>().unwrap();

                let mut guard = LEADERBOARD_CACHE.write().unwrap();
                *guard = new_players.clone();
                drop(guard);

                let mut guard = RECORDS.write().unwrap();
                *guard = records.clone();
                drop(guard);
            });
        }
    }

    let leaderboard: Value = state.database.query_json("select Player {
        id,
        name,
        device
    }", &()).await.unwrap().parse().unwrap();

    let mut i = 0;

    if let Some(players_arr) = players.clone().as_array() {
        if let Some(leaderboard_arr) = leaderboard.as_array() {
            for player in players_arr {
    
                for leaderboard_spot in leaderboard_arr {
                    if leaderboard_spot["id"].as_str().unwrap() == player["id"].as_str().unwrap() {
                        players[i]["name"] = leaderboard_spot["name"].clone();
                        players[i]["device"] = leaderboard_spot["device"].clone();
                        break;
                    }
                }
                
                i += 1;
            }
        }
    }

    ctx.insert("players", &players.clone());

    state.template.render("leaderboard.html", &ctx).unwrap().into()
}

async fn level(State(state): State<AppState>, Path(level_id): Path<u64>) -> (StatusCode, Html<String>) {
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
        } filter .status = Status.Approved order by .time)
    } filter .level_id = <int64>$0", &(level_id as i64,)).await.unwrap().parse().unwrap();

    if level[0]["name"].is_null() {
        let mut ctx = Context::new();
        ctx.insert("status", "400: Bad Request");

        return (StatusCode::BAD_REQUEST, state.template.render("fallback.html", &ctx).unwrap().into())
    }

    ctx.insert("level", &level.as_array().unwrap()[0]);
    (StatusCode::OK, state.template.render("level.html", &ctx).unwrap().into())
}

async fn player(State(state): State<AppState>, jar: CookieJar, Path(username): Path<String>) -> Result<(StatusCode, Html<String>), Redirect> {
    let mut ctx = Context::new();

    let player: Value = state.database.query_json("select Player {
        name,
        points,
        verifications: { name, level_id, placement, video_id },
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

    if player[0]["name"].is_null() {
        let mut ctx = Context::new();
        ctx.insert("status", "400: Bad Request");

        return Ok((StatusCode::BAD_REQUEST, state.template.render("fallback.html", &ctx).unwrap().into()))
    }

    match jar.get("token") {
        Some (ref cookie) => {
            let token = state.database.query_json("
                select AuthToken {
                    account: { player: { name } }
                } filter .token = <str>$0 and .expires > <datetime>datetime_of_statement()
            ", &(cookie.value(),)).await.unwrap().parse::<Value>().unwrap();

            if !token[0]["account"]["player"]["name"].is_null() {
                let names_match = token[0]["account"]["player"]["name"].as_str().unwrap() == player[0]["name"].as_str().unwrap();

                if names_match {
                    return Err(Redirect::to("/account"));
                }
            }
        }

        None => {}
    }

    ctx.insert("player", &player.as_array().unwrap()[0]);
    Ok((StatusCode::OK, state.template.render("player.html", &ctx).unwrap().into()))
}

async fn submit(State(state): State<AppState>, jar: CookieJar) -> Result<Html<String>, Redirect> {
    let mut ctx = Context::new();

    match jar.get("token") {
        Some (ref cookie) => {
            let token = state.database.get_info_from_token(cookie.value()).await;

            if token[0]["account"].is_null() || token[0]["account"]["status"].as_str() != Some("Done") {
                return Err(Redirect::to("/account"));
            }
        }

        None => {
            return Err(Redirect::to("/account"));
        }
    }

    let levels: Value = state.database.query_json("select Level {
        name, level_id, placement
    } order by .placement", &()).await.unwrap().parse().unwrap();

    ctx.insert("levels", &levels);
    Ok(state.template.render("submit.html", &ctx).unwrap().into())
}

#[derive(Deserialize)]
struct RecordInfo {
    time: String,
    levelid: String,
    videoid: String,
    raw: String,
    device: String
}

async fn submit_record(State(state): State<AppState>, jar: CookieJar, Form(body): Form<RecordInfo>) -> Result<Html<String>, Redirect> {
    let token: &str;

    match jar.get("token") {
        Some (ref cookie) => {
            token = cookie.value()
        }

        None => {
            return Err(Redirect::to("/account"))
        }
    }

    let account: Value = state.database.query_json("
        select AuthToken {
            account: { id, player: { id } }
        } filter .token = <str>$0 and .expires > <datetime>datetime_of_statement()
    ", &(token,)).await.unwrap().parse().unwrap();

    if account[0]["account"]["id"].is_null() {
        return Err(Redirect::to("/account"));
    }

    let mut video_id = &body.videoid;

    if video_id.is_empty() {
        video_id = &body.raw
    }

    let entry: Value = state.database.query_json("
        select Entry { id, faster := .time > <duration><str>$0 } filter
            .video_id = <str>$1 and
            .level = (select Level filter .level_id = <int64><str>$2) and
            .status != Status.Denied
    ", &(
        &body.time,
        video_id,
        &body.levelid
    )).await.unwrap().parse().unwrap();

    if !entry[0]["id"].is_null() && entry[0]["faster"].as_bool() != Some(true) {
        return Ok(state.template.render("duplicate.html", &Context::new()).unwrap().into());
    }

    state.database.execute("
        insert Entry {
            status := Status.Waiting,
            video_id := <str>$0,
            raw_video := <str>$1,
            player := <Player><uuid><str>$2,
            level := (select Level filter .level_id = <int64><str>$3),
            time := <duration><str>$4,
            mobile := <bool>$5
        }
    ", &(
        video_id,
        &body.raw,
        account[0]["account"]["player"]["id"].as_str().unwrap(),
        &body.levelid,
        &body.time,
        &body.device == "mobile"
    )).await.unwrap();

    Ok(state.template.render("submitted.html", &Context::new()).unwrap().into())
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

async fn account(State(state): State<AppState>, oauth2: Option<Query<Oauth2>>, mut jar: CookieJar, Host(host): Host) -> Result<Html<String>, (CookieJar, Redirect)> {
    let mut ctx = Context::new();

    let client_id = std::env::var("GOOGLE_OAUTH2_CLIENT_ID").expect("Must set GOOGLE_OAUTH2_CLIENT_ID");
    let client_secret = std::env::var("GOOGLE_OAUTH2_CLIENT_SECRET").expect("Must set GOOGLE_OAUTH2_CLIENT_SECRET");

    let query = form_urlencoded::Serializer::new(String::new())
    .append_pair("scope", "email")
    .append_pair("access_type", "offline")
    .append_pair("response_type", "code")
    .append_pair("redirect_uri", &format!("https://{host}/account"))
    .append_pair("client_id", &client_id)
    .finish();

    let uri = format!("https://accounts.google.com/o/oauth2/v2/auth?{query}").parse::<Uri>().unwrap();

    match jar.get("token") {
        Some (ref cookie) => {
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

                ctx.insert("migration", &migration[0]);

                return Ok(state.template.render("account.html", &ctx).unwrap().into())
            }
        }

        None => {}
    }

    let code: &str;

    match oauth2 {
        Some (ref oauth2) => {
            code = &oauth2.code.as_str();
        }

        None => {
            return Err((jar, Redirect::to(&uri.to_string())))
        }
    }

    let params: [(&str, &str); 5] = [
        ("client_id", &client_id),
        ("client_secret", &client_secret),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", &format!("https://{host}/account"))
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
            code,
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

#[derive(Deserialize)]
struct AccountUpdate {
    method: String,
    id: Option<String>
}

async fn update_account(State(state): State<AppState>, jar: CookieJar, Form(body): Form<AccountUpdate>) -> Redirect {
    let token: &str;

    match jar.get("token") {
        Some(ref cookie) => {
            token = cookie.value()
        }

        None => {
            return Redirect::to("/account")
        }
    }

    let info = state.database.query_json("
        select AuthToken {
            account: { id, mod }
        } filter .token = <str>$0 and .expires > <datetime>datetime_of_statement()
    ", &(token,)).await.unwrap().parse::<Value>().unwrap();

    if info[0]["account"]["id"].is_null() {
        return Redirect::to("/account");
    }

    match body.method.clone().as_str() {
        "deleterecord" => {
            state.database.execute("
                delete Entry filter .id = <uuid><str>$0
            ", &(&body.id.unwrap(),)).await.unwrap()
        }
        _ => {}
    }

    Redirect::to("/account")
}

async fn setup(State(state): State<AppState>, jar: CookieJar) -> Result<Html<String>, Redirect> {
    let mut ctx = Context::new();

    match jar.get("token") {
        Some (ref cookie) => {
            let token = state.database.get_info_from_token(cookie.value()).await;

            if !token[0]["account"].is_null() && token[0]["account"]["status"].as_str() == Some("None") {
                ctx.insert("account", &token[0]["account"]);

                return Ok(state.template.render("setup.html", &ctx).unwrap().into());
            }
        }

        None => {}
    };

    Err(Redirect::to("/account"))
}

#[derive(Deserialize)]
struct SetupInfo {
    username: String,
    profileshape: String,
    device: String
}

async fn setup_account(State(state): State<AppState>, jar: CookieJar, Form(mut body): Form<SetupInfo>) -> Redirect {
    let token: &str;

    match jar.get("token") {
        Some (ref cookie) => {
            token = cookie.value()
        }

        None => {
            return Redirect::to("/account")
        }
    }

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
    ", &(&token,)).await.unwrap().parse().unwrap();

    if account[0]["account"]["id"].is_null() {
        return Redirect::to("/account")
    }

    if body.username.clone().trim().len() > 25 {
        return Redirect::to("/account/setup");
    }

    let player: Value = state.database.query_json("
        insert Player {
            name := <str>$0,
            device := <Device><str>$1
        }
    ", &(
        body.username.clone().trim(),
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

#[derive(Deserialize)]
struct SetupInfoQuery {
    username: String,
    profileshape: String,
    device: String
}

async fn migrate(State(state): State<AppState>, info: Option<Query<SetupInfoQuery>>, jar: CookieJar) -> Result<Html<String>, Redirect> {
    let mut ctx = Context::new();

    match info {
        Some (ref query) => {
            ctx.insert("setupinfo", &json!({
                "username": &query.username,
                "profileshape": &query.profileshape,
                "device": &query.device
            }))
        }

        None => {}
    };

    match jar.get("token") {
        Some (ref cookie) => {
            let token = state.database.get_info_from_token(cookie.value()).await;

            if !token[0]["account"].is_null() && token[0]["account"]["status"].as_str() == Some("None") {
                ctx.insert("account", &token[0]["account"]);

                return Ok(state.template.render("migrate.html", &ctx).unwrap().into());
            }
        }

        None => {}
    }

    Err(Redirect::to("/account"))
}

#[derive(Deserialize)]
struct MigrateInfo {
    username: String,
    profileshape: String,
    device: String,
    discord: String
}

async fn migrate_account(State(state): State<AppState>, jar: CookieJar, Host(host): Host, Form(mut body): Form<MigrateInfo>) -> Redirect {
    let token: &str;

    match jar.get("token") {
        Some (ref cookie) => {
            token = cookie.value()
        }

        None => {
            return Redirect::to("/account")
        }
    }

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
    ", &(token,)).await.unwrap().parse().unwrap();

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
        ("redirect_uri", &format!("https://{host}/oauth"))
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

async fn modpage(State(state): State<AppState>, jar: CookieJar) -> Result<Html<String>, Redirect> {
    let mut ctx = Context::new();

    match jar.get("token") {
        Some (ref cookie) => {
            let token = state.database.get_info_from_token(cookie.value()).await;

            if !token[0]["account"].is_null() && token[0]["account"]["mod"].as_bool().unwrap() {
                ctx.insert("account", &token[0]["account"]);

                return Ok(state.template.render("mod.html", &ctx).unwrap().into());
            }
        }

        None => {}
    }

    Err(Redirect::to("/account"))
}

async fn mod_records(State(state): State<AppState>, jar: CookieJar) -> Result<Html<String>, Redirect> {
    let start = Instant::now();

    let mut ctx = Context::new();

    match jar.get("token") {
        Some (ref cookie) => {
            let token = state.database.get_info_from_token(cookie.value()).await;

            let records = state.database.query_json("
                select Entry {
                    id,
                    video_id,
                    raw_video,
                    time_format := (select to_str(.time, \"FMHH24:MI:SS\")),
                    time_ms := (select to_str(.time, \"MS\")),
                    status,
                    mobile,
                    player: {
                        name,
                        account := (select .<player[is Account] {
                            image,
                            profile_shape,
                            discord: { global_name, user_id, username, avatar }
                        } limit 1)
                    },
                    level: { name, placement, video_id, level_id }
                } filter .status != Status.Approved and .status != Status.Denied order by .created_at asc
            ", &()).await.unwrap().parse::<Value>().unwrap();

            ctx.insert("records", &records.as_array().unwrap());

            if !token[0]["account"].is_null() && token[0]["account"]["mod"].as_bool().unwrap() {
                ctx.insert("account", &token[0]["account"]);
                ctx.insert("elapsed", &((start.clone().elapsed().as_secs_f64() * 100.).round() / 100.));

                return Ok(state.template.render("modrecords.html", &ctx).unwrap().into());
            }
        }

        None => {}
    }

    Err(Redirect::to("/account"))
}

#[derive(Deserialize)]
struct EntryEdit {
    entryid: String,
    time: String,
    status: String,
    reason: String
}

async fn edit_record(State(state): State<AppState>, jar: CookieJar, Form(mut body): Form<EntryEdit>) -> Redirect {
    let token: &str;

    match jar.get("token") {
        Some (ref cookie) => {
            token = cookie.value()
        }

        None => {
            return Redirect::to("/account")
        }
    }

    let info = state.database.query_json("
        select AuthToken {
            account: { id, mod }
        } filter .token = <str>$0 and .expires > <datetime>datetime_of_statement()
    ", &(token,)).await.unwrap().parse::<Value>().unwrap();

    if info[0]["account"]["id"].is_null() {
        return Redirect::to("/account");
    }

    if info[0]["account"]["mod"].as_bool() == Some(false) {
        return Redirect::to("/");
    }

    let entry: Value = state.database.query_json("
        select Entry { level: { id }, player: { id } } filter .id = <uuid><str>$0
    ", &(&body.entryid,)).await.unwrap().parse().unwrap();

    let approved: Value = state.database.query_json("
        select Entry { id, time, faster := .time > <duration><str>$0 } filter
            .level.id = <uuid><str>$1 and
            .player.id = <uuid><str>$2 and
            .status = Status.Approved
    ", &(
        &body.time,
        entry[0]["level"]["id"].as_str().unwrap(),
        entry[0]["player"]["id"].as_str().unwrap()
    )).await.unwrap().parse().unwrap();

    if !approved[0]["id"].is_null() && approved[0]["faster"].as_bool().unwrap() && &body.status == "approved" {
        state.database.execute("
            update Entry filter .id = <uuid><str>$0 set {
                created_at := datetime_of_statement(),
                time := <duration><str>$1,
                status := Status.Approved
            };
            delete Entry filter .id = <uuid><str>$2
        ", &(
            approved[0]["id"].as_str().unwrap(),
            &body.time,
            &body.entryid
        )).await.unwrap();
    } else {
        state.database.execute("
            update Entry filter .id = <uuid><str>$0 set {
                time := <duration><str>$1,
                status := <Status><str>$2,
                reason := <str>$3
            };
            delete Entry filter
                .player.id = <uuid><str>$4 and
                .level.id = <uuid><str>$5
        ", &(
            &body.entryid,
            &body.time,
            format!("{}{}", &body.status.remove(0).to_uppercase(), &body.status),
            &body.reason,
            entry[0]["level"]["id"].as_str().unwrap(),
            entry[0]["player"]["id"].as_str().unwrap()
        )).await.unwrap();
    }

    let records: i64 = state.database.query_required_single_json("
        select count((select Entry filter .status = Status.Approved))
    ", &()).await.unwrap().parse::<Value>().unwrap().as_i64().unwrap();

    if records != RECORDS.read().unwrap().clone() {
        let threadstate = state.clone();
        thread::spawn(move || {
            let players = threadstate.database.query_json("select Player {
                id,
                rank,
                points
            } filter .points > 0 order by .points desc", &());

            let new_players = block_on(players).unwrap().parse::<Value>().unwrap();

            let mut guard = LEADERBOARD_CACHE.write().unwrap();
            *guard = new_players.clone();
            drop(guard);

            let mut guard = RECORDS.write().unwrap();
            *guard = records.clone();
            drop(guard);
        });
    }

    Redirect::to("/mod/records")
}

async fn mod_users(State(state): State<AppState>, jar: CookieJar) -> Result<Html<String>, Redirect> {
    let start = Instant::now();

    let mut ctx = Context::new();

    match jar.get("token") {
        Some (ref cookie) => {
            let token = state.database.get_info_from_token(cookie.value()).await;

            let records = state.database.query_json("
                select MigrationRequest {
                    id,
                    account: {
                        profile_shape,
                        image
                    },
                    player: {
                        name
                    },
                    discord: {
                        global_name,
                        user_id,
                        avatar,
                        username
                    }
                } filter .account.status = AccountStatus.Migrating order by .created_at asc
            ", &()).await.unwrap().parse::<Value>().unwrap();

            ctx.insert("requests", &records.as_array().unwrap());

            if !token[0]["account"].is_null() && token[0]["account"]["mod"].as_bool().unwrap() {
                ctx.insert("account", &token[0]["account"]);
                ctx.insert("elapsed", &((start.clone().elapsed().as_secs_f64() * 100.).round() / 100.));

                return Ok(state.template.render("modusers.html", &ctx).unwrap().into());
            }
        }

        None => {}
    }

    Err(Redirect::to("/account"))
}

#[derive(Deserialize)]
struct UserEdit {
    migrationid: String,
    status: String
}

async fn update_user(State(state): State<AppState>, jar: CookieJar, Form(body): Form<UserEdit>) -> Redirect {
    let token: &str;

    match jar.get("token") {
        Some (ref cookie) => {
            token = cookie.value()
        }

        None => {
            return Redirect::to("/account")
        }
    }

    let info = state.database.query_json("
        select AuthToken {
            account: { id, mod }
        } filter .token = <str>$0 and .expires > <datetime>datetime_of_statement()
    ", &(token,)).await.unwrap().parse::<Value>().unwrap();

    if info[0]["account"]["id"].is_null() {
        return Redirect::to("/account");
    }

    if info[0]["account"]["mod"].as_bool() == Some(false) {
        return Redirect::to("/");
    }

    let request = state.database.query_json("
        select MigrationRequest {
            discord: { id },
            account: { id },
            player: { id }
        } filter .id = <uuid><str>$0
    ", &(&body.migrationid,)).await.unwrap().parse::<Value>().unwrap();

    match body.status.clone().as_str() {
        "accept" => {
            state.database.execute("
                delete MigrationRequest filter .id = <uuid><str>$0;
                update Account filter .id = <uuid><str>$1 set {
                    discord := <Discord><uuid><str>$2,
                    player := <Player><uuid><str>$3,
                    status := AccountStatus.Done
                }
            ", &(
                &body.migrationid,
                request[0]["account"]["id"].as_str().unwrap(),
                request[0]["discord"]["id"].as_str().unwrap(),
                request[0]["player"]["id"].as_str().unwrap()
            )).await.unwrap();
        }

        "deny" => {
            state.database.execute("
                delete AuthToken filter .account.id = <uuid><str>$1;
                delete MigrationRequest filter .id = <uuid><str>$0;
                delete Account filter .id = <uuid><str>$1;
                delete Discord filter .id = <uuid><str>$2
            ", &(
                &body.migrationid,
                request[0]["account"]["id"].as_str().unwrap(),
                request[0]["discord"]["id"].as_str().unwrap()
            )).await.unwrap();
        }

        _ => {}
    }

    Redirect::to("/mod/users")
}

async fn mod_levels(State(state): State<AppState>, jar: CookieJar) -> Result<Html<String>, Redirect> {
    let mut ctx = Context::new();

    match jar.get("token") {
        Some (ref cookie) => {
            let token = state.database.get_info_from_token(cookie.value()).await;

            let levels = state.database.query_json("
                select Level {
                    id, name, placement, video_id, level_id, creator, verifier: { name }
                } order by .placement
            ", &()).await.unwrap().parse::<Value>().unwrap();

            ctx.insert("levels", &levels.as_array().unwrap());

            let players = state.database.query_json("
                select Player {
                    id, name
                } order by .name
            ", &()).await.unwrap().parse::<Value>().unwrap();

            ctx.insert("players", &players.as_array().unwrap());

            if !token[0]["account"].is_null() && token[0]["account"]["mod"].as_bool().unwrap() {
                ctx.insert("account", &token[0]["account"]);

                return Ok(state.template.render("modlevels.html", &ctx).unwrap().into());
            }
        }

        None => {}
    }

    Err(Redirect::to("/account"))
}

#[derive(Deserialize)]
struct LevelData {
    creator: String,
    id: String,
    levelid: String,
    name: String,
    placement: u64,
    verifiername: String,
    videoid: String,
    method: String
}

async fn edit_level(State(state): State<AppState>, jar: CookieJar, Form(body): Form<LevelData>) -> Redirect {
    let token: &str;

    match jar.get("token") {
        Some (ref cookie) => {
            token = cookie.value();
        }

        None => {
            return Redirect::to("/account");
        }
    }

    let info = state.database.query_json("
        select AuthToken {
            account: { id, mod }
        } filter .token = <str>$0 and .expires > <datetime>datetime_of_statement()
    ", &(token,)).await.unwrap().parse::<Value>().unwrap();

    if info[0]["account"]["id"].is_null() {
        return Redirect::to("/account");
    }

    if info[0]["account"]["mod"].as_bool() == Some(false) {
        return Redirect::to("/");
    }

    state.database.execute("
        insert Player { name := <str>$0 } unless conflict on .name;
    ", &(body.verifiername.clone(),)).await.unwrap();

    match body.method.clone().as_str() {
        "editlevel" => {
            let level = state.database.query_json("
                select Level { placement } filter .id = <uuid><str>$0
            ", &(&body.id,)).await.unwrap().parse::<Value>().unwrap();

            state.database.execute("
                update Level filter .id = <uuid><str>$0 set {
                    creator := <str>$1,
                    level_id := <int32><str>$2,
                    name := <str>$3,
                    verifier := (select Player filter .name = <str>$4),
                    video_id := <str>$5
                }
            ", &(
                &body.id,
                &body.creator,
                &body.levelid,
                &body.name,
                &body.verifiername,
                &body.videoid
            )).await.unwrap();

            if level[0]["placement"].as_u64().unwrap() > body.placement {
                state.database.execute("
                    update Level filter .placement >= <int32><str>$0 set { placement := .placement + 1 };
                    update Level filter .placement = <int32><str>$1 + 1 set { placement := <int32><str>$0 };
                    update Level filter .placement > <int32><str>$1 set { placement := .placement - 1 };
                ", &(
                    body.placement.to_string(),
                    level[0]["placement"].as_u64().unwrap().to_string()
                )).await.unwrap();
            }

            if level[0]["placement"].as_u64().unwrap() < body.placement {
                state.database.execute("
                    update Level filter .placement > <int32><str>$0 set { placement := .placement + 1 };
                    update Level filter .placement = <int32><str>$1 set { placement := <int32><str>$0 + 1 };
                    update Level filter .placement >= <int32><str>$1 set { placement := .placement - 1 };
                ", &(
                    body.placement.to_string(),
                    level[0]["placement"].as_u64().unwrap().to_string()
                )).await.unwrap();
            }
        }

        "addlevel" => {
            state.database.execute("
                update Level filter .placement >= <int32><str>$0 set { placement := .placement + 1 };
            ", &(body.placement.to_string(),)).await.unwrap();

            state.database.execute("
                insert Level {
                    creator := <str>$0,
                    level_id := <int32><str>$1,
                    name := <str>$2,
                    placement := <int32><str>$3,
                    verifier := (select Player filter .name = <str>$4),
                    video_id := <str>$5
                }
            ", &(
                &body.creator,
                &body.levelid,
                &body.name,
                body.placement.to_string(),
                &body.verifiername,
                &body.videoid
            )).await.unwrap();
        }

        _ => {}
    } 

    let records: i64 = state.database.query_required_single_json("
        select count((select Entry filter .status = Status.Approved))
    ", &()).await.unwrap().parse::<Value>().unwrap().as_i64().unwrap();

    if records != RECORDS.read().unwrap().clone() {
        let threadstate = state.clone();
        thread::spawn(move || {
            let players = threadstate.database.query_json("select Player {
                id,
                rank,
                points
            } filter .points > 0 order by .points desc", &());

            let new_players = block_on(players).unwrap().parse::<Value>().unwrap();

            let mut guard = LEADERBOARD_CACHE.write().unwrap();
            *guard = new_players.clone();
            drop(guard);

            let mut guard = RECORDS.write().unwrap();
            *guard = records.clone();
            drop(guard);
        });
    }

    Redirect::to("/mod/levels")
}

async fn settings(State(state): State<AppState>, jar: CookieJar) -> Result<Html<String>, Redirect> {
    let mut ctx = Context::new();

    match jar.get("token") {
        Some (ref cookie) => {
            let token = state.database.get_info_from_token(cookie.value()).await;

            if token[0]["account"].is_null() {
                return Err(Redirect::to("/account"));
            }

            ctx.insert("account", &token[0]["account"]);

            if token[0]["account"]["status"].as_str() != Some("Done") {
                return Err(Redirect::to("/account"));
            }
        }

        None => {}
    }

    Ok(state.template.render("settings.html", &ctx).unwrap().into())
}

#[derive(Deserialize)]
struct AccountSettings {
    method: String,
    name: Option<String>,
    device: Option<String>,
    profileshape: Option<String>
}

async fn account_settings(State(state): State<AppState>, jar: CookieJar, Form(body): Form<AccountSettings>) -> Redirect {
    let token: &str;

    match jar.get("token") {
        Some (ref cookie) => {
            token = cookie.value()
        }

        None => {
            return Redirect::to("/account");
        }
    }

    match body.method.clone().as_str() {
        "update" => {
            let info = state.database.query_json("
                select AuthToken { id, account: { id, player } } filter .token = <str>$0
            ", &(token,)).await.unwrap().parse::<Value>().unwrap();

            if info[0]["id"].is_null() {
                return Redirect::to("/account");
            }

            let mut device = body.device.clone().unwrap();
            let mut profileshape = body.profileshape.clone().unwrap();

            if body.name.clone().unwrap().trim().len() > 25 {
                return Redirect::to("/account/settings");
            }

            state.database.execute("
                update Player filter .id = <uuid><str>$0 set {
                    name := <str>$1,
                    device := <Device><str>$2
                };
                update Account filter .id = <uuid><str>$3 set {
                    profile_shape := <ProfileShape><str>$4
                }
            ", &(
                info[0]["account"]["player"]["id"].as_str().unwrap(),
                body.name.clone().unwrap().trim(),
                format!("{}{}", device.remove(0).to_uppercase(), &device),
                info[0]["account"]["id"].as_str().unwrap(),
                format!("{}{}", profileshape.remove(0).to_uppercase(), &profileshape)
            )).await.unwrap();
        }

        "logout" => {
            state.database.execute("
                delete AuthToken filter .token = <str>$0
            ", &(token,)).await.unwrap();

            return Redirect::to("/");
        }

        "delete" => {
            let info = state.database.query_json("
                select AuthToken { id, account: { id, player } } filter .token = <str>$0
            ", &(token,)).await.unwrap().parse::<Value>().unwrap();

            if info[0]["id"].is_null() {
                return Redirect::to("/");
            }

            state.database.execute("
                delete AuthToken filter .account.id = <uuid><str>$0;
                delete MigrationRequest filter .account.id = <uuid><str>$0;
                delete Account filter .id = <uuid><str>$0
            ", &(info[0]["account"]["id"].as_str().unwrap(),)).await.unwrap();

            return Redirect::to("/");
        }

        _ => {}
    }

    Redirect::to("/account/settings")
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
        .route("/submit", post(submit_record))
        .route("/account", get(account))
        .route("/account", post(update_account))
        .route("/account/settings", get(settings))
        .route("/account/settings", post(account_settings))
        .route("/account/setup", get(setup))
        .route("/account/setup", post(setup_account))
        .route("/account/migrate", get(migrate))
        .route("/account/migrate", post(migrate_account))
        .route("/mod", get(modpage))
        .route("/mod/records", get(mod_records))
        .route("/mod/records", post(edit_record))
        .route("/mod/users", get(mod_users))
        .route("/mod/users", post(update_user))
        .route("/mod/levels", get(mod_levels))
        .route("/mod/levels", post(edit_level))
        .route("/terms", get(terms))
        .route("/privacy", get(privacy))
        .route("/rules", get(rules))
        .route("/oauth", get(oauth))

        .route_service("/favicon.ico", ServeFile::new("site/meta/favicon.ico"))
        .route_service("/robots.txt", ServeFile::new("site/robots.txt"))

        .nest_service("/src", ServeDir::new("site/src"))
        .nest_service("/meta", ServeDir::new("site/meta"))

        .fallback(|State(state): State<AppState>| async move {    
            let mut ctx = Context::new();
            ctx.insert("status", "404: Not Found");
            
            (StatusCode::NOT_FOUND, Html::<String>::from(state.template.render("fallback.html", &ctx).unwrap()))
        })

        .with_state(state);

    // Set up 

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8111").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}