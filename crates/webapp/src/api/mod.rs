use axum::{
    Json, Router, body::Body, extract::{Query, State, WebSocketUpgrade}, http::Response, routing::{any, post},
};
use tower_http::trace::TraceLayer;

use crate::api::{
    error::ApiResult, state::{ApiState, board::{ApiStateBoardId, ApiStateBoardInfo}}, types::{HttpBoardCreateRequest, HttpBoardCreateResponse, HttpBoardListRequest, HttpBoardListResponse, HttpBoardWebsocketRequestQuery, HttpVersionResponse, SerialBoardInfoIdPair}, ws::BoardWebSocketHandler,
};

mod types;
mod error;
mod state;
mod ws;

pub fn api_router() -> anyhow::Result<Router> {
    let state = ApiState::new()?;

    let router = Router::new()
        .route("/version", any(version))
        .route("/board/list", any(board_list))
        .route("/board/create", post(board_create))
        .route("/board/ws", any(board_ws))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    Ok(router)
}

async fn version() -> Json<HttpVersionResponse> {
    Json(HttpVersionResponse {
        major: env!("CARGO_PKG_VERSION_MAJOR").parse().expect("CARGO_PKG_VERSION_MAJOR not parseable"),
        minor: env!("CARGO_PKG_VERSION_MINOR").parse().expect("CARGO_PKG_VERSION_MINOR not parseable"),
    })
}

async fn board_create(
    State(state): State<ApiState>,
    Json(request): Json<HttpBoardCreateRequest>,
) -> ApiResult<Json<HttpBoardCreateResponse>> {
    // TODO: Restrict board properties like dimensions and tick interval.
    let board_info = ApiStateBoardInfo::from_serial(request.board)?;
    let board_id = state.create_board(board_info).await?;

    Ok(Json(HttpBoardCreateResponse {
        board_id: board_id.into(),
    }))
}

async fn board_list(
    State(state): State<ApiState>,
    Json(request): Json<HttpBoardListRequest>,
) -> Json<HttpBoardListResponse> {
    let board_manager = state.board_manager.read().await;

    let mut board_infos_iter = board_manager.board_handles
        .iter()
        .map(|(board_id, board_handle)| SerialBoardInfoIdPair {
            id: board_id.to_owned().into(),
            info: board_handle.info.to_serial(),
        })
        .skip(request.from_index as usize);

    let boards = board_infos_iter.by_ref().take(100).collect();
    let has_more = board_infos_iter.next().is_some();

    Json(HttpBoardListResponse {
        from_index: request.from_index,
        has_more,
        boards,
    })
}

async fn board_ws(
    State(state): State<ApiState>,
    Query(query): Query<HttpBoardWebsocketRequestQuery>,
    ws_upgrade: WebSocketUpgrade,
) -> ApiResult<Response<Body>> {
    let board_id: ApiStateBoardId = ApiStateBoardId::try_parse(&query.board_id)?;

    let board_handle = {
        let board_manager = state.board_manager.read().await;
        board_manager.clone_board_handle(&board_id).await?
    };

    let ws_upgrade_response = ws_upgrade
        .on_upgrade(move |ws| {
            let handler = BoardWebSocketHandler::new(ws, board_id, board_handle);
            handler.handle()
        });

    Ok(ws_upgrade_response)
}
