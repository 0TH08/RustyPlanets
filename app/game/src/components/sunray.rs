#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sunray {
    _private: (),
}

impl Default for Sunray {
    fn default() -> Self {
        Self::new()
    }
}

impl Sunray {
    pub fn new() -> Sunray {
        Sunray { _private: () }
    }
}
