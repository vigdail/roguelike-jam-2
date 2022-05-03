pub struct GameLog {
    entries: Vec<String>,
}

impl GameLog {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn push<T: Into<String>>(&mut self, entry: T) {
        self.entries.push(entry.into());
    }

    pub fn entries(&self) -> &Vec<String> {
        &self.entries
    }
}

impl Default for GameLog {
    fn default() -> Self {
        Self::new()
    }
}
