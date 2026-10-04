use std::time::Duration;

use typeshare::typeshare;

#[typeshare]
pub const SERIAL_BOARD_CHUNK_LENGTH: u32 = 16;

#[typeshare]
pub type SerialBoardId = String;

#[typeshare]
pub type SerialBoardTileState = bool;

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SerialBoardInfo {
    pub title: String,
    /// Game rules in Golly-notation
    pub rules: String,
    pub width: u32,
    pub height: u32,
    pub tick_interval_millis: u32,
}
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SerialBoardInfoIdPair {
    pub id: SerialBoardId,
    pub info: SerialBoardInfo,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HttpVersionResponse {
    pub major: u32,
    pub minor: u32,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HttpBoardCreateRequest {
    pub board: SerialBoardInfo,
}
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HttpBoardCreateResponse {
    pub board_id: SerialBoardId,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HttpBoardListRequest {
    pub from_index: u32,
}
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HttpBoardListResponse {
    pub from_index: u32,
    pub has_more: bool,
    pub boards: Vec<SerialBoardInfoIdPair>,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HttpBoardWebsocketRequestQuery {
    pub board_id: SerialBoardId,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum WsServerbound {
    PullChunk(WsServerboundPullChunk),
    PushChunkTile(WsServerboundPushChunkTile),
    SetWatchedChunks(WsServerboundSetWatchedChunks),
}
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WsServerboundPullChunk {
    pub chunk_pos: [u32; 2],
}
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WsServerboundSetWatchedChunks {
    pub watched_chunk_positions: Vec<[u32; 2]>,
}
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WsServerboundPushChunkTile {
    pub chunk_pos: [u32; 2],
    pub chunk_tile_pos: [u32; 2],
    pub tile_state: SerialBoardTileState,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum WsClientbound {
    Chunk(WsClientboundChunk),
}
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WsClientboundChunk {
    pub chunk_pos: [u32; 2],
    pub chunk_width: u32,
    pub chunk_tile_states: Vec<SerialBoardTileState>,
}
