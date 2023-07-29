use lambda_extension::{service_fn, Error, Extension, LambdaEvent};

use tokio::task;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

mod routes;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let state = routes::AppState {
        runtime_api_address: std::env::var("AWS_LAMBDA_RUNTIME_API")
            .expect("Missing AWS_LAMBDA_RUNTIME_API!"),
    };
    debug!(
        "Pulling AWS_LAMBDA_RUNTIME_API end point - {}",
        state.runtime_api_address
    );

    let app = routes::router(state);

    info!("Chaos extension is enabled");
    // run it
    let server =
        axum::Server::bind(&"0.0.0.0:9100".parse().unwrap()).serve(app.into_make_service());

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
