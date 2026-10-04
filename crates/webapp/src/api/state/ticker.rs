use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Context;
use liblife::LifeEngine;
use tokio::{sync::RwLock, time::Instant};

use crate::api::state::board::{ApiStateBoardHandle, ApiStateBoardId};

/// The maximum latency from when a new board tick is scheduled to when the scheduler first gives it a chance to tick.
/// This does not affect the maximum tick rate of boards that have already been scheduled and ticked once.
const SCHEDULER_INACTIVE_CYCLE_INTERVAL: Duration = Duration::from_millis(100);

pub struct ApiStateTickerManager {
    ticked_board_handles_arc: Arc<RwLock<HashMap<ApiStateBoardId, ApiStateBoardHandle>>>,
}

impl ApiStateTickerManager {
    pub fn new() -> Self {
        Self {
            ticked_board_handles_arc: Arc::default(),
        }
    }

    pub async fn register_board(&self, board_id: ApiStateBoardId, board_handle: ApiStateBoardHandle) {
        let mut ticked_board_handles = self.ticked_board_handles_arc.write().await;
        ticked_board_handles.insert(board_id, board_handle);
    }

    pub async fn unregister_board(&self, board_id: ApiStateBoardId) {
        let mut ticked_board_handles = self.ticked_board_handles_arc.write().await;
        ticked_board_handles.remove(&board_id);
    }

    pub fn spawn_ticker(&self) -> anyhow::Result<()> {
        let ticker = ApiStateTicker::new(self.ticked_board_handles_arc.clone());

        let run_ticker_on_new_runtime = move || {
            let tokio_runtime = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .expect("Creating dedicated tokio runtime for board ticker");

            tokio_runtime
                .block_on(ticker.run())
                .expect("Running board ticker");
        };

        let _join_handle = std::thread::Builder::new()
            .name("board-ticker-runtime".to_owned())
            .spawn(run_ticker_on_new_runtime)
            .context("Spawning dedicated thread for board ticker runtime")?;

        Ok(())
    }
}

// TODO: This doesn't scale. We'll likely want to have the tick scheduler run asynchronous of actual ticking, with life engines running on a dedicated thread pool.
//       This way we could scale by multithreading, ticking many boards simultaneously on different CPU threads.
struct ApiStateTicker {
    board_handles_arc: Arc<RwLock<HashMap<ApiStateBoardId, ApiStateBoardHandle>>>,
    board_ticker_data_map: HashMap<ApiStateBoardId, ApiStateTickerBoardData>,
}
impl ApiStateTicker {
    pub fn new(board_handles_arc: Arc<RwLock<HashMap<ApiStateBoardId, ApiStateBoardHandle>>>) -> Self {
        Self {
            board_handles_arc,
            board_ticker_data_map: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            // Run scheduler ticks until there are no more boards to tick right away.
            loop {
                let scheduler_state = self.run_scheduler_cycle().await?;

                if scheduler_state == SchedulerState::Busy {
                    // The scheduler has more work to do right away, run another cycle immediately.
                    continue;
                }

                // Some previously scheduled boards might have been unscheduled. Purge ticker data for any such boards.
                self.purge_unused_board_ticker_data().await;

                match scheduler_state {
                    // The scheduler has work scheduled for the future, which it's ready to run after the given duration.
                    // Wait until it's ready to run, OR enough time has passed to warrant checking for newly scheduled (possibly sooner!) work.
                    SchedulerState::ActiveWaiting(duration_till_next_cycle) => tokio::time::sleep(duration_till_next_cycle.min(SCHEDULER_INACTIVE_CYCLE_INTERVAL)).await,
                    // The scheduler has nothing scheduled currently.
                    // Wait for a while before running the next cycle to check for newly scheduled work.
                    SchedulerState::InactiveWaiting => tokio::time::sleep(SCHEDULER_INACTIVE_CYCLE_INTERVAL).await,
                    // Already handled above, before purging board ticker data.
                    SchedulerState::Busy => unreachable!(),
                }
            }
        }
    }

