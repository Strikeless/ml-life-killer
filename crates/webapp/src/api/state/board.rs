use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Context;
use liblife::{LifeRules, boards::SimpleHeapLifeBoard, engines::SimpleLifeEngine};
use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

use crate::api::types::SerialBoardInfo;

pub type ApiStateBoardId = Uuid;

pub type ApiLifeBoardImplementation = SimpleHeapLifeBoard;
pub type ApiLifeEngineImplementation = SimpleLifeEngine;

pub struct ApiStateBoardManager {
    pub board_handles: HashMap<ApiStateBoardId, ApiStateBoardHandle>,
    engine_cache: HashMap<LifeRules, Arc<ApiLifeEngineImplementation>>,
}

#[derive(Clone)]
pub struct ApiStateBoardHandle {
    pub info: Arc<ApiStateBoardInfo>,
    pub board: Arc<RwLock<ApiLifeBoardImplementation>>,
    pub engine: Arc<ApiLifeEngineImplementation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiStateBoardInfo {
    pub title: String,
    pub rules: LifeRules,
    pub width: u32,
    pub height: u32,
    pub tick_interval: Duration,
}

impl ApiStateBoardManager {
    pub fn new() -> Self {
        Self {
            board_handles: HashMap::new(),
            engine_cache: HashMap::new(),
        }
    }

    pub async fn create_board(&mut self, info: ApiStateBoardInfo) -> anyhow::Result<ApiStateBoardId> {
        let board_id = self.next_board_id();

        let board = self.init_board(info.width, info.height).await?;
        let engine = self.get_or_init_cached_engine(info.rules).await?;

        self.board_handles.insert(
            board_id,
            ApiStateBoardHandle {
                info: Arc::new(info),
                board: Arc::new(RwLock::new(board)),
                engine,
            },
        );

        Ok(board_id)
    }

    pub async fn forget_board(&mut self, board_id: &ApiStateBoardId) -> anyhow::Result<()> {
        self.board_handles.remove(board_id).context("No board with given id to remove")?;
        Ok(())
    }

    pub async fn clone_board_handle(&self, board_id: &ApiStateBoardId) -> anyhow::Result<ApiStateBoardHandle> {
        self.board_handles.get(board_id)
            .cloned()
            .context("No board with given id")
    }

    /// Drops cached but unused life engines.
    /// This can free a good chunk of memory if many life engines have been created, depending on the specific engine implementation.
    pub async fn purge_engine_cache(&mut self) {
        self.engine_cache.retain(|_chunk_manager_rules, chunk_manager_arc| !Arc::is_unique(chunk_manager_arc));
    }

    fn next_board_id(&self) -> ApiStateBoardId {
        Uuid::new_v4()
    }

    async fn init_board(&mut self, width: u32, height: u32) -> anyhow::Result<ApiLifeBoardImplementation> {
        tokio::task::spawn_blocking(move || ApiLifeBoardImplementation::new(width, height))
            .await
            .context("Initializing life board on blocking thread")
    }

    async fn get_or_init_cached_engine(&mut self, rules: LifeRules) -> anyhow::Result<Arc<ApiLifeEngineImplementation>> {
        if let Some(cached_engine_arc) = self.engine_cache.get(&rules) {
            return Ok(cached_engine_arc.clone());
        }

        debug!("Initializing new life engine for rules {}", rules.to_golly());
        let initialized_engine = {
            let rules = rules.clone();

            tokio::task::spawn_blocking(move || ApiLifeEngineImplementation::new(rules))
                .await
                .context("Initializing life engine on blocking thread")?
        };

        // It's possible that while we were initializing the engine for this ruleset, one already got initialized and cached by another future.
        // In such cases, we should use the engine that already got cached, dropping the one we just initialized as wasted work.
        let cached_engine_arc = self.engine_cache
            .entry(rules)
            .or_insert_with(|| Arc::new(initialized_engine));

        Ok(cached_engine_arc.clone())
    }
}

impl ApiStateBoardInfo {
    pub fn from_serial(serial: SerialBoardInfo) -> anyhow::Result<Self> {
        Ok(Self {
            rules: LifeRules::parse_golly(&serial.rules)?,
            title: serial.title,
            width: serial.width,
            height: serial.height,
            tick_interval: Duration::from_millis(serial.tick_interval_millis as u64),
        })
    }

    pub fn to_serial(&self) -> SerialBoardInfo {
        SerialBoardInfo {
            rules: self.rules.to_golly(),
            title: self.title.clone(),
            width: self.width,
            height: self.height,
            tick_interval_millis: self.tick_interval.as_millis() as u32,
        }
    }
}
