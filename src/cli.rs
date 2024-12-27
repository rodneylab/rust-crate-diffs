use std::path::PathBuf;

use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Parser)]
#[clap(author,version,about,long_about=None)]
pub struct Cli {
    /// verbosity
    #[clap(flatten)]
    pub verbose: Verbosity,

    /// repo path
    pub repo_path: PathBuf,
}

#[cfg(test)]
mod tests {
    #[test]
    fn cli_tests() {
        trycmd::TestCases::new().case("tests/cmd/*.toml");
    }
}
