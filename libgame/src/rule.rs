#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub birth: Vec<usize>,
    pub survive: Vec<usize>,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            birth: vec![3],
            survive: vec![2, 3],
        }
    }
}
