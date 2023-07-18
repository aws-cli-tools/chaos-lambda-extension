
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