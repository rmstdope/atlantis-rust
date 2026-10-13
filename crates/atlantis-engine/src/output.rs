#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EngineOutput {
    buffer: String,
}

impl EngineOutput {
    pub fn new() -> Self {
        Self::default()
    }

    /// C++: gameio.cpp `Awrite`; appends one line feed after the text.
    pub fn write_line(&mut self, text: &str) {
        self.buffer.push_str(text);
        self.buffer.push('\n');
    }

    /// C++: gameio.cpp `Adot`; appends a dot without a line feed.
    pub fn write_dot(&mut self) {
        self.buffer.push('.');
    }

    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    pub fn into_string(self) -> String {
        self.buffer
    }
}
