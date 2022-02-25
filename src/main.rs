

fn main() {
    let client_vals: [f64;10] = rand::random();
    let mut masked_vals: [f64;10] = client_vals.clone();
    for i in 0..10 {
        for j in i+1..10 {
            let masking_val: f64 = rand::random();
            masked_vals[i] += masking_val;
            masked_vals[j] -= masking_val;
        }
    }
    let naive_aggregate: f64 = client_vals.iter().sum();
    println!("Server Aggregate result: {:.2}", aggregate_server(&masked_vals));
    println!("Naive Aggregate result: {:.2}", naive_aggregate);
}

fn aggregate_server(masked_vals: &[f64]) -> f64{
    masked_vals.iter().sum()
}