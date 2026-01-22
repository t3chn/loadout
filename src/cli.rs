use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "loadout",
    version,
    about = "Machine-first skill manager for Codex CLI + Claude Code"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Target {
    Codex,
    Claude,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init {
        #[arg(long)]
        primary_url: String,

        #[arg(long, default_value = "main")]
        primary_ref: String,

        #[arg(long, default_value_t = true)]
        trust_primary: bool,

        #[arg(long, default_value_t = true)]
        writable_primary: bool,
    },

    Catalog {
        #[arg(long)]
        target: Target,
    },

    Suggest {
        #[arg(long)]
        target: Target,

        #[arg(long)]
        query: String,

        #[arg(long, default_value_t = 10)]
        limit: usize,
    },

    Add {
        #[arg(long)]
        target: Target,

        ids: Vec<String>,
    },

    Set {
        #[arg(long)]
        target: Target,

        ids: Vec<String>,
    },

    Remove {
        #[arg(long)]
        target: Target,

        ids: Vec<String>,
    },

    Sync {
        #[arg(long)]
        target: Target,
    },

    Status {
        #[arg(long)]
        target: Target,
    },

    Doctor,

    Source {
        #[command(subcommand)]
        command: SourceCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum SourceCommand {
    Add {
        name: String,

        #[arg(long)]
        url: String,

        #[arg(long = "ref", default_value = "main")]
        ref_name: String,
    },
    Trust {
        name: String,

        #[arg(long)]
        yes: bool,
    },
    Pin {
        name: String,

        #[arg(long)]
        to: String,
    },
    Status {
        name: String,
    },
}
