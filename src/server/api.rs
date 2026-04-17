use crate::db::Database;
use axum::{extract::{Query, State}, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct RangeParams {
    pub range: Option<String>,
}

pub fn routes(db: Database) -> Router {
    Router::new()
        .route("/api/username", get(get_username))
        .route("/api/totals", get(totals))
        .route("/api/timeline", get(timeline))
        .route("/api/app-distribution", get(app_distribution))
        .route("/api/daily-avg", get(daily_avg))
        .with_state(db)
}

async fn get_username() -> impl IntoResponse {
    let username = std::env::var("USERNAME")
        .unwrap_or_else(|_| "Pardon".to_string());
    Json(json!({ "username": username })).into_response()
}

fn parse_range(params: &RangeParams) -> &str {
    match params.range.as_deref() {
        Some("week") | Some("7d") => "7d",
        Some("month") | Some("30d") => "30d",
        Some("year") | Some("365d") => "365d",
        _ => "24h",
    }
}

fn range_to_hours(range: &str) -> u32 {
    match range {
        "7d" => 24 * 7,
        "30d" => 24 * 30,
        "365d" => 24 * 365,
        _ => 24,
    }
}

fn range_to_days(range: &str) -> u32 {
    match range {
        "7d" => 7,
        "30d" => 30,
        "365d" => 365,
        _ => 1,
    }
}

async fn totals(
    State(db): State<Database>,
    Query(params): Query<RangeParams>,
) -> impl IntoResponse {
    let range = parse_range(&params);
    let hours_back = range_to_hours(range);
    match db.query_totals_range(hours_back).await {
        Ok(totals) => Json(json!(totals)).into_response(),
        Err(e) => {
            tracing::error!("Error querying totals: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

async fn timeline(
    State(db): State<Database>,
    Query(params): Query<RangeParams>,
) -> impl IntoResponse {
    let range = parse_range(&params);
    let hours_back = range_to_hours(range);
    match db.query_timeline_range(hours_back).await {
        Ok(data) => Json(json!(data)).into_response(),
        Err(e) => {
            tracing::error!("Error querying timeline: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

async fn app_distribution(
    State(db): State<Database>,
    Query(params): Query<RangeParams>,
) -> impl IntoResponse {
    let range = parse_range(&params);
    let days_back = range_to_days(range);
    match db.query_app_distribution(days_back).await {
        Ok(data) => Json(json!(data)).into_response(),
        Err(e) => {
            tracing::error!("Error querying app distribution: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

async fn daily_avg(
    State(db): State<Database>,
    Query(params): Query<RangeParams>,
) -> impl IntoResponse {
    let range = parse_range(&params);
    let days_back = range_to_days(range);
    match db.query_daily_stats(days_back).await {
        Ok(data) => Json(json!(data)).into_response(),
        Err(e) => {
            tracing::error!("Error querying daily averages: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
