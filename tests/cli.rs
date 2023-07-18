#[allow(unused_imports)]
mod cli_tests {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::process::Command;

    #[cfg(feature = "aws_configured")]
    #[test]
    fn happy_flow() {
        let mut cmd = Command::cargo_bin("whoamiaws").unwrap();

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("UserARN"));
    }
}
