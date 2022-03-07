use rand::{distributions::Uniform, Rng};
use std::num::Wrapping;

fn main() {
    let num_participants = 1000;
    let range = Uniform::from(5..12);
    let client_vals: Vec<Wrapping<u32>> = rand::thread_rng().sample_iter(&range).take(num_participants).map(|x| Wrapping(x)).collect();
    let mut masked_vals: Vec<Wrapping<u32>> = client_vals.clone();
    for i in 0..num_participants {
        for j in i+1..num_participants {
            let masking_val: Wrapping<u32> = Wrapping(rand::thread_rng().gen());
            masked_vals[i] = masked_vals[i] + masking_val;
            masked_vals[j] = masked_vals[j] - masking_val;
        }
    }
    let naive_aggregate: Wrapping<u32> = client_vals.iter().sum();
    let server_aggregate: Wrapping<u32> = masked_vals.iter().sum();
    println!("Server Aggregate result: {:.2}", server_aggregate);
    println!("Naive Aggregate result: {:.2}", naive_aggregate);
}