use clap::{Parser, Subcommand};
use rs_poker_cli::cli::{client, run_example_game};
use rs_poker_server::handler::{game_full_view::GameFullViewRequest, game_list::ListGamesRequest};
use rs_poker_types::game::GameId;

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
    /// Start a new poker game
    Play,
    /// List all games
    ListGames {
        /// Show only active games
        #[arg(short, long)]
        active_only: bool,
    },
    /// Show full view of a specific game
    ShowGame {
        /// Game ID to show
        game_id: String,
        /// Show debug information
        #[arg(short, long)]
        debug: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Play => {
            run_example_game(cli.mock_server).await;
        }
        Commands::ListGames { active_only } => {
            list_games(cli.mock_server, active_only).await;
        }
        Commands::ShowGame { game_id, debug } => {
            show_game(cli.mock_server, game_id, debug).await;
        }
    }
}

async fn show_game(mock_server: bool, game_id: String, debug: bool) {
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
