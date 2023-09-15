use std::time::Duration;
use tokio::time::sleep;
use usdt::UniqueId;

use crate::probes;
use crate::{data_processor::DataProcessor, network::Network};

pub struct Client<'a> {
    network: &'a Network,
    server: &'a DataProcessor,
}

impl<'a> Client<'a> {
    pub fn new(network: &'a Network, server: &'a DataProcessor) -> Self {
        Self { network, server }
    }

    pub async fn go(&self) {
        // Forever...
        loop {
            // Generate an id simply to correlate start and done probes.
            let id = UniqueId::new();
            probes::request__start!(|| &id);
            // Send the request over the network.
            self.network.traverse().await;
            // Make a request to the server.
            self.server.request(&id).await;
            // Return the response over the network.
            self.network.traverse().await;
            probes::request__done!(|| &id);

            // Pause for a second.
            sleep(Duration::from_secs(1)).await;
        }
    }
}
