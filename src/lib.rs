use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_sts::error::SdkError;
use aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityError;
use aws_sdk_sts::{config::Region, Client};
use aws_types::SdkConfig;
use clap::ValueEnum;
use log::info;
use mockall::*;
use serde_json::json;
use std::fmt::Debug;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputType {
    /// Output as json
    Json,
    /// Output as regular string
    String,
}

#[automock]
#[async_trait]
pub trait GetCallerIdentity {
    async fn get_caller_identity(
        &self,
    ) -> Result<
        aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityOutput,
        SdkError<GetCallerIdentityError>,
    >;
}
pub struct StsClient {
    client: Client,
}

impl StsClient {
    pub fn new(sdk_config: &::aws_types::sdk_config::SdkConfig) -> Self {
        Self {
            client: Client::new(sdk_config),
        }
    }
}

#[async_trait]
impl GetCallerIdentity for StsClient {
    async fn get_caller_identity(
        &self,
    ) -> Result<
        aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityOutput,
        SdkError<GetCallerIdentityError>,
    > {
        self.client.get_caller_identity().send().await
    }
}
pub fn get_region_provider(region: Option<String>) -> RegionProviderChain {
    info!("Getting region details");

    RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"))
}

pub async fn get_aws_config(
    profile: Option<String>,
    region_provider: RegionProviderChain,
) -> SdkConfig {
    if let Some(p) = profile {
        info!("Using profile - {}", p);
        aws_config::from_env()
            .region(region_provider)
            .profile_name(p)
            .load()
            .await
    } else {
        info!("Using default profile");
        aws_config::from_env().region(region_provider).load().await
    }
}
pub async fn get_caller_identity(
    client: &impl GetCallerIdentity,
    output_type: OutputType,
    mut writer: impl std::io::Write,
) -> Result<()> {
    info!("Calling 'get_caller_identity'");
    let response = client
        .get_caller_identity()
        .await
        .with_context(|| "Failed loading AWS config details. Did you run 'aws configure' ?")?;

    info!("Successful call");
    let account_id = response.account().unwrap_or_default();
    let user_arn = response.arn().unwrap_or_default();

    info!("Output type is {:?}", output_type);
    match output_type {
        OutputType::String => {
            writeln!(writer, "AccountId = {}", account_id)?;
            writeln!(writer, "UserARN = {}", user_arn)?;
        }
        OutputType::Json => {
            let result = json!({
                "accountId": account_id,
                "UserARN": user_arn,
            });
            writeln!(writer, "{}", result)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use aws_sdk_sts::operation::get_caller_identity::GetCallerIdentityOutput;
    use rstest::rstest;

    #[rstest]
    #[case(OutputType::String, "AccountId = account\nUserARN = arn\n")]
    #[case(OutputType::Json, "{\"UserARN\":\"arn\",\"accountId\":\"account\"}\n")]
    #[tokio::test]
    pub async fn get_caller_identity_test(#[case] input: OutputType, #[case] expected: String) {
        let mut result = Vec::new();
        let mut mock = MockGetCallerIdentity::new();
        let response = GetCallerIdentityOutput::builder()
            .account("account")
            .arn("arn")
            .build();
        mock.expect_get_caller_identity()
            .returning(move || Ok(response.clone()));

        let _ = get_caller_identity(&mock, input, &mut result).await;

        let result_str = String::from_utf8(result).unwrap();

        assert_eq!(&result_str, &expected.to_string());
    }
}
