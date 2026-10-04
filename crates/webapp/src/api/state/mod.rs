use std::{fmt::Debug, sync::Arc};

use tokio::sync::{Mutex, RwLock};
use tracing::debug;

use crate::api::state::{board::{ApiStateBoardId, ApiStateBoardInfo, ApiStateBoardManager}, ticker::ApiStateTickerManager};

pub mod board;
pub mod ticker;

#[derive(Clone)]
pub struct ApiState {
    pub board_manager: Arc<RwLock<ApiStateBoardManager>>,
    pub ticker_manager: Arc<Mutex<ApiStateTickerManager>>,
}

impl Debug for ApiState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiState {{ ... }}")
    }
}

impl ApiState {
    pub fn new() -> anyhow::Result<Self> {
        let board_manager = ApiStateBoardManager::new();

        let ticker_manager = ApiStateTickerManager::new();
        ticker_manager.spawn_ticker()?;

        Ok(Self {
            board_manager: Arc::new(RwLock::new(board_manager)),
            ticker_manager: Arc::new(Mutex::new(ticker_manager)),
        })
    }

    pub async fn create_board(&self, board_info: ApiStateBoardInfo) -> anyhow::Result<ApiStateBoardId> {
        let (board_id, board_handle) = {
            let mut board_manager = self.board_manager.write().await;
            let board_id = board_manager.create_board(board_info.clone()).await?;
            let board_handle = board_manager.clone_board_handle(&board_id).await?;
            (board_id, board_handle)
        };

        {
            let ticker_manager = self.ticker_manager.lock().await;
            ticker_manager.register_board(board_id, board_handle).await;
        }

        debug!(id = ?board_id, info = ?board_info, "Created new board");
        Ok(board_id)
    }

    pub async fn delete_board(&self, board_id: ApiStateBoardId) -> anyhow::Result<()> {
        {
            let mut board_manager = self.board_manager.write().await;
            board_manager.forget_board(&board_id).await?;
        }

        {
            let ticker_manager = self.ticker_manager.lock().await;
            ticker_manager.unregister_board(board_id).await;
        }

        // TODO: Close remaining board websocket handlers

        debug!(id = ?board_id, "Deleted board");
        Ok(())
    }
}
