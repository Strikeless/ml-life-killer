use libml::{
    game::{NetworkPlayer, NetworkPlayerConfig},
    network::Network,
};
use ouroboros::self_referencing;

use crate::State;

use super::Ticker;

pub struct MLTicker {
    network_and_player: NetworkAndPlayer,
}

impl MLTicker {
    pub fn new(network: Network, config: NetworkPlayerConfig) -> Self {
        let network_and_player = NetworkAndPlayerBuilder {
            network,
            player_builder: |network| NetworkPlayer::new(config, network),
        }
        .build();

        Self { network_and_player }
    }
}

impl Ticker for MLTicker {
    fn tick(&mut self, state: &mut State) {
        self.network_and_player.with_player_mut(|network_player| {
            network_player.play_step(&mut state.game);
        });
    }
}

#[self_referencing]
struct NetworkAndPlayer {
    network: Network,

    #[borrows(mut network)]
    #[not_covariant] // It isn't safe to use NetworkPlayer with a lifetime smaller than 'this.
    pub player: NetworkPlayer<'this>,
}
