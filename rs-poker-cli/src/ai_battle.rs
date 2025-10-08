use rs_poker_server::{
    handler::{game_make_action::MakeActionRequest, game_player_view::GamePlayerViewRequest},
    poker_client::PokerClient,
};
use rs_poker_types::{
    game::{GameInfo, GamePlayerView, GameSettings},
    player::Player,
};

use crate::{
    ai_player,
    frame::{self, Frame},
};

// Run single game between two AI players.
pub async fn ai_battle(client: PokerClient) {
    frame::start_session();

    let players = vec![
        Player::ai(
            "Alice",
            "gpt-4o-mini",
            "You are a poker player. Play to win.",
        ),
        Player::ai("Bob", "gpt-4o-mini", "You are a poker player. Play to win."),
    ];

    let game_info = play_game(&client, players).await;
    frame::end_session();
    render_current_frame(&client, &game_info).await;
}

pub async fn play_game(client: &PokerClient, players: Vec<Player>) -> GameInfo {
    let players_count = players.len();

    // Start new game.
    let mut game_info = client
        .new_game(&GameSettings {
            tournament_id: None,
            tournament_game_number: None,
            game_id: None,
            small_blind: 5.0,
            players,
            stacks: vec![100.0; players_count],
            hands: None,
            community_cards: None,
            dealer_index: 0,
        })
        .await
        .unwrap();

    let game_id = game_info.game_id.clone();

    println!("Started new game: {:?}", game_id);

    while !game_info.is_finished() {
        // Render frame
        render_current_frame(client, &game_info).await;

        let current_player = game_info.current_player().unwrap();
        let current_player_name = current_player.name().clone();

        let view: GamePlayerView = client
            .game_player_view(GamePlayerViewRequest {
                game_id: game_id.clone(),
                player_name: current_player.name().clone(),
            })
            .await
            .unwrap();

        // Get player view for the current player.
        let game_view = view.summary.clone();
        let possible_actions = view.possible_actions.clone();

        let decision = if let Player::AI {
            model, strategy, ..
        } = current_player
        {
            ai_player::decide(
                model.to_string(),
                strategy.to_string(),
                game_view,
                format!("{:?}", possible_actions),
            )
            .await
        } else {
            panic!("Current player is not an AI");
        };

        game_info = client
            .make_action(MakeActionRequest {
                game_id: game_id.clone(),
                player_name: current_player_name,
                decision,
            })
            .await
            .unwrap();
    }
    game_info
}

pub async fn current_frame(client: &PokerClient, game_info: &GameInfo) -> Frame {
    let view = client.game_full_view(&game_info.game_id).await.unwrap();
    let mut frame = Frame::new(&view.game_id);
    frame.with_game_info(game_info.clone());
    frame.with_game_summary(view.summary);
    frame
}

pub async fn render_current_frame(client: &PokerClient, game_info: &GameInfo) {
    let frame = current_frame(client, &game_info).await;
    frame::clean_frame();
    println!("{}", frame.render());
}
