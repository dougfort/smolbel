#[derive(Debug, Clone)]
pub enum State {
    Default,
    AccumulateChar(String),
}
