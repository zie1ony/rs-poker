use std::{
    any::type_name,
    error::Error,
    marker::PhantomData,
    sync::{Arc, Mutex, OnceLock},
};

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
        CreateChatCompletionRequestArgs, ResponseFormat, ResponseFormatJsonSchema,
    },
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

pub fn init_dotenv() {
    dotenv::dotenv().ok();
}

// --- Stats ---

const PRINT_STATS: bool = false;

#[derive(Clone)]
pub struct Stats {
    llm_calls: u32,
    input_tokens: u32,
    output_tokens: u32,
}

static STATS: OnceLock<Arc<Mutex<Stats>>> = OnceLock::new();

pub fn get_stats_value() -> Stats {
    get_stats().lock().unwrap().clone()
}

fn get_stats() -> Arc<Mutex<Stats>> {
    STATS
        .get_or_init(|| {
            Arc::new(Mutex::new(Stats {
                llm_calls: 0,
                input_tokens: 0,
                output_tokens: 0,
            }))
        })
        .clone()
}

fn increment_llm_call() {
    let stats = get_stats();
    let mut stats_guard = stats.lock().unwrap();
    stats_guard.llm_calls += 1;
}

fn update_token_stats(input_tokens: u32, output_tokens: u32) {
    let stats = get_stats();
    let mut stats_guard = stats.lock().unwrap();
    stats_guard.input_tokens += input_tokens;
    stats_guard.output_tokens += output_tokens;
}

pub fn print_stats() {
    let stats = get_stats();
    let stats_guard = stats.lock().unwrap();
    println!("LLM Stats:");
    println!("  Calls: {}", stats_guard.llm_calls);
    println!("  Input tokens: {}", stats_guard.input_tokens);
    println!("  Output tokens: {}", stats_guard.output_tokens);
}

pub fn count_tokens(input: &str) -> usize {
    use tiktoken_rs::o200k_base;
    let bpe = o200k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens(input);
    tokens.len()
}

// Models
pub enum LLM {
    GPT5,
    GPT5Mini,
    GPT5Nano,
    Gemini25FlashLight,
}

pub enum Provider {
    OpenAI,
    OpenRouter,
    Google,
}

pub struct LLMDef {
    pub name: LLM,
    pub provider: Provider,
}

impl LLMDef {
    pub fn from(llm: LLM) -> Self {
        match llm {
            // --- OpenAI ---
            LLM::GPT5 => Self {
                name: LLM::GPT5,
                provider: Provider::OpenAI,
            },
            LLM::GPT5Mini => Self {
                name: LLM::GPT5Mini,
                provider: Provider::OpenAI,
            },
            LLM::GPT5Nano => Self {
                name: LLM::GPT5Nano,
                provider: Provider::OpenAI,
            },
            // --- Google ---
            LLM::Gemini25FlashLight => Self {
                name: LLM::Gemini25FlashLight,
                provider: Provider::Google,
            },
        }
    }
}

// --- OpenAI Client ---
const USE_OPENROUTER: bool = false;

fn client() -> Client<OpenAIConfig> {
    if USE_OPENROUTER {
        let api_key =
            std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY env var not set.");
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base("https://openrouter.ai/api/v1");
        Client::with_config(config)
    } else {
        Client::new()
    }
}

pub trait LLMResponse: serde::Serialize + DeserializeOwned + JsonSchema {
    const DESCRIPTION: &'static str;

    fn name() -> &'static str {
        let full = type_name::<Self>();
        full.rsplit("::").next().unwrap_or(full)
    }
}

pub struct LLMClient<T: LLMResponse> {
    model: String,
    response_type: PhantomData<T>,
}

impl<T: LLMResponse> LLMClient<T> {
    pub fn new(model: &str) -> Self {
        init_dotenv();
        Self {
            model: model.to_string(),
            response_type: PhantomData,
        }
    }

    pub async fn respond(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<T, Box<dyn Error>> {
        increment_llm_call();
        let messages = vec![
            ChatCompletionRequestSystemMessage::from(system_prompt).into(),
            ChatCompletionRequestUserMessage::from(user_prompt).into(),
        ];

        // let schema = schemars::schema_for!(T);
        // let schema_value = serde_json::to_value(&schema.schema)?;
        let schema = openai_schemars::Schema::new::<T>().unwrap();
        let schema_value = serde_json::to_value(&schema.value)?;
        // print!("Using schema: {:#}", schema_value);
        let response_format = ResponseFormat::JsonSchema {
            json_schema: ResponseFormatJsonSchema {
                description: Some(String::from(T::DESCRIPTION)),
                name: String::from(T::name()),
                schema: Some(schema_value),
                strict: Some(true),
            },
        };

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .response_format(response_format)
            .build()?;

        // println!("LLM {} request: {:#?}", &self.model, &request);

        let client = client();
        let start = std::time::Instant::now();
        let response = client.chat().create(request).await;
        // println!("Full response: {:#?}", &response);
        let response = response?;
        let duration = start.elapsed();
        if PRINT_STATS {
            println!("LLM {} request took: {:?}", &self.model, duration);
        }

        // Update token stats if usage information is available
        if let Some(usage) = response.usage {
            update_token_stats(usage.prompt_tokens, usage.completion_tokens);
        }

        if PRINT_STATS {
            print_stats();
        }
        for choice in response.choices {
            if let Some(content) = choice.message.content {
                return Ok(serde_json::from_str::<T>(&content)?);
            }
        }

        Err("No valid response from LLM".into())
    }
}
