use liblife::{LifeBoard, LifeTilePos, LifeTileState};
use libmodel::{Model, runner::{ModelEvalData, ModelEvaluator, ModelEvalStride}};

pub struct HarnessMoveData {
    pub kind: HarnessMoveKind,
    pub decision_score: usize,
}

pub enum HarnessMoveKind {
    None,
    Flip(LifeTilePos),
}

pub struct ModelLifeHarness {
    pub model_evaluator: Box<dyn ModelEvaluator>,
    pub chunk_width: usize,
}

impl ModelLifeHarness {
    pub fn evaluate_move(&self, model: &Model, board: &impl LifeBoard) -> HarnessMoveData {
        let (tile_positions, tile_states): (Vec<_>, Vec<_>) = board.enumerate_tiles().unzip();
        let evaluated_tile_scores = self.eval_tile_scores(model, board, tile_states);

        let Some((max_score_index, max_score)) = evaluated_tile_scores
            .into_iter()
            .filter(|score| *score > 0)
            .enumerate()
            .max_by_key(|(_index, score)| *score)
        else {
            return HarnessMoveData {
                kind: HarnessMoveKind::None,
                decision_score: 0,
            };
        };
        let max_score_pos = tile_positions[max_score_index];

        HarnessMoveData {
            kind: HarnessMoveKind::Flip(max_score_pos),
            decision_score: max_score,
        }
    }

    fn eval_tile_scores(&self, model: &Model, board: &impl LifeBoard, tile_states: impl IntoIterator<Item = LifeTileState>) -> Vec<usize> {
        let tile_state_inputs: Vec<_> = tile_states
            .into_iter()
            .map(move |tile_state| match tile_state {
                // 0 reserved for OOB chunk tiles within the model evaluator.
                false => 1,
                true => 2,
            })
            .collect();

        let eval_data = ModelEvalData {
            input: &tile_state_inputs,
            stride: Some(ModelEvalStride {
                line_width: board.width() as usize,
                chunk_width: self.chunk_width,
            })
        };

        self.model_evaluator.evaluate_scores(model, eval_data)
    }
}
