use clap::{Parser, Subcommand};
use rs_poker_cli::{
    run_game::{client, run_example_game},
    series_runner,
};
use rs_poker_server::handler::{game_full_view::GameFullViewRequest, game_list::ListGamesRequest};
use rs_poker_types::{
    game::GameId,
    player::{Player, PlayerName},
    series::{SeriesId, SeriesSettings},
    tournament::{TournamentEndCondition, TournamentId, TournamentSettings},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "poker-cli")]
#[command(about = "A CLI for interacting with the poker server")]
struct Cli {
    /// Use mock server instead of HTTP server
    #[arg(short, long)]
    mock_server: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Game-related commands
    Game {
        #[command(subcommand)]
        command: GameCommands,
    },
    /// Tournament-related commands
    Tournament {
        #[command(subcommand)]
        command: TournamentCommands,
    },

    /// Run AI Players.
    Tower {
        /// Number of worker threads to spawn.
        #[arg(short, long, default_value = "1")]
        workers: usize,

        /// Maximum number of tasks to process before exiting.
        #[arg(short, long, default_value = "1")]
        max_tasks: usize,
    },

    Series {
        #[command(subcommand)]
        command: SeriesCommand,
    },
}

#[derive(Subcommand)]
enum GameCommands {
    /// Start a new poker game
    Play,
    /// List all games
    List {
        /// Show only active games
        #[arg(short, long)]
        active_only: bool,
    },
    /// Show full view of a specific game
    View {
        /// Game ID to show
        game_id: String,
        /// Show debug information
        #[arg(short, long)]
        debug: bool,
    },
}

#[derive(Subcommand)]
enum TournamentCommands {
    /// Create a new tournament
    New {
        #[command(subcommand)]
        tournament_type: TournamentType,
    },
    /// List all tournaments
    List {
        /// Show only active tournaments
        #[arg(short, long)]
        active_only: bool,
    },
    /// Show tournament info
    Info {
        /// Tournament ID to show
        tournament_id: String,
    },
    /// Show full tournament view
    FullView {
        /// Tournament ID to view
        tournament_id: String,
    },
    /// Show tournament view for a specific player
    PlayerView {
        /// Player name
        player_name: String,
        /// Tournament ID to view
        tournament_id: String,
    },
}

#[derive(Subcommand)]
enum TournamentType {
    /// Random 3 players tournament.
    Random3,
    /// 3 AI players tournament.
    AI3,
}

#[derive(Subcommand)]
enum SeriesCommand {
    /// Run a series of tournaments.
    Run,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Game { command } => match command {
            GameCommands::Play => {
                run_example_game(cli.mock_server).await;
            }
            GameCommands::List { active_only } => {
                list_games(cli.mock_server, active_only).await;
            }
            GameCommands::View { game_id, debug } => {
                game_full_view(cli.mock_server, game_id, debug).await;
            }
        },
        Commands::Tournament { command } => match command {
            TournamentCommands::New { tournament_type } => {
                create_tournament(cli.mock_server, tournament_type).await;
            }
            TournamentCommands::List { active_only } => {
                list_tournaments(cli.mock_server, active_only).await;
            }
            TournamentCommands::Info { tournament_id } => {
                tournament_info(cli.mock_server, tournament_id).await;
            }
            TournamentCommands::FullView { tournament_id } => {
                tournament_full_view(cli.mock_server, tournament_id).await;
            }
            TournamentCommands::PlayerView {
                player_name,
                tournament_id,
            } => {
                tournament_player_view(cli.mock_server, player_name, tournament_id).await;
            }
        },
        Commands::Tower { workers, max_tasks } => {
            rs_poker_tower::run(workers, max_tasks).await;
        }
        Commands::Series { command } => match command {
            SeriesCommand::Run => {
                run_series(cli.mock_server).await;
            }
        },
    }
}

async fn game_full_view(mock_server: bool, game_id: String, debug: bool) {
    let client = client(mock_server);

    match client
        .game_full_view(GameFullViewRequest {
            game_id: GameId::new(&game_id),
            debug,
        })
        .await
    {
        Ok(game_view) => {
            println!("Game ID: {}", game_id);
            println!("Status: {:?}", game_view.status);
            println!();
            println!("{}", game_view.summary);
        }
        Err(e) => {
            eprintln!("Error fetching game view: {:?}", e);
            std::process::exit(1);
        }
    }
}

async fn list_games(mock_server: bool, active_only: bool) {
    let client = client(mock_server);

    match client.list_games(ListGamesRequest { active_only }).await {
        Ok(response) => {
            if response.game_ids.is_empty() {
                if active_only {
                    println!("No active games found.");
                } else {
                    println!("No games found.");
                }
            } else {
                for (game_id, status) in response.game_ids {
                    println!("{} - {:?}", game_id, status);
                }
            }
        }
        Err(e) => {
            eprintln!("Error listing games: {:?}", e);
            std::process::exit(1);
        }
    }
}

