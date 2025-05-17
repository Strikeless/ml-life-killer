use crate::State;
use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};

pub mod ml;
pub mod nature;

pub trait TickerGenerator = FnOnce() -> Box<dyn Ticker> + Send;

pub struct TickerHost {
    stop_sender: Sender<()>,
    rate_sender: Sender<u64>,
}

impl TickerHost {
    pub fn start(
        state_arc: Arc<RwLock<State>>,
        mut interval: Duration,
        ticker_generator: Box<dyn TickerGenerator>,
    ) -> Self {
        let (stop_sender, stop_receiver) = mpsc::channel();
        let (rate_sender, rate_receiver) = mpsc::channel();

        thread::spawn(move || {
            let mut ticker = ticker_generator();

            while stop_receiver.try_recv().is_err() {
                let mut state = state_arc.write().unwrap();
                ticker.tick(&mut state);
                drop(state);

                if let Ok(rate) = rate_receiver.try_recv() {
                    interval = Duration::from_millis(rate);
                }

                thread::sleep(interval);
            }
        });

        Self { stop_sender, rate_sender }
    }

    pub fn stop(self) {
        self.stop_sender.send(()).unwrap();
    }

    pub fn set_rate(&mut self, rate_millis: u64) {
        self.rate_sender.send(rate_millis).unwrap();
    }
}

pub trait Ticker {
    fn tick(&mut self, state: &mut State);
}
