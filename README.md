# Tutorial: Secure Aggregation



```rust

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
```

People fill out online surveys from time to time, and a majority of the surveys claim that they will guarantee **anonymities** to their participants. People also are confident that privacy will not be invaded, since they don't write their names or addresses in the survey. However, their responses are sent to the organizers **intact**, without any processing, along with their IP addresses. 

Is there a way for the survey organizer to receive the overall results of the survey, but not know about each individual questionnaire? For sure we also don't want any participant to hold information about other participants. It turns out this is possible with **Secure Aggregation protocols**, 

> **Secure Aggregation protocols** allow a collection of mutually distrust parties, each holding a private value, to collaboratively compute the sum of those values without revealing the values themselves.

One simplest secure aggregation protocol is to **add antiparticles before sending numbers from client (participant) to the server (survey organizer)**.  Each pair of users generate a random masking number, and one user adds the number to his value, while the other subtracts the number from his value. When they send their values to the server, the inputs will be random when viewed alone, but summing them up will cancel out the masking values and give the correct summed value.

Let's imagine the university wants to estimate the average hours of sleep its students have everyday, and it presents a survey to students, asking them to fill in the number.  

Let's try to simulate with **Rust**, we first take 100 random values from range 5 to 12, which means 100 students take the survey, and their answers are uniformly distributed in the range $[5, 12]$.

```rust
let range = Uniform::from(5..12);
let client_vals: Vec<i64> = rand::thread_rng().sample_iter(&range).take(100).collect();

```

Then, to conduct the secure aggregation protocol above, we do some masking on these values before sending them to the server. For each pair of students, we generate a random masking value in range $[-10, 10]$,  we add the value to one student's sleeping time, and then subtract the value from the other's sleeping time.  One thing to notice is that each student's sleeping time is masked by **99** masking values since it has a pair with every other student.

```rust
let mut masked_vals: Vec<i64> = client_vals.clone();
for i in 0..100 {
    for j in i+1..100 {
        
        let masking_val: i64 = rand::thread_rng().gen_range(-10..10);
        masked_vals[i] += masking_val;								
        masked_vals[j] -= masking_val;
    }
}
```

Finally, we are gonna send these masked sleeping hours to the server, and check if the server gets the same aggregate value as simply summing up all unmasked values. 

```rust
let naive_aggregate: i64 = client_vals.iter().sum();
println!("Server Aggregate result: {:.2}", aggregate_server(masked_vals));
println!("Naive Aggregate result: {:.2}", naive_aggregate);

/* Code output
Server Aggregate result: 797
Naive Aggregate result: 797
*/
```

```rust
// This is what the server need to do
fn aggregate_server(masked_vals: Vec<i64>) -> i64{
    masked_vals.iter().sum()
}
```

We can see that the aggregate server, without knowing the actual sleeping hours of each student, still get the correct summation of the values, and they can compute the average sleeping time with the value. This is how secure aggregation works.









