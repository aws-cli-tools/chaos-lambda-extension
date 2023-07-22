use axum::{
    routing::{get, post},
    Router,
};

use lambda_extension::{service_fn, Error, Extension, LambdaEvent};

use tokio::task;
use tracing::{debug, info};

mod routes;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let app = Router::new()
        .route(
            "/2018-06-01/runtime/invocation/next",
            get(routes::get_next_invocation),
        )
        .route(
            "/2018-06-01/runtime/invocation/:request_id/response",
            post(routes::post_invoke_response),
        )
        .route(
            "/2018-06-01/runtime/init/error",
            post(routes::post_initialization_error),
        )
        .route(
            "/2018-06-01/runtime/invocation/:request_id/error",
            post(routes::post_invoke_error),
        );

    debug!(
        "Pulling AWS_LAMBDA_RUNTIME_API end point - {}",
        *routes::AWS_LAMBDA_RUNTIME_API
    );
    // run it
    let server =
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(app.into_make_service());

    task::spawn(async move {
        server.await.unwrap();
    });

    Extension::new()
        .with_events(&[])
        .with_events_processor(service_fn(boot_extension))
        .run()
        .await
}

async fn boot_extension(event: LambdaEvent) -> Result<(), Error> {
    info!("Received the following Lambda event - {:?} ", event.next);
    Ok(())
}
