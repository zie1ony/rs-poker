use rs_poker_llm_client::LLMClient;
use rs_poker_types::game::Decision;

const SYSTEM_PROMPT: &str = r#"
You are an expert Texas Hold'em poker player.
You take part in the Texas Hold'em poker tournament with max 10 games.
If the tournament reaches 10 games, the player with the highest total chips wins.
Winner takes it all.
You will be given the full tournament log so far, including all previous games and the current game state.
For your convenience, you will be given possible actions you can take.
Before you decide, think.
At the end make a decision what to do next, and only respond with one of the available actions.
Follow given strategy.

BETTING RULES:
- Use Bet(amount) to specify the TOTAL amount you want to bet in the current round.
- Minimum raise must equal the previous bet/raise amount (e.g., if big blind is X, minimum raise is X more, so Bet(2*X) total)
- If you're in small blind (Y) and want to raise, Bet(Z) means you're adding Z-Y more chips (Z total - Y already posted)
- Invalid bet amounts will result in an automatic fold, so always ensure your bet meets minimum requirements
- Use Call to match the current bet, AllIn to bet all remaining chips, or Fold to quit the hand
"#;

pub async fn decide(
    model: String,
    strategy: String,
    game_view: String,
    possible_actions: String,
) -> Decision {
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
        .respond(SYSTEM_PROMPT, &user_prompt)
        .await
        .expect("Failed to get response from LLM");
    response
}
