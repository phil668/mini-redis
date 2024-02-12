struct Get {
    key: String,
}

impl Get {
    fn new(key: impl ToString) -> Self {
        Get {
            key: key.to_string(),
        }
    }

    fn key(&self) -> &str {
        &self.key
    }
}