    async fn run_scheduler_cycle(&mut self) -> anyhow::Result<SchedulerState> {
        let board_handles = self.board_handles_arc.read().await;

        let board_ticker_datas = board_handles.iter()
            .map(|(board_id, board_handle)| {
                // Get or initialize ticker data for this board
                let board_ticker_data = self.board_ticker_data_map
                    .entry(*board_id)
                    .or_insert_with(move || ApiStateTickerBoardData::new(board_handle));

                // Need to copy ticker data, since the Rust compiler can't prove that we're not mutating it in further iterations.
                (board_id, board_handle, *board_ticker_data)
            });

        let Some((earliest_ticking_board_id, earliest_ticking_board_handle, earliest_ticking_data)) = board_ticker_datas
            .min_by_key(|(_, _, board_ticker_data)| board_ticker_data.next_tick_instant)
        else {
            // Not a single board is scheduled to tick, we can wait till one is.
            return Ok(SchedulerState::InactiveWaiting);
        };

        let instant_now = Instant::now();

        if instant_now >= earliest_ticking_data.next_tick_instant {
            /*
             * This scheduled board tick is ready to be run, so run it.
             */
            self.run_board_tick(earliest_ticking_board_handle).await?;

            // Reschedule the board for the next tick. For this, we first need a mutable borrow of the ticker data.
            let earliest_ticking_data = self.board_ticker_data_map.get_mut(earliest_ticking_board_id).unwrap();
            earliest_ticking_data.schedule_next_tick(earliest_ticking_board_handle);

            // Done with this tick, run another scheduler cycle to check if more ticks are ready to run.
            Ok(SchedulerState::Busy)
        } else {
            /*
             * This (earliest) scheduled board tick is not yet ready to run.
             * We can wait until it is ready or an earlier board tick gets scheduled before it.
             */
            let duration_till_earliest_tick = earliest_ticking_data.next_tick_instant - instant_now;
            Ok(SchedulerState::ActiveWaiting(duration_till_earliest_tick))
        }
    }

    async fn run_board_tick(&self, board_handle: &ApiStateBoardHandle) -> anyhow::Result<()> {
        let mut board = board_handle.board.write().await;

        // NOTE: Computationally heavy! The scheduler must be run on a thread where continuous blocking is acceptable!
        board_handle.engine.tick(&mut *board);

        Ok(())
    }

    async fn board_ticker_data(&mut self, board_id: ApiStateBoardId, board_handle: &ApiStateBoardHandle) -> &ApiStateTickerBoardData {
        self.board_ticker_data_mut(board_id, board_handle).await
    }

    async fn board_ticker_data_mut(&mut self, board_id: ApiStateBoardId, board_handle: &ApiStateBoardHandle) -> &mut ApiStateTickerBoardData {
        self.board_ticker_data_map
            .entry(board_id)
            .or_insert_with(|| ApiStateTickerBoardData::new(board_handle))
    }

    async fn purge_unused_board_ticker_data(&mut self) {
        let board_handles = self.board_handles_arc.read().await;
        self.board_ticker_data_map.retain(|board_id, _| board_handles.contains_key(board_id));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ApiStateTickerBoardData {
    pub next_tick_instant: Instant,
}
impl ApiStateTickerBoardData {
    pub fn new(board_handle: &ApiStateBoardHandle) -> Self {
        let next_tick_instant = Instant::now() + board_handle.info.tick_interval;
        Self { next_tick_instant }
    }

    pub fn schedule_next_tick(&mut self, board_handle: &ApiStateBoardHandle) {
        self.next_tick_instant = Instant::now() + board_handle.info.tick_interval;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SchedulerState {
    /// Scheduler wants to run a cycle immediately to check whether it has more work it can do.
    Busy,
    /// Scheduler currently has a board tick scheduled for the future, which it wants to run once the given duration has passed.
    ActiveWaiting(Duration),
    /// Scheduler currently has nothing scheduled, and is waiting for something to be scheduled.
    InactiveWaiting,
}
