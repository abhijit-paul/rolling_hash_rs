use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct GenSignatureArgs {
    #[arg(short, long, value_name = "OLD_FILE")]
    pub old_file: PathBuf,

    #[arg(short, long, value_name = "SIGNATURE_FILE")]
    pub signature_file: PathBuf,
}

#[derive(Parser)]
pub struct GenDiffArgs {
    #[arg(short, long, value_name = "SIGNATURE_FILE")]
    pub signature_file: PathBuf,

    #[arg(short, long, value_name = "NEW_FILE")]
    pub new_file: PathBuf,
    /// Delta file
    #[arg(short, long, value_name = "DELTA_FILE")]
    pub delta_file: PathBuf,
}

#[derive(Parser)]
pub enum SubCommand {
    GenerateSignature(GenSignatureArgs),
    GenerateDiff(GenDiffArgs),
}

#[derive(Parser)]
pub struct CliOptions {
    #[clap(subcommand)]
    pub sub_command: SubCommand,
}
