#![allow(unused)]

mod config;
mod selection;
mod parse;

use std::fs::read;
use std::mem::take;
use anyhow::{anyhow, bail, Result as AnyResult};
use clap::Parser;
use env::EnvFile;
use sqlx::{Connection, PgConnection};
use regex::regex;
use tracing::Instrument;
use parse::{Parse, ParseError};
use selection::Selection;
use crate::parse::{Punctuated, SelectionArgs, Slash};
use crate::selection::{ParseSelection, Selector};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, short)]
    interactive: bool,

    #[clap(long, short)]
    force: bool,

    #[clap(long, short)]
    /// Source database URL
    source_url: Option<String>,

    #[clap(long, short)]
    /// Dest database URL
    dest_url: Option<String>,

    args: Vec<String>,
}

fn get_source_url(arg: Option<String>) -> Option<String> {
    if arg.is_some() {
        return arg;
    }
    if let Some(env) = EnvFile::read(".env.production").ok() {
        return env.lookup("DATABASE_URL").map(|s| s.to_string());
    }
    if let Some(env) = EnvFile::read("../.env.production").ok() {
        return env.lookup("DATABASE_URL").map(|s| s.to_string());
    }
    None
}

fn get_dest_url(arg: Option<String>) -> Option<String> {
    if arg.is_some() {
        return arg;
    }
    if let Some(env) = EnvFile::read(".env").ok() {
        return env.lookup("DATABASE_URL").map(|s| s.to_string());
    }
    if let Some(env) = EnvFile::read("../.env").ok() {
        return env.lookup("DATABASE_URL").map(|s| s.to_string());
    }
    None
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let mut args = Args::parse();
    let source_url = get_source_url(args.source_url).ok_or(anyhow!("No source url provided or found in .env.production"))?;
    let dest_url = get_dest_url(args.dest_url).ok_or(anyhow!("No dest url provided or found in .env"))?;
    let mut from_conn = PgConnection::connect(&source_url).await?;
    let mut to_conn = PgConnection::connect(&dest_url).await?;

    if args.args.is_empty() {
        bail!("No tables selected for seeding");
    }
    let args = SelectionArgs::new(args.args);
    let selections = Punctuated::<Slash, ParseSelection>::parse(&mut args.token_stream())?;
    let selections: Vec<Selection> = selections.into_vec().into_iter().map(Into::into).collect();
    dbg!(selections);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_selection() -> AnyResult<()> {
    //     let s = "org 123 . deduction rand 100, sort created_at desc, created_at gt 'now() - interval 1 day'";
    //     let s = shlex::split(s).unwrap();
    //     let result = parse_args(s)?;
    //     dbg!(&result);
    //     assert_eq!(result, vec![Selection {
    //         table: "org".to_string(),
    //         selectors: vec![Selector::Id("123".to_string())],
    //     }, Selection {
    //         table: "deduction".to_string(),
    //         selectors: vec![
    //             Selector::ForeignKey(0),
    //         ],
    //     }]);
    //     Ok(())
    // }
}

