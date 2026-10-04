use crate::Model;

pub mod cpu;

pub struct ModelEvalData<'a> {
    pub input: &'a [u8],
    pub stride: Option<ModelEvalStride>,
}

pub struct ModelEvalStride {
    /// The length of consecutive chunk data slices.
    pub chunk_width: usize,
    /// The width of a single "line", meaning the distance between the beginnings of consecutive chunk data slices.
    pub line_width: usize,
}

pub trait ModelEvaluator {
    fn evaluate_scores(&self, model: &Model, data: ModelEvalData<'_>) -> Vec<usize>;
}
