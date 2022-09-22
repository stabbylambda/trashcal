use std::convert::TryFrom;

use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use http::{HeaderMap, HeaderValue, StatusCode};
use icalendar::Calendar;
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use tracing::info;
use tracing::{span, Level};
use trashcal::trashcal;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // get the ID
    let params = event.path_parameters();
    let query = event.query_string_parameters();
    let id = params
        .first("id")
        .or_else(|| query.first("id"))
        .unwrap_or("null");

    // start tracing
    let _x = span!(Level::INFO, "trashcal", id).entered();

    let calendar = trashcal(id).await?;
    let accept = get_mime_type(event.headers());

    // build the response as either json or calendar
    let resp = Response::builder().status(StatusCode::OK);
    let resp = if accept.starts_with("application/json") {
        info!("Returning calendar as json");
        let json = serde_json::to_string_pretty(&calendar)?;

        resp.header(CONTENT_TYPE, "application/json")
            .body(json.into())
    } else {
        info!("Returning calendar as iCal feed");
        let calendar = Calendar::try_from(calendar)?;

        resp.header(CONTENT_TYPE, "text/calendar;charset=UTF-8")
            .header(CONTENT_DISPOSITION, "attachment; filename=trashcal.ics")
            .body(calendar.to_string().into())
    };

    Ok(resp.map_err(Box::new)?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

/// Safely get the accept header. Fastmail apparently doesn't send an accept header at all (!)
fn get_mime_type(headers: &HeaderMap<HeaderValue>) -> &str {
    let inner = || -> Result<&str, Box<dyn std::error::Error>> {
        let value = headers
            .get(http::header::ACCEPT)
            .ok_or("no header present")?;

        let s = value.to_str()?;
        Ok(s)
    };

    inner().unwrap_or("text/calendar")
}

#[cfg(test)]
mod test {
    use http::HeaderMap;

    use crate::get_mime_type;

    #[test]
    fn test_missing_accept_header() {
        let headers = HeaderMap::new();
        let result = get_mime_type(&headers);
        assert_eq!(result, "text/calendar");
    }

    #[test]
    fn test_json() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::ACCEPT, "application/json".parse().unwrap());
        let result = get_mime_type(&headers);
        assert_eq!(result, "application/json");
    }

    #[test]
    fn test_calendar() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::ACCEPT, "text/calendar".parse().unwrap());
        let result = get_mime_type(&headers);
        assert_eq!(result, "text/calendar");
    }
}
