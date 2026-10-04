pub mod runner;

pub type ModelCell = u8;
pub type ModelChunkSize = u16;

#[derive(Clone, PartialEq, Eq)]
pub struct Model {
    pub configuration: ModelConfiguration,
    pub pattern_list: ModelPatternList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelConfiguration {
    pub chunk_length: usize,
    pub activation: ModelActivationFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelActivationFunction {
    Equal,
    Greater,
    Lesser,
    ZeroRem,
    NonZeroRem,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ModelPatternList {
    pub patterns: Vec<ModelPatternNode>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ModelPatternNode {
    pub chunk_activation_values: Vec<ModelCell>,
    pub sub_patterns: Vec<ModelPatternNode>,
}
