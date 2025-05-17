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

    /// The amount of alive cells to spawn at the start of a game.
    pub alive_cells: usize,

    /// The size of a single "block" of cells to spawn, a value of one will result in completely random spawning,
    /// while a value of two will result in 2x2 clusters of alive cells. The only reason this is implemented is
    /// because the networks forgot how to kill 2x2 blocks after a while of training (lol).
    /// NOTE: alive_cells still refers to "cells" instead of "blocks", but will round down to a block cell count.
    pub block_size: usize,

    /// The maximum number of rounds a single game will be played for in one iteration.
    /// If all cells are dead earlier, the game will stop early.
    pub max_rounds: usize,

    /// Disable any "natural" progression of the game completely,
    /// leaving only the network to make changes to the board, this may be useful in the beginning of training.
    pub disable_nature: bool,

    /// Whether to reward for cells killed or cells brought to life.
    pub evil: bool,
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
            GameBoard::new_random(config.width, config.height, config.alive_cells, config.block_size),
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

            if self.config.evil {
                initial_cells_alive as isize - finished_cells_alive as isize
            } else {
                finished_cells_alive as isize - initial_cells_alive as isize
            }
        }
    }

    fn get_network_score(&self, mut game: Game, network: &mut Network) -> (isize, isize) {
        let mut network_player = NetworkPlayer::new(self.player_config, network);

        let initial_cells_alive = game.count_cells(TileState::Alive);
        let mut skipped_turns = 0;

        let mut rounds_taken = 0;
        let finished_cells_alive = loop {
            rounds_taken += 1;

            let network_move = network_player.play_step(&mut game);
            if network_move.is_none() {
                skipped_turns += 1;
            }

            if !self.config.disable_nature {
                game.tick();
            }

            let alive_cells = game.count_cells(TileState::Alive);
            if rounds_taken >= self.config.max_rounds || alive_cells == 0 {
                break alive_cells;
            }
        };

        let cells_turned_reward = if self.config.evil {
            initial_cells_alive as isize - finished_cells_alive as isize
        } else {
            finished_cells_alive as isize - initial_cells_alive as isize
        };

        // NOTE: It's quite essential that the taken rounds punishment is divided, since otherwise
        //       we would cancel out all the reward out of directly killed cells, which doesn't work out.
        let taken_rounds_punishment = (self.config.max_rounds - rounds_taken) as isize / 2;
        
        // Also punish for many skipped turns, this may not be totally "correct" in every circumstance,
        // but will generally be a sign of bad behavior at the current state of the networks.
        let skipped_turns_punishment = skipped_turns / 5;
        
        let punishment = taken_rounds_punishment + skipped_turns_punishment;
        (cells_turned_reward, punishment)
    }
}

impl TrainerAdapter for GameTrainerAdapter {
    fn try_out(&self, network: &mut Network) -> isize {
        let game = self.game_template.clone();

        let reference_score = self.get_reference_score(&game);
        let (network_score, network_punishment) = self.get_network_score(game, network);

        // NOTE: I'd imagine it's useful to have the actual performance based score separated from all the
        //       "artificial" punishments (taken steps etc), as otherwise the comparison to the reference
        //       score isn't all that fair (if I'm thinking this correctly).
        let reward = network_score - reference_score;
        reward - network_punishment
    }
}
