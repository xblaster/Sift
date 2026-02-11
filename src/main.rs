//! Sift - High-performance photo organization utility for network storage
//!
//! Sift is a Rust-based CLI tool for organizing massive photo libraries on network
//! storage (SMB/NFS) with minimal dependencies and maximum performance.
//!
//! # Features
//!
//! - **Blake3 Hashing**: Fast, parallelized file hashing for duplicate detection
//! - **Local Index**: Persistent indexing for idempotent operations
//! - **Date Extraction**: Automatic date extraction from file metadata
//! - **Chronological Organization**: Automatic folder hierarchy (YYYY/MM/DD/)
//! - **Geographic Clustering**: DBSCAN-based spatial clustering with reverse geocoding
//! - **Network Optimization**: Buffered I/O and exponential backoff retry logic
//! - **Full CLI**: Comprehensive command-line interface with multiple operations
//!
//! # Architecture
//!
//! The application is organized into functional modules:
//!
//! - `hash`: Blake3 hashing engine with parallelization
//! - `index`: Persistent deduplication index
//! - `metadata`: Date extraction from file metadata
//! - `organization`: Folder structure management
//! - `clustering`: Geographic clustering with reverse geocoding
//! - `geonames`: Embedded location database
//! - `network_io`: Network-optimized I/O operations
//! - `cli`: Command-line argument parsing
//!
//! # Examples
//!
//! ```bash
//! # Organize photos with automatic clustering
//! sift organize /source/photos /destination/organized --with-clustering
//!
//! # Hash a single file
//! sift hash /photos/image.jpg
//!
//! # Hash an entire directory in parallel
//! sift hash /photos --recursive
//!
//! # View index contents
//! sift index my_index.bin --limit 20
//!
//! # Benchmark network performance
//! sift benchmark /mnt/network/share --size-mb 500
//! ```

pub mod error;
pub mod hash;
pub mod index;
pub mod metadata;
pub mod organization;
pub mod clustering;
pub mod geonames;
pub mod network_io;
pub mod cli;
pub mod organize;

use std::error::Error;
use cli::{Cli, Commands};
use organize::{OrganizeContext, Orchestrator};

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
            jobs,
            index,
        } => {
            let ctx = OrganizeContext::new(source, destination, with_clustering, jobs, index);
            let mut orchestrator = Orchestrator::new(ctx);
            orchestrator.run()?;
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
