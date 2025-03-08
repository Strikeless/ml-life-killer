use libgame::{Game, board::GameBoard, rule::Rule};
use player::NetworkPlayer;
use serde::{Deserialize, Serialize};

use crate::{
    network::Network,
    trainer::adapter::{TrainerAdapter, TrainerAdapterFactory},
};

mod kernel;
mod player;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct GameTrainerAdapterConfig {
    pub width: usize,
    pub height: usize,
    pub alive_cells: usize,
    pub max_steps: usize,

    pub network_consecutive_turns: usize,
    pub game_consecutive_turns: usize,
    pub kernel_diameter: usize,
}

pub struct GameTrainerAdapterFactory {
    pub config: GameTrainerAdapterConfig,
}

impl TrainerAdapterFactory<GameTrainerAdapter> for GameTrainerAdapterFactory {
    fn create_adapter(&self) -> GameTrainerAdapter {
        GameTrainerAdapter::new_randomized(self.config)
    }
}

pub struct GameTrainerAdapter {
    config: GameTrainerAdapterConfig,
    game_template: Game,
}

impl GameTrainerAdapter {
    pub fn new_randomized(config: GameTrainerAdapterConfig) -> Self {
        let game_template = Game::new(
            GameBoard::new_random(config.width, config.height, config.alive_cells),
            Rule::default(),
        );

        Self {
            config,
            game_template,
        }
    }
}

impl TrainerAdapter for GameTrainerAdapter {
    fn try_out(&self, network: &mut Network) -> f32 {
        let mut player = NetworkPlayer::new(network, &self.game_template, self.config);

        let initial_alive_cells = player.count_alive_cells();

        for step in 0..self.config.max_steps {
            player.play_step();

            if player.count_alive_cells() == 0 {
                // Task accomplished, reward the killed cell count plus least steps taken.
                let taken_steps_bonus = self.config.max_steps - step;
                return (initial_alive_cells + taken_steps_bonus) as f32;
            }
        }

        // Task wasn't accomplished in step limit, punish the least cells killed.
        let end_alive_cells = player.count_alive_cells();
        -100.0 + initial_alive_cells as f32 - end_alive_cells as f32
    }
}
