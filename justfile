
# Run all checks
gate: lint format test audit

lint:
	cargo clippy --all-features {{ if env_var_or_default("CI", "false") == "true" { "" } else { "--fix --allow-dirty" } }}

format:
	cargo fmt --all {{ if env_var_or_default("CI", "false") == "true" { "-- --check" } else { "" } }}  

# If your environment is connected to AWS `test` will run integration tests as well.
test:
	cargo test {{ if has_aws == "true" {"--all-features"} else {""} }} -- --test-threads=1

code-coverage $CARGO_INCREMENTAL="{{cargo_incremental}}":
	LLVM_PROFILE_FILE=tmp-%p-%m.profraw RUSTFLAGS=-Cinstrument-coverage just test
	grcov . -s . --binary-path ./target/debug/ -t lcov --branch --ignore '../*' --ignore "/*" --ignore-not-existing -o ./target/cov.lcov
	rm -f *.profraw

audit:
	cargo audit

# Deploy the extension to AWS. The deployment will be to the default region, x86_64-unknown-linux-gnu profile and compiled in debug mode. profile types - x86_64-unknown-linux-gnu || aarch64-unknown-linux-gnu
deploy-debug-extension region="us-east-1" architecture="x86_64-unknown-linux-gnu":
	@rm -rf ./target/lambda
	just build-extension debug {{architecture}}
	just deploy-extension debug {{region}} {{architecture}}

# Deploy the extension to AWS. The deployment will be to the default region, x86_64-unknown-linux-gnu architecture and compiled in release mode. Architecture types - x86_64-unknown-linux-gnu || aarch64-unknown-linux-gnu
deploy-release-extension region="us-east-1" architecture="x86_64-unknown-linux-gnu":
	@rm -rf ./target/lambda
	just build-extension release {{architecture}}
	just deploy-extension  release {{region}} {{architecture}}

build-extension target architecture:
	@echo 'Building extension {{target}} for {{architecture}}'
	cargo lambda build --extension {{ if target == "release" { "--release" } else { "" } }} --target {{ architecture }}

deploy-extension target region architecture:
	@echo 'Deploying {{target}} for {{architecture}} in region {{region}}'
	@command -v aws &> /dev/null || { echo "aws cli not found"; exit 1; }
	
	zip -j ./target/lambda/extensions/chaos-lambda-extension.zip ./misc/bootstrap && \
	cd ./target/lambda/ && \
	zip -ur ./extensions/chaos-lambda-extension.zip ./extensions/chaos-lambda-extension
	
	AWS_PAGER="" aws lambda publish-layer-version --region {{region}} --layer-name chaos-lambda-extension-{{architecture}}-{{target}} \
	--zip-file fileb://./target/lambda/extensions/chaos-lambda-extension.zip \
	--description "Add some chaos to you Lambda" \
	--compatible-runtimes python3.10 python3.9 python3.8 nodejs18.x nodejs16.x nodejs14.x java17 java8.al2 java11 dotnet6 ruby3.2 ruby2.7 provided.al2 \
	--compatible-architectures {{ if architecture == "aarch64-unknown-linux-gnu" { "arm64" } else { "x86_64" } }} \
	--license-info https://github.com/aws-cli-tools/chaos-lambda-extension/blob/main/LICENSE


# The variable interpolation for path_exists might not work as expected in just
credentials := env_var("HOME") + "/.aws/credentials"
cargo_incremental := if env_var_or_default("CI", "false") == "true" { "0" } else { "1" }
has_aws := if path_exists(credentials) == "true" {
  "true"
} else if env_var_or_default("AWS_ACCESS_KEY_ID", "") != "" {
  "true"
} else {
  "false"
}