async fn create_tournament(mock_server: bool, tournament_type: TournamentType) {
    let settings = match tournament_type {
        TournamentType::Random3 => TournamentSettings {
            tournament_id: TournamentId::random(),
            players: vec![
                Player::random("Alice"),
                Player::random("Bob"),
                Player::random("Charlie"),
            ],
            starting_player_stack: 100.0,
            starting_small_blind: 10.0,
            double_blinds_every_n_games: Some(2),
            end_condition: TournamentEndCondition::SingleWinner,
            see_historical_thoughts: true,
            public_chat: false,
        },
        TournamentType::AI3 => TournamentSettings {
            tournament_id: TournamentId::random(),
            players: vec![
                Player::ai("Alice", "gpt-4o-mini", "Play tight aggressive"),
                Player::ai("Bob", "gpt-4o-mini", "Play loose aggressive"),
                Player::ai("Charlie", "gpt-4o-mini", "Play tight passive"),
            ],
            starting_player_stack: 100.0,
            starting_small_blind: 10.0,
            double_blinds_every_n_games: Some(2),
            end_condition: TournamentEndCondition::SingleWinner,
            see_historical_thoughts: true,
            public_chat: true,
        },
    };
    let client = client(mock_server);
    match client.new_tournament(settings).await {
        Ok(response) => {
            println!("Created tournament with ID: {:?}", response.tournament_id);
        }
        Err(e) => {
            eprintln!("Error creating tournament: {:?}", e);
            std::process::exit(1);
        }
    }
}

async fn list_tournaments(mock_server: bool, active_only: bool) {
    let client = client(mock_server);

    match client
        .list_tournaments(
            rs_poker_server::handler::tournament_list::ListTournamentsRequest { active_only },
        )
        .await
    {
        Ok(response) => {
            if response.tournament_ids.is_empty() {
                if active_only {
                    println!("No active tournaments found.");
                } else {
                    println!("No tournaments found.");
                }
            } else {
                for (tournament_id, status) in response.tournament_ids {
                    println!("{:?} - {:?}", tournament_id, status);
                }
            }
        }
        Err(e) => {
            eprintln!("Error listing tournaments: {:?}", e);
            std::process::exit(1);
        }
    }
}

async fn tournament_info(mock_server: bool, tournament_id: String) {
    let client = client(mock_server);
    let tournament_id = TournamentId(tournament_id);
    match client.tournament_info(&tournament_id).await {
        Ok(info) => {
            // let settings = &info.settings;
            println!("Tournament ID: {:?}", tournament_id);
            println!("Game played: {}", info.games_played);
            println!("Status: {:?}", info.status);
            println!("Current game ID: {:?}", info.current_game_id);
        }
        Err(e) => {
            eprintln!("Error fetching tournament info: {:?}", e);
            std::process::exit(1);
        }
    }
}

async fn tournament_full_view(mock_server: bool, tournament_id: String) {
    let client = client(mock_server);
    let tournament_id = TournamentId(tournament_id);

    match client.tournament_full_view(&tournament_id).await {
        Ok(response) => {
            println!("{}", response.summary);
        }
        Err(e) => {
            eprintln!("Error fetching tournament full view: {:?}", e);
            std::process::exit(1);
        }
    }
}

async fn tournament_player_view(mock_server: bool, player_name: String, tournament_id: String) {
    let client = client(mock_server);
    let tournament_id = TournamentId(tournament_id);
    let player_name = PlayerName::new(&player_name);

    match client
        .tournament_player_view(&tournament_id, player_name)
        .await
    {
        Ok(response) => {
            println!("{}", response.summary);
        }
        Err(e) => {
            eprintln!("Error fetching tournament player view: {:?}", e);
            std::process::exit(1);
        }
    }
}

async fn run_series(mock_server: bool) {
    let client = client(mock_server);
    let series = SeriesSettings {
        // series_id: SeriesId::new("random-3p"),
        series_id: SeriesId::new("gpt5x3"),
        players: vec![
            // Player::random("Alice"),
            // Player::random("Bob"),
            // Player::random("Charlie"),
            Player::ai("AliceAI", "gpt-5", "Win the tournament."),
            Player::ai("BobAI", "gpt-5-mini", "Win the tournament."),
            Player::ai("CharlieAI", "gpt-5-nano", "Win the tournament."),
        ],
        number_of_tournaments: 50,
        starting_player_stack: 100.0,
        starting_small_blind: 5.0,
        double_blinds_every_n_games: Some(3),
        end_condition: TournamentEndCondition::SingleWinner,
        see_historical_thoughts: true,
        public_chat: false,
        random_seed: 42,
    };
    series_runner::run_series(&client, series).await;
}
