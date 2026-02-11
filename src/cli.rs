use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "Sift",
    version = "0.1.0",
    about = "High-performance photo organization utility for network storage",
    long_about = "Sift organizes massive photo libraries on network storage (SMB/NFS) with automatic chronological and geographic clustering."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Organize photos from source to destination
    Organize {
        /// Source directory containing photos
        #[arg(value_name = "SOURCE")]
        source: PathBuf,

        /// Destination directory for organized photos
        #[arg(value_name = "DESTINATION")]
        destination: PathBuf,

        /// Enable geographic clustering
        #[arg(short, long)]
        with_clustering: bool,

        /// Number of parallel workers (default: CPU count)
        #[arg(short = 'j', long)]
        jobs: Option<usize>,

        /// Path to load/save index file
        #[arg(short, long)]
        index: Option<PathBuf>,
    },

    /// Hash a file or directory
    Hash {
        /// File or directory to hash
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Compute hash for all files in directory recursively
        #[arg(short, long)]
        recursive: bool,
    },

    /// Show index contents
    Index {
        /// Path to index file
        #[arg(value_name = "INDEX_FILE")]
        path: PathBuf,

        /// Number of entries to display
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Perform geographic clustering on EXIF data
    Cluster {
        /// Source directory containing photos
        #[arg(value_name = "SOURCE")]
        source: PathBuf,

        /// Show cluster details
        #[arg(short, long)]
        details: bool,
    },

    /// Test performance on network share
    Benchmark {
        /// Path to network share or local path for testing
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// File size to create for testing (in MB)
        #[arg(short, long, default_value = "100")]
        size_mb: usize,

        /// Number of test iterations
        #[arg(short = 'n', long, default_value = "5")]
        iterations: usize,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Parser::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parser() {
        let args = vec![
            "sift",
            "organize",
            "/source",
            "/dest",
            "--with-clustering",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.verbose == false);

        match cli.command {
            Commands::Organize { .. } => (),
            _ => panic!("Expected Organize command"),
        }
    }
}
