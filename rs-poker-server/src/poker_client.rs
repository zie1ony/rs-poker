pub struct PokerClient;

impl PokerClient {
    /// Create a new PokerClient instance
    pub fn new() -> Self {
        PokerClient
    }

    pub fn ping(&self) -> String {
        todo!("Implement ping method")
    }

    pub fn as_json(&self, payload: serde_json::Value) -> serde_json::Value {
        todo!("Implement as_json method")
    }
}