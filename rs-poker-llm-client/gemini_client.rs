use std::env;

use gemini_rust::Gemini;
use serde_json::json;

#[derive(serde::Deserialize, Debug)]
pub struct MyResponse {
    reason: String,
    action: MyAction,
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum MyAction {
    SaySomething { message: String },
    DoNothing,
}

#[tokio::main]
pub async fn main() {
    dotenv::dotenv().ok();
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    // Define a simple JSON schema that Gemini accepts (without $schema, definitions, $ref)
    let schema = json!({
        "type": "object",
        "properties": {
            "reason": {
                "type": "string",
                "description": "Reason for the response about the programming language"
            },
            "action": {
                "type": "object",
                "properties": {
                    "type": {
                        "type": "string",
                        "enum": ["SaySomething", "DoNothing"],
                        "description": "Type of action to take"
                    },
                    "message": {
                        "type": "string",
                        "description": "Message to say (only required if type is SaySomething)"
                    }
                },
                "required": ["type"],
                "description": "Action to take in response"
            }
        },
        "required": ["reason", "action"]
    });

    println!("Using schema: {:#}", schema);

    let response = client
        .generate_content()
        .with_system_prompt("You provide information about programming languages in JSON format. Always respond with the specified JSON structure.")
        .with_user_message("Don't say anything.")
        .with_response_mime_type("application/json")
        .with_response_schema(schema)
        .execute()
        .await
        .unwrap();

    let json_response: serde_json::Value = serde_json::from_str(&response.text()).unwrap();

    println!("Response: {:#?}", json_response);

    let parsed: MyResponse = serde_json::from_value(json_response).unwrap();

    println!("Parsed response: {:#?}", parsed);

}