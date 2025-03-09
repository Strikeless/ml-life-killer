use libgame::{
    Game,
    board::{GameBoard, TileState},
    rule::Rule,
};
use libml::{
    game::{NetworkPlayer, NetworkPlayerConfig},
    network::Network,
};
use serde::{Deserialize, Serialize};

use super::{TrainerAdapter, TrainerAdapterFactory};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct GameTrainerAdapterConfig {
    pub width: usize,
    pub height: usize,
    pub alive_cells: usize,
    pub max_steps: usize,

    pub network_consecutive_turns: usize,
    pub game_consecutive_turns: usize,
}

pub struct GameTrainerAdapterFactory {
    pub config: GameTrainerAdapterConfig,
    pub player_config: NetworkPlayerConfig,
}

impl TrainerAdapterFactory<GameTrainerAdapter> for GameTrainerAdapterFactory {
    fn create_adapter(&self) -> GameTrainerAdapter {
        GameTrainerAdapter::new_randomized(self.config, self.player_config)
    }
}

pub struct GameTrainerAdapter {
    config: GameTrainerAdapterConfig,
    player_config: NetworkPlayerConfig,
    game_template: Game,
}

impl GameTrainerAdapter {
    pub fn new_randomized(
        config: GameTrainerAdapterConfig,
        player_config: NetworkPlayerConfig,
    ) -> Self {
        let game_template = Game::new(
            GameBoard::new_random(config.width, config.height, config.alive_cells),
            Rule::default(),
        );

        Self {
            config,
            player_config,
            game_template,
        }
    }
}

impl TrainerAdapter for GameTrainerAdapter {
    fn try_out(&self, network: &mut Network) -> f32 {
        let mut game = self.game_template.clone();
        let mut player = NetworkPlayer::new(self.player_config, network, &mut game);

        let initial_alive_cells = player.game.count_cells(TileState::Alive);
        let mut killed_cells: isize = 0;

        let mut steps_taken = 0;
        loop {
            steps_taken += 1;
            if steps_taken >= self.config.max_steps {
                break;
            }

            let made_move = player.play_step();

            if let Some(made_move) = made_move {
                match made_move.new_state {
                    TileState::Alive => killed_cells -= 1,
                    TileState::Dead => killed_cells += 1,
                }
            }

            let currently_alive_cells = if self.config.game_consecutive_turns == 0 {
                // Faster shortcut to getting currently alive cells, since only we change cell states.
                (initial_alive_cells as isize - killed_cells) as usize
            } else {
                player.game.count_cells(TileState::Alive)
            };

            if currently_alive_cells == 0 {
                break;
            }
        }

        killed_cells as f32 - steps_taken as f32
    }
}
