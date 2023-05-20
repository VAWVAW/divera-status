use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;

use clap::ArgGroup;
use clap::Parser;
use derive_getters::Getters;

#[derive(Parser, Getters)]
#[command(author, version, about, long_about = None)]
#[command(group(ArgGroup::new("token_input").required(true).args(["token", "token_file"])))]
pub struct Arguments {
    /// your personal api token for the divera instance
    #[arg(short, long)]
    pub(crate) token: Option<String>,

    /// file with the api token as first line
    #[arg(short = 'f', long)]
    pub(crate) token_file: Option<PathBuf>,

    /// update interval in seconds
    #[arg(short, long, default_value_t = 30)]
    pub(crate) interval: u8,

    /// divera instance to use
    #[arg(long, default_value = "https://app.divera247.com")]
    pub(crate) server: String,


    /// debug output
    #[arg(short, long)]
    pub(crate) debug: bool,
}

impl Arguments {
    pub fn get_token(&self) -> Result<String, io::Error> {
        let token: String = if let Some(token) = &self.token {
            token.clone()
        } else {
            let mut buffer = String::new();
            let file = File::open(
                self.token_file
                    .as_ref()
                    .expect("neither token or token-file provided"),
            )?;
            let _ = io::BufReader::new(file).read_line(&mut buffer)?;
            buffer.pop();
            buffer
        };
        Ok(token)
    }
}

#[non_exhaustive]
#[allow(unused)]
#[derive(Debug, PartialEq)]
pub enum Update {
    Reload,
    StatusNext,
    StatusPrev,
}
