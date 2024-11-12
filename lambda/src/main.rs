use lambda_runtime::{
    run, service_fn,
    tracing::{self},
    Error,
};
mod handler;
mod image_task;
mod lock;
mod s3_sequencer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(handler::function_handler)).await
}
