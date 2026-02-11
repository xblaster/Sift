//! Command-line interface for Sift photo organization utility.
//!
//! This module provides the CLI argument parsing using Clap, supporting
//! multiple subcommands for organizing photos, computing hashes, clustering,
//! and benchmarking performance on network storage.
//!
//! # Examples
//!
//! ```bash
//! # Organize photos with geographic clustering
//! sift organize /source /dest --with-clustering
//!
//! # Hash a file or directory
//! sift hash /photos --recursive
//!
//! # Run performance benchmark
//! sift benchmark /mnt/smb --size-mb 500 --iterations 10
//! ```

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// The main CLI struct containing the command and global options.
///
/// This struct is populated by Clap when parsing command-line arguments.
/// It supports a single subcommand plus global flags like `--verbose`.
///
/// # Example
///
/// ```no_run
/// # use sift::cli::Cli;
/// let cli = Cli::parse_args();
/// // Handle the command...
/// ```
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

    /// Enable verbose output for debugging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

/// Available CLI commands for Sift.
///
/// Each variant represents a different operation the user can perform.
#[derive(Subcommand)]
pub enum Commands {
    /// Organize photos from source to destination with automatic classification.
    ///
    /// Copies photos from the source directory to the destination, organizing them
    /// into a chronological folder structure (YYYY/MM/DD/). Optionally applies
    /// geographic clustering if metadata is available.
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

        /// Preview changes without copying files
        #[arg(short, long)]
        dry_run: bool,
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
    /// Parses command-line arguments into a Cli struct.
    ///
    /// Uses Clap's default parsing mechanism to read arguments from std::env::args().
    /// Automatically prints help and exits on parse errors or --help.
    ///
    /// # Returns
    ///
    /// A Cli struct containing the parsed command and options
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sift::cli::Cli;
    /// let cli = Cli::parse_args();
    /// if cli.verbose {
    ///     eprintln!("Verbose mode enabled");
    /// }
    /// ```
    pub fn parse_args() -> Self {
        Parser::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organize_command_basic() {
        let args = vec!["sift", "organize", "/source", "/dest"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Organize {
                source,
                destination,
                with_clustering,
                jobs,
                index,
                dry_run,
            } => {
                assert_eq!(source.to_str().unwrap(), "/source");
                assert_eq!(destination.to_str().unwrap(), "/dest");
                assert!(!with_clustering);
                assert!(jobs.is_none());
                assert!(index.is_none());
                assert!(!dry_run);
            }
            _ => panic!("Expected Organize command"),
        }
    }

    #[test]
    fn test_organize_command_with_clustering() {
        let args = vec![
            "sift",
            "organize",
            "/source",
            "/dest",
            "--with-clustering",
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Organize {
                with_clustering, ..
            } => {
                assert!(with_clustering);
            }
            _ => panic!("Expected Organize command"),
        }
    }

    #[test]
    fn test_organize_command_with_jobs() {
        let args = vec![
            "sift",
            "organize",
            "/source",
            "/dest",
            "--jobs",
            "8",
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Organize { jobs, .. } => {
                assert_eq!(jobs, Some(8));
            }
            _ => panic!("Expected Organize command"),
        }
    }

    #[test]
    fn test_hash_command_recursive() {
        let args = vec!["sift", "hash", "/photos", "--recursive"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Hash { path, recursive } => {
                assert_eq!(path.to_str().unwrap(), "/photos");
                assert!(recursive);
            }
            _ => panic!("Expected Hash command"),
        }
    }

    #[test]
    fn test_hash_command_single_file() {
        let args = vec!["sift", "hash", "/photo.jpg"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Hash { path, recursive } => {
                assert_eq!(path.to_str().unwrap(), "/photo.jpg");
                assert!(!recursive);
            }
            _ => panic!("Expected Hash command"),
        }
    }

    #[test]
    fn test_index_command() {
        let args = vec!["sift", "index", "index.bin", "--limit", "50"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Index { path, limit } => {
                assert_eq!(path.to_str().unwrap(), "index.bin");
                assert_eq!(limit, 50);
            }
            _ => panic!("Expected Index command"),
        }
    }

    #[test]
    fn test_cluster_command() {
        let args = vec!["sift", "cluster", "/photos", "--details"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Cluster { source, details } => {
                assert_eq!(source.to_str().unwrap(), "/photos");
                assert!(details);
            }
            _ => panic!("Expected Cluster command"),
        }
    }

    #[test]
    fn test_benchmark_command() {
        let args = vec![
            "sift",
            "benchmark",
            "/mnt/smb",
            "--size-mb",
            "200",
            "-n",
            "10",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Benchmark {
                path,
                size_mb,
                iterations,
            } => {
                assert_eq!(path.to_str().unwrap(), "/mnt/smb");
                assert_eq!(size_mb, 200);
                assert_eq!(iterations, 10);
            }
            _ => panic!("Expected Benchmark command"),
        }
    }

    #[test]
    fn test_verbose_flag() {
        let args = vec!["sift", "--verbose", "organize", "/source", "/dest"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
    }

    #[test]
    fn test_no_verbose_flag() {
        let args = vec!["sift", "organize", "/source", "/dest"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(!cli.verbose);
    }

    #[test]
    fn test_organize_with_all_options() {
        let args = vec![
            "sift",
            "--verbose",
            "organize",
            "/src",
            "/dst",
            "--with-clustering",
            "--jobs",
            "4",
            "--index",
            "my_index.bin",
            "--dry-run",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
        match cli.command {
            Commands::Organize {
                source,
                destination,
                with_clustering,
                jobs,
                index,
                dry_run,
            } => {
                assert_eq!(source.to_str().unwrap(), "/src");
                assert_eq!(destination.to_str().unwrap(), "/dst");
                assert!(with_clustering);
                assert_eq!(jobs, Some(4));
                assert_eq!(index.as_ref().unwrap().to_str().unwrap(), "my_index.bin");
                assert!(dry_run);
            }
            _ => panic!("Expected Organize command"),
        }
    }

    #[test]
    fn test_organize_dry_run_flag() {
        let args = vec!["sift", "organize", "/source", "/dest", "--dry-run"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Organize { dry_run, .. } => {
                assert!(dry_run);
            }
            _ => panic!("Expected Organize command"),
        }
    }

    #[test]
    fn test_organize_without_dry_run() {
        let args = vec!["sift", "organize", "/source", "/dest"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Organize { dry_run, .. } => {
                assert!(!dry_run);
            }
            _ => panic!("Expected Organize command"),
        }
    }
}
