use crate::datastructs::NetEvent;
use futures::stream::Stream;
use std::error::Error;
use std::io;

pub struct NetworkSource {
}

impl NetworkSource {
    pub fn new() -> io::Result<NetworkSource> {
        Ok(NetworkSource {})
    }

    pub async fn collect_current(&mut self) -> Result<Vec<NetEvent>, Box<dyn Error>> {
        Ok(vec!())
    }

    pub fn stream(&mut self) -> impl Stream<Item = io::Result<NetEvent>> + '_ {
        futures::stream::empty::<io::Result<NetEvent>>()
    }
}
