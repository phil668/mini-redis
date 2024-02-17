use crate::parse::Parse;

#[derive(Debug)]
pub struct Get {
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

    // Get name 解析出name
    pub fn parse_frame(parse: &mut Parse) -> crate::Result<Get> {
        let key = parse.next_string()?;
        Ok(Get { key })
    }
}
