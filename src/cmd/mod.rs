mod get;

#[derive(Debug)]
pub enum Command {
    Get,
    Publish,
    Set,
    Sbuscribe,
    Unsubscribe,
    Ping,
    Unknown,
}
