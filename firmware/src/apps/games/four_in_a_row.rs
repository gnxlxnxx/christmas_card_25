
pub struct Task {}

impl Task {
    pub fn new() -> Self {
        Self {}
    }

    pub fn poll(&mut self) -> bool {
        true
    }
}
