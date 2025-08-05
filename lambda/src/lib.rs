use anyhow::Result;
use http::{HeaderMap, HeaderValue};

use crate::trashcal::trashcal;
use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE, EXPIRES};
use http::StatusCode;
use lambda_http::{Body, Request, RequestExt, Response};
use lambda_runtime::tracing::info;

use tracing::instrument;

pub mod error;
pub mod pickup;
pub mod pickup_calendar;
pub mod trashcal;

#[instrument]
pub async fn get_trashcal(id: &str, accept: &str, whimsy: bool) -> Result<Response<Body>> {
    let is_ics_request = id.contains(".ics");

    let calendar = trashcal(id).await?;

    // build the response as either json or calendar
    let resp = Response::builder().status(StatusCode::OK);
    let resp = if accept.starts_with("application/json") && !is_ics_request {
        info!(
            message = "Returning calendar as JSON", 
            address = %calendar.address,
            pickup_dates = ?calendar.pickups.iter().map(|p| p.date).collect::<Vec<_>>()
        );
        let json = serde_json::to_string_pretty(&calendar)?;

        resp.header(CONTENT_TYPE, "application/json")
            .header(EXPIRES, calendar.expires_header())
            .body(json.into())
    } else {
        info!(
            message = "Returning calendar as iCal",
            address = %calendar.address,
            pickup_dates = ?calendar.pickups.iter().map(|p| p.date).collect::<Vec<_>>()
        );
        let expires = calendar.expires_header();
        let calendar = calendar.to_calendar(whimsy)?;

        resp.header(CONTENT_TYPE, "text/calendar;charset=UTF-8")
            .header(CONTENT_DISPOSITION, "attachment; filename=trashcal.ics")
            .header(EXPIRES, expires)
            .body(calendar.to_string().into())
    };
    Ok(resp?)
}

pub async fn trashcal_handler(event: Request) -> Result<Response<Body>> {
    // get the ID
    let params = event.path_parameters();
    let query = event.query_string_parameters();
    let id = params
        .first("id")
        .or_else(|| query.first("id"))
        .unwrap_or("null");
    let accept = get_mime_type(event.headers());
    
    // get the whimsy parameter, defaults to true
    let whimsy = query
        .first("whimsy")
        .map(|v| v != "false")
        .unwrap_or(true);

    get_trashcal(id, accept, whimsy).await
}

/// Safely get the accept header. Fastmail apparently doesn't send an accept header at all (!)
pub fn get_mime_type(headers: &HeaderMap<HeaderValue>) -> &str {
    headers
        .get(http::header::ACCEPT)
        .and_then(|x| x.to_str().ok())
        .unwrap_or("text/calendar")
}

#[cfg(test)]
mod test {
    use http::HeaderMap;
    use std::sync::Once;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    use crate::get_mime_type;

    static INIT: Once = Once::new();

    fn init_tracing() {
        INIT.call_once(|| {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "trashcal=debug,tower_http=debug".into()),
                )
                .with(tracing_subscriber::fmt::layer())
                .init();
        });
    }

    #[test]
    fn test_missing_accept_header() {
        init_tracing();
        let headers = HeaderMap::new();
        let result = get_mime_type(&headers);
        assert_eq!(result, "text/calendar");
    }

    #[test]
    fn test_json() {
        init_tracing();
        let mut headers = HeaderMap::new();
        headers.insert(http::header::ACCEPT, "application/json".parse().unwrap());
        let result = get_mime_type(&headers);
        assert_eq!(result, "application/json");
    }

    #[test]
    fn test_calendar() {
        init_tracing();
        let mut headers = HeaderMap::new();
        headers.insert(http::header::ACCEPT, "text/calendar".parse().unwrap());
        let result = get_mime_type(&headers);
        assert_eq!(result, "text/calendar");
    }

    #[test]
    fn test_weird_header_value() {
        init_tracing();
        let mut headers = HeaderMap::new();
        headers.insert(http::header::ACCEPT, "💩".parse().unwrap());
        let result = get_mime_type(&headers);
        assert_eq!(result, "text/calendar");
    }
}
