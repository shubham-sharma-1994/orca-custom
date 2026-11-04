use orca_kernel::Host;
use orca_plugin_core_config::CoreConfig;
use orca_plugin_worktree::Worktree;
use serde_json::Value;

const USAGE: &str = "usage: orca-next plugins | commands | call <command> [json-args]";

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(args).await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // The only place that knows which plugins exist; swap for dynamic discovery later.
    let mut host = Host::builder().plugin(CoreConfig).plugin(Worktree).build()?;
    host.start().await?;
    let result = dispatch(&host, &args).await;
    host.shutdown().await;
    result
}

async fn dispatch(host: &Host, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["plugins"] => host.plugins().iter().for_each(|m| println!("{} {} - {}", m.id, m.version, m.description)),
        ["commands"] => host.commands().iter().for_each(|c| println!("{c}")),
        ["call", name, rest @ ..] => {
            let params: Value = rest.first().map(|s| serde_json::from_str(s)).transpose()?.unwrap_or(Value::Null);
            println!("{}", serde_json::to_string_pretty(&host.invoke(name, params).await?)?);
        }
        _ => return Err(USAGE.into()),
    }
    Ok(())
}
