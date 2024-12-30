use agentai::Agent;
use anyhow::Result;
use genai::Client;
use log::{info, LevelFilter};
use schemars::JsonSchema;
use serde::Deserialize;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

const MODEL: &str = "gpt-4o-mini";

const SYSTEM: &str = "You are helpful assistant";

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

    let question = "Why sky is blue?";

    info!("Question: {}", question);

    let mut agent = Agent::new(&client, SYSTEM, &ctx);

    let answer: Answer = agent.run(MODEL, question).await?;

    info!("{:#?}", answer);

    Ok(())
}

struct Ctx {}

#[derive(Deserialize, JsonSchema, Debug)]
struct Answer {
    // It is always good idea to include thinking field for LLM's debugging
    /// In this field provide your thinking steps
    #[serde(rename = "_thinking")]
    thinking: String,

    /// In this field provide answer
    answer: String,
}
