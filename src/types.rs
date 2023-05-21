use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;

use clap::{ArgGroup, Parser};
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

    /// statuses to be displayed (ids, comma separated)
    #[arg(short, long)]
    pub(crate) shown_statuses: String,

    /// order of statuses for quick change (ids, comma separated)
    #[arg(short = 'o', long)]
    pub(crate) status_order: String,

    /// format for updates to stdout, possible {}-values are: full_text, short_text, status_name, status_color, \[status_id], \[status_id]_count, \[status_id]_color
    #[arg(
        short,
        long,
        default_value = "{{\"full_text\": \"{full_text} <span color=\"#{status_color}\">@</span>\", \"short_text\": \"{short_text}\"}}"
    )]
    pub(crate) display_format: String,

    /// escape double quotes in {full_text} and {short_text}
    #[arg(short, long)]
    pub(crate) escape_quotes: bool,

    /// disable pango markup with the status_color in {full_text} and {short_text}
    #[arg(long)]
    pub(crate) no_pango: bool,

    /// debug output
    #[arg(long)]
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
