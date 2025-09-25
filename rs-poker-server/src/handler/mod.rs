use axum::{routing::MethodRouter, Json};
use reqwest::Method;

use crate::{error::ServerError, poker_server::ServerState};

pub mod health_check;
pub mod game_new;
pub mod game_list;
pub mod game_full_view;
pub mod game_player_view;
pub mod game_info;
pub mod game_make_action;
pub mod new_tournament;

pub type HandlerResponse<T> = Json<Result<T, ServerError>>;

pub trait Handler {
    type Request: serde::de::DeserializeOwned + serde::Serialize;
    type Response: serde::de::DeserializeOwned + serde::Serialize;
    fn router() -> MethodRouter<ServerState>;
    fn method() -> Method;
    fn path() -> &'static str;
}

#[macro_export]
macro_rules! define_handler {
    (
        $handler_name:ident {
            Request = $request:ty;
            Response = $response:ty;
            Method = $method:ident;
            Path = $path:expr;
            FN = $handler_fn:ident;
        }
    ) => {
        pub struct $handler_name;

        impl crate::handler::Handler for $handler_name {
            type Request = $request;
            type Response = $response;

            fn method() -> reqwest::Method {
                reqwest::Method::$method
            }

            fn path() -> &'static str {
                $path
            }
            
            fn router() -> axum::routing::MethodRouter<crate::poker_server::ServerState> {
                match stringify!($method) {
                    "GET" => axum::routing::get($handler_fn),
                    "POST" => axum::routing::post($handler_fn),
                    "PUT" => axum::routing::put($handler_fn),
                    "DELETE" => axum::routing::delete($handler_fn),
                    "PATCH" => axum::routing::patch($handler_fn),
                    _ => panic!("Unsupported HTTP method: {}", stringify!($method)),
                }
            }
        }
    };
}