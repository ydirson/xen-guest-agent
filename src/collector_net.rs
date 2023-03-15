use crate::publisher::ConcretePublisher;
use std::error::Error;
use std::io;

pub struct NetworkSource {
}

impl NetworkSource {
    pub fn new() -> Result<NetworkSource, io::Error> {
        Ok(NetworkSource {})
    }

    pub async fn collect_publish_current(&mut self, __publisher: &ConcretePublisher
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
