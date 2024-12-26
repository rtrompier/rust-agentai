use agentai::agent::Agent;
use anyhow::Result;
use genai::Client;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

const MODEL: &str = "";

const SYSTEM: &str = "";

#[tokio::main]
async fn main() -> Result<()> {
    TermLogger::init(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;
    info!("Starting AgentAI");

    // Creating empty Context, in this example we don't require to use Context, but object need
    // to be passed and initialised.
    let ctx = Ctx {};

    // Creating GenAI client
    let client = Client::default();

    let question = "";

    info!("Question: {}", question);

    let mut agent = Agent::new(&client, SYSTEM, &ctx);

    let answer: String = agent.run(MODEL, question).await?;

    info!("Answer: {}", answer);

    Ok(())
}

struct Ctx {}
