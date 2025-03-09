use crate::State;

use super::Ticker;

pub struct MLTicker {}

impl Ticker for MLTicker {
    fn tick(&self, state: &mut State) {}
}
