use std::{ffi::OsString, process::ExitCode};

#[tokio::main]
async fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if !requests_stdio(&arguments) {
        eprintln!("Usage: genko-ls --stdio");
        return ExitCode::from(2);
    }

    genko_ls::run_stdio().await;
    ExitCode::SUCCESS
}

fn requests_stdio(arguments: &[OsString]) -> bool {
    arguments.len() == 1 && arguments[0] == "--stdio"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_one_stdio_argument() {
        assert!(requests_stdio(&[OsString::from("--stdio")]));
        assert!(!requests_stdio(&[]));
        assert!(!requests_stdio(&[OsString::from("--tcp")]));
        assert!(!requests_stdio(&[
            OsString::from("--stdio"),
            OsString::from("extra")
        ]));
    }
}
