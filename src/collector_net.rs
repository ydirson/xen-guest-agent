use crate::publisher::Publisher;
use std::error::Error;
use std::io;

pub struct NetworkSource {
}

impl NetworkSource {
    pub fn new() -> io::Result<NetworkSource> {
        Ok(NetworkSource {})
    }

    pub async fn collect_publish_current(&mut self, __publisher: &Publisher
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub async fn collect_publish_loop(&mut self, publisher: &Publisher
    ) -> io::Result<()> {
        Ok(())
    }
}
