use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use tracing::{span, Level};
use trashcal::trashcal;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let params = event.path_parameters();
    let id = params.first("id").unwrap_or("null");
    let _x = span!(Level::INFO, "trashcal", id).entered();
    let calendar = trashcal(&id).await?;
    _x.exit();

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/calendar;charset=UTF-8")
        .header("content-disposition", "attachment; filename=trashcal.ics")
        .body(calendar.to_string().into())
        .map_err(Box::new)?;
    Ok(resp)
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
