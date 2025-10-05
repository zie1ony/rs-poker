use rs_poker_llm_client::{LLMClient, LLMResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub output: String,
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MathReasoningResponse {
    pub final_answer: String,
    pub steps: Vec<Step>,
}

impl LLMResponse for MathReasoningResponse {
    const DESCRIPTION: &'static str =
        "A response that breaks down the solution to a math problem into steps.";
}

#[tokio::main]
async fn main() {
    let client = LLMClient::<MathReasoningResponse>::new("gemini-2.5-flash");
    let response = client
        .respond(
            "You are a helpful math tutor. Guide the user through the solution step by step.",
            "how can I solve 8x + 7 = -23",
        )
        .await;
    println!("Response: {:#?}", response);
}
