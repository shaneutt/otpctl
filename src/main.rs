use std::process::exit;

use log::debug;
use otplib::Authenticator;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use url::Url;

// -----------------------------------------------------------------------------
// Main
// -----------------------------------------------------------------------------

fn main() {
    let args = Cli::from_args();
    let config = config_setup(&args);
    let codes = get_codes(config.tokens);
    for (issuer, digits, code) in codes {
        println!("{} => {:0width$}", issuer, code, width = digits as usize);
    }
}

// -----------------------------------------------------------------------------
// Consts
// -----------------------------------------------------------------------------

const ERR_NO_TOKENS: i32 = 25;
const ERR_INVALID_CONFIG: i32 = 26;
const ERR_YAML: i32 = 27;

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    tokens: Option<Vec<String>>,
}

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

// -----------------------------------------------------------------------------
// Private Functions
// -----------------------------------------------------------------------------

fn config_setup(args: &Cli) -> Config {
    let config_raw = match std::fs::read_to_string(&args.path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("{:?}", err);
            exit(err.raw_os_error().unwrap());
        }
    };
    match serde_yaml::from_str(&config_raw) {
        Ok(c) => c,
        Err(err) => {
            debug!("{}", err);

            if format!("{}", err).contains("EOF") {
                eprintln!("invalid configuration file");
                exit(ERR_INVALID_CONFIG);
            }

            eprintln!("YAML: couldn't parse {:?}", &args.path);
            exit(ERR_YAML);
        }
    }

    // TODO - fail on bad config permissions
}

fn get_codes(tokens_opt: Option<Vec<String>>) -> Vec<(String, u32, u32)> {
    let mut codes: Vec<(String, u32, u32)> = Vec::new();
    for token_url_str in parse_tokens(tokens_opt) {
        let token_url = Url::parse(&token_url_str).unwrap();
        let mut digits: u32 = 6;
        let mut issuer: String = "unknown".to_string();
        for (k, v) in token_url.query_pairs() {
            if k == "issuer" {
                issuer = v.into_owned();
            } else if k == "digits" {
                let digits_str: String = v.into_owned();
                digits = digits_str.parse::<u32>().unwrap();
            }
        }
        let auth = Authenticator::from_token_url(&token_url_str).unwrap();
        codes.push((issuer, digits, auth.generate_totp()));
    }

    return codes;
}

fn parse_tokens(tokens: Option<Vec<String>>) -> Vec<String> {
    match tokens {
        Some(t) => t,
        None => {
            eprintln!("no tokens provided in config");
            exit(ERR_NO_TOKENS);
        }
    }
}
