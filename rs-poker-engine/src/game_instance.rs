use rand::rngs::ThreadRng;
use rs_poker::{
    arena::{Agent, action::AgentAction, agent::RandomAgent},
    core::{Card, Deck},
};
use rs_poker_types::{
    game::{
        Decision, GameFinalResults, GameFullView, GameId, GamePlayerView, GameSettings, GameStatus,
    },
    player::{AutomatType, Player, PlayerName},
};

use crate::{
    game_simulation::{GameActionRequired, GameSimulation},
    game_summary::GameSummary,
};

#[derive(Clone)]
pub struct GameInstance {
    pub game_id: GameId,
    pub simulation: GameSimulation,
    pub players: Vec<Player>,
}

impl GameInstance {
    pub fn new(
        game_id: GameId,
        players: Vec<Player>,
        initial_stacks: Vec<f32>,
        big_blind: f32,
        small_blind: f32,
        player_hands: Vec<[Card; 2]>,
        community_cards: [Card; 5],
    ) -> Self {
        let simulation = GameSimulation::new(
            game_id.clone(),
            big_blind,
            small_blind,
            initial_stacks.clone(),
            players.clone(),
            player_hands,
            community_cards,
            players.iter().map(|p| p.name()).collect(),
        );
        Self {
            game_id,
            simulation,
            players,
        }
    }

    pub fn new_from_config_with_random_cards(config: &GameSettings) -> Self {
        Self::new_with_random_cards(
            config.game_id.clone(),
            config.players.clone(),
            config.stacks.clone(),
            config.small_blind * 2.0,
            config.small_blind,
        )
    }

    pub fn new_with_random_cards(
        game_id: GameId,
        players: Vec<Player>,
        initial_stacks: Vec<f32>,
        big_blind: f32,
        small_blind: f32,
    ) -> Self {
        let mut rng = rand::rng();
        let mut deck = Deck::default();

        let player_hands: Vec<[Card; 2]> = players
            .iter()
            .map(|_| {
                let cards = n_cards(&mut deck, 2, &mut rng);
                [cards[0], cards[1]]
            })
            .collect();

        let community_cards_vec = n_cards(&mut deck, 5, &mut rng);
        let community_cards: [Card; 5] = [
            community_cards_vec[0],
            community_cards_vec[1],
            community_cards_vec[2],
            community_cards_vec[3],
            community_cards_vec[4],
        ];

        Self::new(
            game_id,
            players,
            initial_stacks,
            big_blind,
            small_blind,
            player_hands,
            community_cards,
        )
    }

    pub fn game_id(&self) -> GameId {
        self.game_id.clone()
    }

    pub fn run(&mut self) {
        loop {
            match self.simulation.run() {
                GameActionRequired::PlayerToAct {
                    idx,
                    possible_actions: _,
                } => match &self.players[idx] {
                    Player::Automat {
                        automat_type,
                        name: _,
                    } => {
                        let decision = match automat_type {
                            AutomatType::Random => {
                                let mut agent = RandomAgent::default();
                                let action = agent.act(0u128, &self.simulation.game_state);
                                Decision {
                                    action,
                                    reason: String::from("RandomAgent decision"),
                                }
                            }
                            AutomatType::AllIn => Decision {
                                action: AgentAction::AllIn,
                                reason: String::from("AllIn Automat decision"),
                            },
                            AutomatType::Calling => Decision {
                                action: AgentAction::Call,
                                reason: String::from("Calling Automat decision"),
                            },
                            AutomatType::Filding => Decision {
                                action: AgentAction::Fold,
                                reason: String::from("Filding Automat decision"),
                            },
                        };
                        self.simulation.execute_player_action(decision);
                    }
                    Player::Human { name: _ } => return,
                    Player::AI {
                        name: _,
                        model: _,
                        strategy: _,
                    } => return,
                },
                GameActionRequired::NoActionRequired => return,
            }
        }
    }

    pub fn excute_player_action(&mut self, decision: Decision) {
        self.simulation.execute_player_action(decision);
    }

    pub fn as_game_full_view(&self) -> GameFullView {
        GameFullView {
            game_id: self.game_id.clone(),
            status: self.game_status(),
            summary: GameSummary::full(self.simulation.events.clone()).summary(),
        }
    }

    pub fn as_game_player_view(&self, player_name: &PlayerName) -> GamePlayerView {
        // Check if the requested player is currently playing.
        let current_player_idx = self.simulation.game_state.to_act_idx();
        let current_player_name = self.players[current_player_idx].name();
        let summary =
            GameSummary::for_player(self.simulation.events.clone(), player_name.clone()).summary();
        let is_active_player = &current_player_name == player_name;
        let possible_actions = if is_active_player {
            self.simulation.get_possible_actions_for_current_player()
        } else {
            vec![]
        };

        GamePlayerView {
            game_id: self.game_id.clone(),
            player: player_name.clone(),
            is_active_player,
            summary,
            possible_actions,
        }
    }

    pub fn game_status(&self) -> GameStatus {
        if self.simulation.game_state.is_complete() {
            GameStatus::Finished
        } else {
            GameStatus::InProgress
        }
    }

    pub fn is_complete(&self) -> bool {
        self.simulation.game_state.is_complete()
    }

    pub fn current_player_name(&self) -> Option<PlayerName> {
        if self.simulation.game_state.is_complete() {
            None
        } else {
            let idx = self.simulation.game_state.to_act_idx();
            Some(self.players[idx].name())
        }
    }

    pub fn actions_str(&self) -> String {
        self.simulation
            .actions
            .iter()
            .map(|a| format!("{:?}\n", a))
            .collect()
    }

    pub fn game_final_results(&self) -> Option<GameFinalResults> {
        if self.is_complete() {
            Some(GameFinalResults {
                game_id: self.game_id.clone(),
                player_names: self.players.iter().map(|p| p.name()).collect(),
                final_stacks: self.simulation.game_state.stacks.clone(),
            })
        } else {
            None
        }
    }
}

fn n_cards(deck: &mut Deck, n: usize, rng: &mut ThreadRng) -> Vec<Card> {
    let mut cards = Vec::with_capacity(n);
    for _ in 0..n {
        if let Some(card) = deck.deal(rng) {
            cards.push(card);
        } else {
            panic!("Not enough cards in the deck");
        }
    }
    cards
}
