mod hash;
mod index;
mod metadata;
mod organization;
mod clustering;
mod geonames;
mod network_io;
mod cli;

use std::error::Error;
use cli::{Cli, Commands};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse_args();

    if cli.verbose {
        eprintln!("Sift v0.1.0 - Photo organization utility");
    }

    match cli.command {
        Commands::Organize {
            source,
            destination,
            with_clustering,
            jobs: _,
            index: _,
        } => {
            println!(
                "Organizing photos from {:?} to {:?}",
                source, destination
            );
            if with_clustering {
                println!("Geographic clustering enabled");
            }
            println!("Organization feature not yet implemented");
        }

        Commands::Hash { path, recursive } => {
            println!("Hashing path: {:?}", path);
            if recursive {
                println!("Recursive mode enabled");
            }
            println!("Hash feature not yet implemented");
        }

        Commands::Index { path, limit } => {
            println!("Loading index from: {:?}", path);
            println!("Showing {} entries", limit);
            println!("Index display feature not yet implemented");
        }

        Commands::Cluster { source, details } => {
            println!("Clustering photos from: {:?}", source);
            if details {
                println!("Showing detailed cluster information");
            }
            println!("Clustering feature not yet implemented");
        }

        Commands::Benchmark {
            path,
            size_mb,
            iterations,
        } => {
            println!("Benchmarking performance on: {:?}", path);
            println!("Test file size: {} MB", size_mb);
            println!("Iterations: {}", iterations);
            println!("Benchmark feature not yet implemented");
        }
    }

    Ok(())
}
