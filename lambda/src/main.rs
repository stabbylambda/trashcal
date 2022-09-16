use std::convert::TryFrom;

use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use http::StatusCode;
use icalendar::Calendar;
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use tracing::info;
use tracing::{span, Level};
use trashcal::trashcal;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // get the ID
    let params = event.path_parameters();
    let query = event.query_string_parameters();
    let id = params.first("id").or(query.first("id")).unwrap_or("null");

    // start tracing
    let _x = span!(Level::INFO, "trashcal", id).entered();

    let calendar = trashcal(&id).await?;
    let accept = &event.headers()[http::header::ACCEPT];

    // build the response as either json or calendar
    let resp = Response::builder().status(StatusCode::OK);
    let resp = if accept.to_str()?.starts_with("application/json") {
        info!("Returning calendar as json");
        let json = serde_json::to_string_pretty(&calendar)?;

        resp.header(CONTENT_TYPE, "application/json")
            .body(json.into())
    } else {
        info!("Returning calendar as iCal feed");
        let calendar: Calendar = Calendar::try_from(calendar)?;

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
