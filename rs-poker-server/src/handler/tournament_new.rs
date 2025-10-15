use axum::{extract::State, Json};
use rs_poker_types::tournament::{TournamentId, TournamentSettings};

use crate::{
    define_handler, error::ServerError, handler::HandlerResponse, poker_server::ServerState,
};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub struct TournamentCreatedResponse {
    pub tournament_id: TournamentId,
}

async fn new_tournament_handler(
    State(state): State<ServerState>,
    Json(settings): Json<TournamentSettings>,
) -> HandlerResponse<TournamentCreatedResponse> {
    let server = state.engine.lock().unwrap();
    let tournament_id = settings.tournament_id.clone();

    // Fail if the tournament ID already exists.
    if server.tournaments.contains_key(&tournament_id) {
        return Json(Err(ServerError::TournamentAlreadyExists(tournament_id)));
    }

    // Create a new tournament instance.
    // let tournament = TournamentInstance::new(&settings);

    // // Store it.
    // server.update_tournament(&tournament);

    // // Start playing.
    // server.progress_tournament(&tournament_id);

    Json(Ok(TournamentCreatedResponse {
        tournament_id: settings.tournament_id.clone(),
    }))
}

define_handler!(
    NewTournamentHandler {
        Request = TournamentSettings;
        Response = TournamentCreatedResponse;
        Method = POST;
        Path = "/new_tournament";
        FN = new_tournament_handler;
    }
);

//     pub fn progress_tournament(&mut self, tournament_id: &TournamentId) {
//         if let Some(mut tournament) = self.tournament(tournament_id) {
//             while let Some(action) = tournament.next_action() {
//                 match action {
//                     TournamentAction::StartNextGame { game_settings } => {
//                         // Create and store a new game instance.
//                         let mut game =
//                             GameInstance::new(game_settings);
//                         game.run();

//                         if !game.is_complete() {
//                             // Store the game.
//                             self.update_game(&game);

//                             // Update tournament and break if the game could
// not be completed.                             // This means it is waiting for
// player input.
// self.update_tournament(&tournament);                             break;
//                         }

//                         // If the game is complete, load the final results.
//                         let game_result = game.game_final_results().unwrap();

//                         // Also store the game.
//                         self.update_game(&game);

//                         // Finish the game in the tournament.
//                         tournament.finish_game(&game_result).unwrap();
//                         // Continue to the next action.
//                     }
//                     TournamentAction::FinishGame { game_id } => {
//                         // This state means the game has already been
// started,                         // and needs to be pushed to completion.
//                         if let Some(mut game) = self.game(&game_id) {
//                             if !game.is_complete() {
//                                 game.run();
//                                 if game.is_complete() {
//                                     let game_result =
// game.game_final_results().unwrap();
// tournament.finish_game(&game_result).unwrap();
// self.update_game(&game);                                 } else {
//                                     self.update_game(&game);
//                                     self.update_tournament(&tournament);
//                                     // Game is still not complete, break to
// wait for player input.                                     break;
//                                 }
//                             } else {
//                                 // Game is already complete, just finish it
// in the tournament.                                 let game_result =
// game.game_final_results().unwrap();
// tournament.finish_game(&game_result).unwrap();                             }
//                         } else {
//                             // Game not found, this is an error in the
// tournament state.
// self.update_tournament(&tournament);                             break;
//                         }
//                     }
//                 }
//             }
//             // Update tournament after all actions are processed
//             self.update_tournament(&tournament);
//         }
//     }
// }
