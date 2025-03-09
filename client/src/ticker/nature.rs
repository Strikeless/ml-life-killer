use crate::State;

use super::Ticker;

pub struct NatureTicker;

impl Ticker for NatureTicker {
    fn tick(&self, state: &mut State) {
        state.game.tick();
    }
}
