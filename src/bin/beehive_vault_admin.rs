use std::{error::Error, fs, path::PathBuf};

use beehive_vault_api::{
    config::Settings,
    database::Database,
    features::{
        demo::{DEMO_HOUSEHOLD_ID, DemoRepository},
        institutions::admin::{InstitutionAdminService, InstitutionCatalog},
    },
    types::InstitutionId,
};
use clap::{Parser, Subcommand};
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Parser)]
#[command(name = "beehive_vault_admin")]
#[command(about = "Administer Beehive Vault server-side data")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Load or remove the fixed synthetic demonstration household.
    Demo {
        #[command(subcommand)]
        command: DemoCommand,
    },
    /// Manage the global financial institution catalog.
    Institutions {
        #[command(subcommand)]
        command: InstitutionCommand,
    },
}

#[derive(Debug, Subcommand)]
enum DemoCommand {
    /// Load the fixtures once, preserving an existing demonstration household.
    Load,
    /// Remove only the demonstration household and its unused demo institutions.
    Reset,
}

#[derive(Debug, Subcommand)]
enum InstitutionCommand {
    /// List institutions ordered by name.
    List,
    /// Add one institution.
    Add {
        #[arg(long)]
        name: String,
    },
    /// Rename one institution without changing its stable identifier.
    Rename {
        #[arg(long)]
        id: InstitutionId,
        #[arg(long)]
        name: String,
    },
    /// Add institutions missing from a JSON catalog file.
    Import {
        #[arg(long)]
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    let settings = Settings::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(settings.database_max_connections)
        .connect(&settings.database_url)
        .await?;
    sqlx::migrate!().run(&pool).await?;

    let database = Database::new(pool);
    let service = InstitutionAdminService::new(database.clone());
    match cli.command {
        Command::Demo { command } => {
            let demo = DemoRepository::new(database);
            let changed = match command {
                DemoCommand::Load => demo.load().await?,
                DemoCommand::Reset => demo.reset().await?,
            };
            println!("Demo household {DEMO_HOUSEHOLD_ID}: changed={changed}");
        }
        Command::Institutions { command } => run_institution_command(&service, command).await?,
    }

    Ok(())
}

async fn run_institution_command(
    service: &InstitutionAdminService,
    command: InstitutionCommand,
) -> Result<(), Box<dyn Error>> {
    match command {
        InstitutionCommand::List => {
            for institution in service.list().await? {
                println!("{}\t{}", institution.id, institution.name);
            }
        }
        InstitutionCommand::Add { name } => {
            let institution = service.add(name).await?;
            println!("Added {}\t{}", institution.id, institution.name);
        }
        InstitutionCommand::Rename { id, name } => {
            let institution = service.rename(id, name).await?;
            println!("Renamed {}\t{}", institution.id, institution.name);
        }
        InstitutionCommand::Import { file } => {
            let contents = fs::read_to_string(file)?;
            let report = service
                .import_catalog(InstitutionCatalog::from_json(&contents)?)
                .await?;
            println!(
                "Import complete: {} added, {} unchanged",
                report.added, report.unchanged
            );
        }
    }

    Ok(())
}
