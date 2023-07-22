
# Run all checks
gate: lint format test audit

lint:
	cargo clippy --all-features {{ if env_var_or_default("CI", "false") == "true" { "" } else { "--fix --allow-dirty" } }}

format:
	cargo fmt --all {{ if env_var_or_default("CI", "false") == "true" { "-- --check" } else { "" } }}  

# If your environment is connected to AWS `test` will run integration tests as well.
test:
	cargo test {{ if has_aws == "true" {"--all-features"} else {""} }}

code-coverage $CARGO_INCREMENTAL="{{cargo_incremental}}":
	LLVM_PROFILE_FILE=tmp-%p-%m.profraw RUSTFLAGS=-Cinstrument-coverage just test
	grcov . -s . --binary-path ./target/debug/ -t lcov --branch --ignore '../*' --ignore "/*" --ignore-not-existing -o ./target/cov.lcov
	rm -f *.profraw

audit:
	cargo audit

deploy-debug-extension:
	just _deploy-extension debug
deploy-release-extension:
	just _deploy-extension release

_deploy-extension target:
	@echo 'Building {{target}} for x86_64'
	@command -v aws &> /dev/null || { echo "aws cli not found"; exit 1; }
	@rm -rf ./target/lambda
	cargo lambda build --extension {{ if target == "release" { "--release" } else { "" } }}
	zip -j ./target/lambda/extensions/chaos-lambda-extension.zip ./target/lambda/extensions/chaos-lambda-extension ./misc/bootstrap
	AWS_PAGER="" aws lambda publish-layer-version --layer-name chaos-lambda-extension-x86_64-{{target}} --zip-file fileb://./target/lambda/extensions/chaos-lambda-extension.zip --description "Add some chaos to you Lambda" --compatible-runtimes python3.10 nodejs18.x java17 dotnet6 go1.x ruby3.2 provided.al2 --compatible-architectures x86_64 --license-info https://github.com/aws-cli-tools/chaos-lambda-extension/blob/main/LICENSE

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