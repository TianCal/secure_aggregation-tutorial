
use rand::{distributions::Uniform, Rng};

fn main() {
    let range = Uniform::from(5..12);
    let client_vals: Vec<i64> = rand::thread_rng().sample_iter(&range).take(100).collect();
    let mut masked_vals: Vec<i64> = client_vals.clone();
    for i in 0..100 {
        for j in i+1..100 {
            let masking_val: i64 = rand::thread_rng().gen_range(-10..10);
            masked_vals[i] += masking_val;
            masked_vals[j] -= masking_val;
        }
    }
    let naive_aggregate: i64 = client_vals.iter().sum();
    println!("Server Aggregate result: {:.2}", aggregate_server(masked_vals));
    println!("Naive Aggregate result: {:.2}", naive_aggregate);
}

fn aggregate_server(masked_vals: Vec<i64>) -> i64{
    masked_vals.iter().sum()
}