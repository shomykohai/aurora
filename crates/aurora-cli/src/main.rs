mod cmds;
mod config;
mod macros;
mod style;

use anyhow::{Result, anyhow};
use aurora::Device;
use clap::{CommandFactory, Parser};

use crate::cmds::Commands;
use crate::config::AuroraConfig;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    #[arg(
        short,
        long,
        global = true,
        value_name = "ID",
        help = "Specify the board to interact with"
    )]
    pub board: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

fn main() {
    if let Err(e) = run() {
        let error = style::ERROR;
        anstream::eprintln!("{error}aurora:{error:#} {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = CliArgs::parse();
    let config = AuroraConfig::load()?;

    let Some(command) = args.command else {
        CliArgs::command().print_help()?;
        return Ok(());
    };

    let mut dev = match args.board.as_deref().or(config.default_board.as_deref()) {
        Some(id) => {
            let board = aurora::boards::by_id(id).ok_or_else(|| anyhow!("unknown board {id}"))?;
            Device::find_board(board)?
        }
        None => Device::find()?,
    };

    command.execute(&mut dev, &config)
}
