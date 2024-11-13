use clap::Parser;
use colored::Colorize;
use dotenv::dotenv;
use entropy_test_cli::{run_command, Cli, PROGRAM_VERSION_NUMBER};
use generate_types::generate_types;
use project_root::get_project_root;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let program = format!(
        "{}/target/wasm32-unknown-unknown/release/{{project-name}}.wasm",
        get_project_root()?.to_string_lossy()
    );
    generate_types();
    let config_interface = format!(
        "{}/{{project-name}}_serialized_config_type.txt",
        get_project_root()?.to_string_lossy()
    );
    let aux_data_interface = format!(
        "{}/{{project-name}}_serialized_aux_data_type.txt",
        get_project_root()?.to_string_lossy()
    );

    let cli = Cli::parse();
    let json_ouput = cli.json;
    match run_command(
        cli,
        Some(program.into()),
        Some(config_interface.into()),
        Some(aux_data_interface.into()),
        Some(PROGRAM_VERSION_NUMBER),
    )
    .await
    {
        Ok(output) => {
            if json_ouput {
                println!("{}", output);
            } else {
                println!("Success: {}", output.green());
            }
            Ok(())
        }
        Err(err) => {
            if !json_ouput {
                eprintln!("{}", "Failed!".red());
            }
            Err(err)
        }
    }
}
