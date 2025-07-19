use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = "A bookmark manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    Add {
        key: String,
        url: String,
        #[arg(short, long)]
        desc: String,
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },
    Open {
        key: String,
    },
    Delete {
        key: String,
    },
}
