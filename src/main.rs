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
                        let data = torrent::TorrentFile::from_bencode(&data).unwrap();
                        println!("");
                        println!("announce: \"{}\"", data.announce);
                        println!("info:");
                        if let Some(l) = data.info.length {
                            println!("  length: {}", l);
                        }
                        println!("   piece_length: {}", data.info.piece_length);
                        if let Some(f) = data.info.files {
                            println!("   files:");
                            for f in f {
                                if f.path.len() > 0 {
                                    println!("      - {} [{}]", &f.path[0], f.length);
                                }
                            }
                        }
                        
                        println!("   pieces:");
                        let mut i = 0;
                        for s in data.info.pieces.0 {
                            i += 1;
                            if i > 10 {
                                println!("       - ...more...");
                                break;
                            }
                            let id: String = s.iter().map(|c| format!("{:02X}", c)).collect();
                            println!("       - {}", &id);
                        }
                    },
                    Err(e) => {
                        eprintln!("Error parsing file: {:?}", e);
                    }
                }
            }
        }
    }

}
