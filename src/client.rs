use rand::Rng;
use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep;

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
            // Generate a random request ID
            let id = rand::thread_rng().gen();
            let now = Instant::now();
            println!("sending request {id}");
            // Send the request over the network.
            self.network.traverse().await;
            // Make a request to the server.
            self.server.request(id).await;
            // Return the response over the network.
            self.network.traverse().await;
            println!("finished request {id} in {:?}", now.elapsed());

            // Pause for a second.
            sleep(Duration::from_secs(1)).await;
        }
    }
}
