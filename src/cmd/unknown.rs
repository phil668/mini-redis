use crate::parse::Parse;

#[derive(Debug)]
pub struct Unknown {
    command_name: String,
}

impl Unknown {
    pub fn new(command_name: impl ToString) -> Unknown {
        Unknown {
            command_name: command_name.to_string(),
        }
    }

    fn get_name(&self) -> &str {
        &self.command_name
    }
}
