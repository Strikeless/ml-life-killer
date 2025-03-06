use std::{env, fs, path::PathBuf, time::Instant};

use network::Network;
use player::kernel::Kernel;
use trainer::Trainer;

mod network;
mod player;
mod trainer;

fn main() {
    let mut args = env::args().skip(1);

    let run_id = args
        .next()
        .map(|run_id| (&run_id != "-").then_some(run_id))
        .flatten()
        .unwrap_or_default();

    let run_prefix = [chrono::Local::now().format("%Y%m%d").to_string(), run_id].join("_");

    let network_input_layer = Kernel::<1>::create_input_layer();

    let mut network = if let Some(network_path) = args.next() {
        let network_serialized = fs::read(network_path).expect("Couldn't read network file");

        let mut network: Network<Kernel<1>> =
            rmp_serde::from_slice(&network_serialized).expect("Couldn't deserialize network file");
        network.input_layer = network_input_layer;
        network
    } else {
        // TODO: Create new network
        Network::new(
            network_input_layer,
            3, // Mid layer count
            3, // Mid layer height
            2, // Output layer height
        )
    };

    let trainer = Trainer {
        generation_contenders: 8,
        generation_iterations: 4,
        generation_mutations: 9,
        generation_unstable: false,
        iteration_max_steps: 32,
        game_board_width: 8,
        game_board_height: 8,
        game_board_alive_cells: 32,

        player_network_consecutive_turns: 1,
        player_game_consecutive_turns: 0, // Disable "nature" entirely
    };

    let mut prev_score = None;
    let mut best_score = None;
    let mut last_notif_instant = Instant::now();

    for generation in 0.. {
        let (trained_network, new_score) = trainer.train_generation(network);
        network = trained_network;

        let improved = best_score.is_some_and(|best_score| new_score > best_score);

        if improved || generation % 1000 == 0 {
            let improved_suffix = (improved)
                .then(|| "IMPROVED".to_owned())
                .unwrap_or_default();

            let time_since_last_notif = Instant::now()
                .duration_since(last_notif_instant)
                .as_millis();
            last_notif_instant = Instant::now();

            println!(
                "score {:04} -> {:04} (best {:04}), gen {:8}, {:04} ms    {}",
                prev_score.unwrap_or(0),
                new_score,
                best_score.unwrap_or(0),
                generation,
                time_since_last_notif,
                improved_suffix,
            );
        }

        if improved {
            let dir = PathBuf::from("networks");

            let path = dir.join(format!("{}_{}", run_prefix, generation));

            let network_serialized =
                rmp_serde::to_vec(&network).expect("Couldn't serialize network");

            let _ = fs::create_dir_all(&dir);
            fs::write(path, network_serialized).expect("Couldn't save serialized network");
        }

        best_score = Some(
            best_score
                .map(|prev_best| prev_best.max(new_score))
                .unwrap_or(new_score),
        );
        prev_score = Some(new_score);
    }
}
