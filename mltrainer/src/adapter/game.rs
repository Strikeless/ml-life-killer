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
    pub max_rounds: usize,

    /// Disable any "natural" progression of the game completely,
    /// leaving only the network to make changes to the board, this may be useful in the beginning of training.
    pub disable_nature: bool,
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

    fn get_reference_score(&self, game: &Game) -> isize {
        if self.config.disable_nature {
            0
        } else {
            let mut game = game.clone();
            let initial_cells_alive = game.count_cells(TileState::Alive);

            for _ in 0..self.config.max_rounds {
                game.tick();
            }

            let finished_cells_alive = game.count_cells(TileState::Alive);
            initial_cells_alive as isize - finished_cells_alive as isize
        }
    }

    fn get_network_score(&self, mut game: Game, network: &mut Network) -> isize {
        let mut network_player = NetworkPlayer::new(
            self.player_config,
            network,
        );

        let initial_cells_alive = game.count_cells(TileState::Alive);

        let mut rounds_taken = 0;
        let finished_cells_alive = loop {
            rounds_taken += 1;

            network_player.play_step(&mut game);

            if !self.config.disable_nature {
                game.tick();
            }

            let alive_cells = game.count_cells(TileState::Alive);
            if rounds_taken >= self.config.max_rounds || alive_cells == 0 {
                break alive_cells;
            }
        };

        let cells_killed_reward = initial_cells_alive as isize - finished_cells_alive as isize;
        let taken_rounds_punishment = (self.config.max_rounds - rounds_taken) as isize;

        cells_killed_reward - taken_rounds_punishment
    }
}

impl TrainerAdapter for GameTrainerAdapter {
    fn try_out(&self, network: &mut Network) -> isize {
        let game = self.game_template.clone();

        let reference_score = self.get_reference_score(&game);
        let network_score = self.get_network_score(game, network);

        network_score - reference_score
    }
}
