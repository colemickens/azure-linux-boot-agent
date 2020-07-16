#[macro_use]
extern crate log;

use azure_linux_boot_agent as azlba;
use clap::Clap;

// TODO: more aggressively report failures as prov status (if ActivateNix fails, for example)
// TODO: support windows? (why though? just for fun to practice writing more rust?)

#[derive(Clap, PartialEq, Debug)]
enum MetadataMode {
    NoOp,
    Apply,
    Stash,
}

#[derive(Clap)]
struct Opts {
    #[clap(arg_enum, long = "metadata-mode", default_value = "stash")]
    metadata_mode: MetadataMode,

    #[clap(long = "seed-entropy", parse(try_from_str), default_value = "true")]
    seed_entropy: bool,

    #[clap(long = "create-users", parse(try_from_str), default_value = "true")]
    create_users: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    trace!("starting up");

    let opts: Opts = Opts::parse();

    let mut agent = azlba::agent::Agent::new()
        .await
        .expect("failed to init agent");

    let agent_opts = azlba::agent::AgentOptions {
        metadata_mode: match opts.metadata_mode {
            MetadataMode::NoOp => azlba::agent::AgentMetadataMode::NoOp,
            MetadataMode::Apply => azlba::agent::AgentMetadataMode::Apply,
            MetadataMode::Stash => azlba::agent::AgentMetadataMode::Stash,
        },
        seed_entropy: opts.seed_entropy,
        create_users: opts.create_users,
    };
    agent.provision(&agent_opts).await?;

    info!("provisioning complete");
    trace!("exiting");
    Ok(())
}
