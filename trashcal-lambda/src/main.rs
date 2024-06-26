use ::trashcal::trashcal_handler;
use lambda_http::{run, service_fn, tower::ServiceExt, Error};
use trashcal::diagnostic_wrapper::DiagnosticWrapper;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::tracing::init_default_subscriber();
    run(service_fn(trashcal_handler).map_err(DiagnosticWrapper::new)).await
}
