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

pub struct TickerHost {
    stop_sender: Sender<()>,
}

impl TickerHost {
    pub fn start(
        state_arc: Arc<RwLock<State>>,
        interval: Duration,
        ticker: Box<dyn Ticker>,
    ) -> Self {
        let (stop_sender, stop_receiver) = mpsc::channel();

        thread::spawn(move || {
            while stop_receiver.try_recv().is_err() {
                let mut state = state_arc.write().unwrap();
                ticker.tick(&mut state);
                drop(state);

                thread::sleep(interval);
            }
        });

        Self { stop_sender }
    }

    pub fn stop(self) {
        self.stop_sender.send(()).unwrap();
    }
}

pub trait Ticker: Send {
    fn tick(&self, state: &mut State);
}
