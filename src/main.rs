use clap::Parser;
use cli_parser::*;
use handlers::file_diff::write_diff_file;
use handlers::file_io::{read_handler, write_handler};
use handlers::signature::write_signature_file;

mod cli_parser;
mod handlers;

fn main() {
    let opts = CliOptions::parse();

    match opts.sub_command {
        SubCommand::GenerateSignature(gen_sign_command) => {
            let old_file = read_handler(&gen_sign_command.old_file).unwrap();
            let mut signature_file = write_handler(&gen_sign_command.signature_file).unwrap();
            write_signature_file(&old_file, &mut signature_file).unwrap();
            println!(
                "Generated signature file: {}",
                gen_sign_command.signature_file.display()
            );
        }
        SubCommand::GenerateDiff(gen_diff_command) => {
            let signature_file = read_handler(&gen_diff_command.signature_file).unwrap();
            let new_file = read_handler(&gen_diff_command.new_file).unwrap();
            let mut diff_file = write_handler(&gen_diff_command.delta_file).unwrap();
            write_diff_file(&signature_file, &new_file, &mut diff_file).unwrap();
            println!(
                "Generated diff file: {}",
                gen_diff_command.delta_file.display()
            );
        }
    }
}
