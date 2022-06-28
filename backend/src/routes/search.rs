use rocket::serde::json::Json;

use crate::{
    app::Application,
    core::auth::AuthenticatedAccount,
    util::{madness::Madness, types::Character},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct SearchResponse {
    query: String,
    results: Vec<Character>,
}

#[get("/api/search?<query>")]
async fn query(
    query: String,
    account: AuthenticatedAccount,
    app: &rocket::State<Application>,
) -> Result<Json<SearchResponse>, Madness> {
    account.require_access("search")?;

    let search_like = format!("%{}%", query);
    let results = sqlx::query!(
        "SELECT id, name, COALESCE(\"alt_character\".account_id, id) AS account_id
        FROM \"character\" 
        LEFT JOIN \"alt_character\" ON id = \"alt_character\".alt_id 
        WHERE name ILIKE $1 LIMIT 25",
        search_like
    )
    .fetch_all(app.get_db())
    .await?
    .into_iter()
    .map(|r| Character {
        id: r.id,
        name: r.name,
        account_id: r.account_id,
    })
    .collect();

    Ok(Json(SearchResponse { query, results }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![query]
}
