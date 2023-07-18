use anyhow::Result;
use clap::Parser;
use std::fmt::Debug;
use whoamiaws::{OutputType, StsClient};

#[derive(Debug, Parser)]
struct Opt {
    /// The AWS Region.
    #[clap(short, long)]
    region: Option<String>,

    /// Which profile to use.
    #[clap(short, long)]
    profile: Option<String>,

    #[arg(value_enum)]
    #[arg(default_value_t=OutputType::String)]
    #[clap(short, long)]
    output_type: OutputType,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Opt::parse();

    let region_provider = whoamiaws::get_region_provider(args.region);

    let shared_config = whoamiaws::get_aws_config(args.profile, region_provider).await;

    let client = StsClient::new(&shared_config);
    whoamiaws::get_caller_identity(&client, args.output_type, &mut std::io::stdout()).await?;

    Ok(())
}
