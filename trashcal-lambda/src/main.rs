use ::trashcal::trashcal_handler;
use lambda_http::{
    lambda_runtime::diagnostic::Diagnostic, run, service_fn, tower::ServiceExt, Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::tracing::init_default_subscriber();
    run(service_fn(trashcal_handler).map_err(std::convert::Into::<Diagnostic>::into)).await
}
