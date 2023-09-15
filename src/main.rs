use client::Client;
use data_processor::DataProcessor;
use futures::future::join_all;
use network::Network;

#[usdt::provider(provider = "isim")]
mod probes {
    fn request__start(_: &UniqueId) {}
    fn request__done(_: &UniqueId) {}
    fn persist__start(_: &UniqueId) {}
    fn persist__done(_: &UniqueId) {}
    fn request__block(_: &UniqueId) {}
    fn request__nonblock(_: &UniqueId) {}
}

mod client;
mod data_processor;
mod network;
mod persist;

#[tokio::main]
async fn main() {
    usdt::register_probes().expect("Failed to register probes");
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
