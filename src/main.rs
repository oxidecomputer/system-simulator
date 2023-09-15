use client::Client;
use data_processor::DataProcessor;
use futures::future::join_all;
use network::Network;

mod client;
mod data_processor;
mod network;
mod persist;

#[tokio::main]
async fn main() {
    println!("System simulation running...");

    let network = Network::new();
    let server = DataProcessor::new();

    // Specify the number of clients:
    // 15 causes very little blocking behavior (in the steady state)
    // 125 causes some blocking behavior and some non-blocking
    // 135: essentially all requests incur blocking
    const N_CLIENTS: usize = 125;

    // Make the prescribed number of clients.
    let clients = (0..N_CLIENTS)
        .map(|_| Client::new(&network, &server))
        .collect::<Vec<_>>();

    // Run their processing loops.
    join_all(clients.iter().map(|c| c.go())).await;
}
