use rs_poker_llm_client::LLMClient;
use rs_poker_types::game::Decision;



pub async fn decide(
    model: String,
    system_promot: String,
    strategy: String,
    game_view: String,
    possible_actions: String,
) -> (String, Decision) {
    let client = LLMClient::<Decision>::new(&model);
    let user_prompt = format!(
        r#"
<Game>
{}
</Game>
<Strategy>
{}
</Strategy>
<Actions>
{}
</Actions>"#,
        game_view, strategy, possible_actions
    );

    let response = client
        .respond(&system_promot, &user_prompt)
        .await
        .expect("Failed to get response from LLM");
    (user_prompt, response)
}
