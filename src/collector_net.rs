use crate::datastructs::{NetEvent, NetInterfaceCache};
use futures::stream::Stream;
use std::error::Error;
use std::io;

pub struct NetworkSource {
}

impl NetworkSource {
    pub fn new(_cache: &'static mut NetInterfaceCache) -> io::Result<NetworkSource> {
        Ok(NetworkSource {})
    }

    pub async fn collect_current(&mut self) -> Result<Vec<NetEvent>, Box<dyn Error>> {
        Ok(vec!())
    }

    pub fn stream(&mut self) -> impl Stream<Item = io::Result<NetEvent>> + '_ {
        futures::stream::empty::<io::Result<NetEvent>>()
    }
}
