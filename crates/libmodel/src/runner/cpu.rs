use crate::{ModelActivationFunction, ModelCell, ModelChunkSize, ModelConfiguration, ModelPatternList, ModelPatternNode, runner::{ModelEvalStride, ModelEvaluator}};

pub struct CpuModelRunner;

impl ModelEvaluator for CpuModelRunner {
    fn evaluate_scores(&self, model: &crate::Model, data: super::ModelEvalData<'_>) -> Vec<usize> {
        let data_chunks = data.input
            .windows(model.configuration.chunk_length)
            .map(|chunk_offset_data| DataChunk(chunk_offset_data));

        data_chunks
            .map(|chunk| self.chunk_score_pattern_list(&model.configuration, &data.stride, &chunk, &model.pattern_list))
            .collect()
    }
}

impl CpuModelRunner {
    fn chunk_score_pattern_list(
        &self,
        model_configuration: &ModelConfiguration,
        stride: &Option<ModelEvalStride>,
        chunk: &DataChunk,
        pattern_list: &ModelPatternList,
    ) -> usize {
        pattern_list.patterns.iter()
            .map(|pattern| self.chunk_score_pattern_recursive(model_configuration, stride, chunk, pattern))
            .sum()
    }

    fn chunk_score_pattern_recursive(
        &self,
        model_configuration: &ModelConfiguration,
        stride: &Option<ModelEvalStride>,
        chunk: &DataChunk,
        pattern: &ModelPatternNode,
    ) -> usize {
        if !self.chunk_score_pattern_only(model_configuration, stride, chunk, pattern) {
            return 0;
        }

        let sub_patterns_score: usize = pattern.sub_patterns.iter()
            .map(|sub_pattern| self.chunk_score_pattern_recursive(model_configuration, stride, chunk, sub_pattern))
            .sum();

        1 + sub_patterns_score
    }

    fn chunk_score_pattern_only(
        &self,
        model_configuration: &ModelConfiguration,
        stride: &Option<ModelEvalStride>,
        chunk: &DataChunk,
        pattern: &ModelPatternNode,
    ) -> bool {
        debug_assert_eq!(pattern.chunk_activation_values.len(), model_configuration.chunk_length);

        let cell_values = chunk.iter(stride);
        let activation_values = pattern.chunk_activation_values.iter().copied();

        cell_values.zip(activation_values)
            .all(|(cell_value, activation_value)| Self::apply_activation_function(&model_configuration.activation, cell_value, activation_value))
    }

    fn apply_activation_function(activation_function: &ModelActivationFunction, cell_value: ModelCell, activation_value: ModelCell) -> bool {
        match activation_function {
            ModelActivationFunction::Equal => activation_value == cell_value,
            ModelActivationFunction::Greater => activation_value > cell_value,
            ModelActivationFunction::Lesser => activation_value < cell_value,
            ModelActivationFunction::ZeroRem => cell_value != 0 && (activation_value % cell_value) == 0,
            ModelActivationFunction::NonZeroRem => cell_value != 0 && (activation_value % cell_value) != 0,
        }
    }
}

struct DataChunk<'a>(&'a [ModelCell]);

impl DataChunk<'_> {
    pub fn get(&self, stride: &Option<ModelEvalStride>, index: ModelChunkSize) -> Option<ModelCell> {
        let index = index as usize;

        match stride {
            None => self.0.get(index).copied(),
            Some(stride) => {
                let access_index = {
                    let row_offset = index / stride.chunk_width * stride.line_width;
                    let column_offset = index % stride.chunk_width;
                    row_offset + column_offset
                };

                self.0.get(access_index).copied()
            },
        }
    }

    // TODO: Maybe edit to manually specify chunk size and then return a distinct value for OOB tiles? This way the model can learn patterns for edges too.
    pub fn iter(&self, stride: &Option<ModelEvalStride>) -> impl Iterator<Item = ModelCell> {
        (0..).map_while(|index| self.get(stride, index as ModelChunkSize))
    }
}
