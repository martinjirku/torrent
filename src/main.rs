use clap::{Parser, Subcommand};
mod bencode;
mod torrent;

#[derive(Parser)]
#[command(
    version,
    author,
    about = "A brief description of your application",
    long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// print information parsed from the torrent file
    Info {
        /// The torrent file to parse
        #[arg(short, long, value_name = "FILE")]
        file: std::path::PathBuf,
    }
}

fn main() {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Info { file } => {
                println!("Parsing file: {:?}", file);
                let mut reader = std::io::BufReader::new(std::fs::File::open(file).unwrap());
                let mut parser = bencode::Parser::new(&mut reader);
                match parser.parse() {
                    Ok(data) => {
                        let data = torrent::TorrentFile::from_bencode(&data);
                        print!("{:?}", data);
                    },
                    Err(e) => {
                        eprintln!("Error parsing file: {:?}", e);
                    }
                }
            }
            // Handle other subcommands here
        }
    }

}
