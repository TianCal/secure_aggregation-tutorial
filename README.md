# Tutorial: Basic Secure Aggregation
## 1.1 Naive Simulation
People fill out online surveys from time to time, and a majority of the surveys claim that they will guarantee **anonymities** to their participants. People also are confident that privacy will not be invaded, since they don't write their names or addresses in the survey. However, their responses are sent to the organizers **intact**, without any processing, along with their IP addresses. 

Is there a way for the survey organizer to receive the overall results of the survey, but not know about each individual questionnaire? For sure we also don't want any participant to hold information about other participants. It turns out this is possible with **Secure Aggregation protocols**, 

> **Secure Aggregation protocols** allow a collection of mutually distrust parties, each holding a private value, to collaboratively compute the sum of those values without revealing the values themselves.

One simplest secure aggregation protocol is to **add antiparticles before sending numbers from client (participant) to the server (survey organizer)**.  Each pair of users generate a random masking number, and one user adds the number to his value, while the other subtracts the number from his value. When they send their values to the server, the inputs will be random when viewed alone, but summing them up will cancel out the masking values and give the correct summed value.

Let's imagine the university wants to estimate the average hours of sleep its students have everyday, and it presents a survey to students, asking them to fill in the number.  

Let's try to simulate with **Rust**, we first take 1000 random values from range 5 to 12, which means 100 students take the survey, and their answers are uniformly distributed in the range [5, 12].

```rust
let num_participants = 1000;
let range = Uniform::from(5..12);
let client_vals: Vec<Wrapping<u32>> = rand::thread_rng().sample_iter(&range).take(num_participants).map(|x| Wrapping(x)).collect();
let mut masked_vals: Vec<Wrapping<u32>> = client_vals.clone();
```

Then, to conduct the secure aggregation protocol above, we do some masking on these values before sending them to the server. For each pair of students, we generate a random masking value in range of u32 values. 

Notice that we used `Wrapping<u32>` type instead of `u32` for all values in the code. This is because we want the server to get completely no information from the client, and we need the random number to be taken from the whole range of `u32`. However, operations like summing two large values might cause overflow in rust. Wrapping u32 up in rust could tolerate overflow to allow modular arithmetic, which enhances the privacy of the client.

We add the value to one student's sleeping time, and then subtract the value from the other's sleeping time.  One thing to notice is that each student's sleeping time is masked by **999** masking values since it has a pair with every other student.

```rust
for i in 0..num_participants {
    for j in i+1..num_participants {
        let masking_val: Wrapping<u32> = Wrapping(rand::thread_rng().gen());
        masked_vals[i] = masked_vals[i] + masking_val;
        masked_vals[j] = masked_vals[j] - masking_val;
    }
}
```

Finally, we are gonna aggregate the masked values as a simulation action for server, and check if the server gets the same aggregate value as simply summing up all unmasked values. 

```rust
let naive_aggregate: Wrapping<u32> = client_vals.iter().sum();
let server_aggregate: Wrapping<u32> = masked_vals.iter().sum();
println!("Server Aggregate result: {:.2}", server_aggregate);
println!("Naive Aggregate result: {:.2}", naive_aggregate);

/* Code output
Server Aggregate result: 7978
Naive Aggregate result: 7978
*/
```
We can see that the aggregate server, without knowing the actual sleeping hours of each student, still get the correct summation of the values, and they can compute the average sleeping time with the value. This is how secure aggregation works.



### Full code
```rust
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
```

## 1.2 Abstraction

We wrote a complete simulation in the last chapter, but apparently the abstractions of the program are terrible. It's hard to distinguish what the program is doing without a complete reading guide like the last chapter. Let's try to write it in a clearer way now. 

### Client

First, let's define a struct called `Client` , with one field representing the original value it wants to send to the server, one field representing the masked value, and last field representing it's name.

```rust
struct Client {
    value: Wrapping<u32>,
    masked_value: Wrapping<u32>,
    name: String,
}
```

Then we define 5 functions for this struct

* `Initialize_multiple` and `new` are used to initialize a number of participating clients, where each client's original value is randomly taken from [5, 12].

  ```rust
  pub fn initialize_multiple (num_participants: i64) -> Vec<Client> {
      let mut clients: Vec<Client> = Vec::new();
      for i in 0..num_participants {
          let client_name: String = String::from(format!("Client #{}", i+1));
          let client_value: Wrapping<u32> = Wrapping(rand::thread_rng().gen_range(5..12));
          let mut curr_client = Client::new(client_name, client_value);
              clients.push(curr_client);
      }
      clients
  }
  
  pub fn new (name: String, sending_value: Wrapping<u32>) -> Client {
      Client {value: sending_value, name:name, masked_value: sending_value}
  }
  ```

* `interact_with_others` is called when a client is provided with lists of its collaborating clients, and the client will generate a masking value for each of its collaborator, subtract the masking value from the masked value, and call `add_to_value` for each of its collaborators to let them add the masking value to their masked value.

  ```rust
  fn add_to_value (&mut self, masking_val: Wrapping<u32>) {
      self.masked_value = self.masked_value + masking_val;
  }
  
  fn interact_with_others (&mut self, other_clients: &mut Vec<Client>) {
      for curr_collaborator in other_clients {
          let masking_val: Wrapping<u32> = Wrapping(rand::thread_rng().gen());
          self.masked_value = self.masked_value - masking_val;
          curr_collaborator.add_to_value(masking_val);
      } 
  }
  ```

* `share_value` is called for each client when the clients finished masking their values and are ready to share the masked value to the server.

  ```rust
  fn share_value (&self) -> Wrapping<u32> {
      self.masked_value
  }
  ```

### Server

We also need a struct for `Server`, with the one field representing the aggregate result, and the other representing the list of all its clients.

```rust
struct Server {
    aggregate_value: Wrapping<u32>,
    clients: Vec<Client>,
}
```

`Server` only needs two functions, one is `initialize`, which tells its clients to prepare for sharing the masked value by calling `interact_with_others` for each of its client.

```rust
fn initialize(&mut self) {
    for i in 0..self.clients.len() {
        let mut current_client = self.clients.remove(i);
        current_client.interact_with_others(&mut self.clients);
        self.clients.insert(i, current_client);
    }
}
```

The other function is `aggregate`. It simply asks for server's clients to share their masked values, and sums up all the values to get the final aggregate value

```rust
fn aggregate(&mut self) -> Wrapping<u32> {
    let mut ret:Wrapping<u32> = Wrapping(0u32);
    for i in 0..self.clients.len() {
        ret += self.clients[i].share_value();
    }
    self.aggregate_value = ret;
    self.aggregate_value
}
```

### Main

Now we have some idea about what servers and clients can do. Let's try to simulate the sleeping-time survey again.

We assume there are 1000 participants (clients), and we initialize 1000 clients by calling `Client::initialize_multiple` and initialize a server instance by passing in the vector of clients and initial aggregate value of zero to the server constructor.

```
let num_participants = 1000;
let mut clients: Vec<Client> = Client::initialize_multiple(num_participants);
let mut server: Server = Server{aggregate_value:Wrapping(0), clients:clients};
```

Before letting clients process their values, we first aggregate their original values. Then we call `initialize` of the server which asks each client to mask their values. Finally we print out the server's aggregate value to see if it equals the naive aggregate value.

```rust
let naive_aggregate: Wrapping<u32> = server.clients.iter().map(|c| c.value).sum();
println!("Naive Aggregate result: {:.2}", naive_aggregate);

server.initialize();
println!("Server Aggregate result: {:.2}", server.aggregate());

/* Code output
Server Aggregate result: 7942
Naive Aggregate result: 7942
*/
```

With a clearer abstraction, the server only gets the masked value from its clients, and each client only knows its own value.  We can see that two results are the same. This is how basic secure aggregation works.

### Full Code

```rust
use rand::{distributions::Uniform, Rng};
use std::num::Wrapping;

#[derive(Debug)]
struct Client {
    value: Wrapping<u32>,
    masked_value: Wrapping<u32>,
    name: String,
}

impl Client {
    pub fn initialize_multiple (num_participants: i64) -> Vec<Client> {
        let mut clients: Vec<Client> = Vec::new();
        for i in 0..num_participants {
            let client_name: String = String::from(format!("Client #{}", i+1));
            let client_value: Wrapping<u32> = Wrapping(rand::thread_rng().gen_range(5..12));
            let mut curr_client = Client::new(client_name, client_value);
            clients.push(curr_client);
        }
        clients
    }

    pub fn new (name: String, sending_value: Wrapping<u32>) -> Client {
        Client {value: sending_value, name:name, masked_value: sending_value}
    }

    fn add_to_value (&mut self, masking_val: Wrapping<u32>) {
        self.masked_value = self.masked_value + masking_val;
    }

    fn interact_with_others (&mut self, other_clients: &mut Vec<Client>) {
        for curr_collaborator in other_clients {
            let masking_val: Wrapping<u32> = Wrapping(rand::thread_rng().gen());
            self.masked_value = self.masked_value - masking_val;
            curr_collaborator.add_to_value(masking_val);
        } 
    }

    fn share_value (&self) -> Wrapping<u32> {
        self.masked_value
    }
}

#[derive(Debug)]
struct Server {
    aggregate_value: Wrapping<u32>,
    clients: Vec<Client>,
}

impl Server {
    fn initialize(&mut self) {
        for i in 0..self.clients.len() {
            let mut current_client = self.clients.remove(i);
            current_client.interact_with_others(&mut self.clients);
            self.clients.insert(i, current_client);
        }
    }

    fn aggregate(&mut self) -> Wrapping<u32> {
        let mut ret:Wrapping<u32> = Wrapping(0u32);
        for i in 0..self.clients.len() {
            ret += self.clients[i].share_value();
        }
        self.aggregate_value = ret;
        self.aggregate_value
    }
}

fn main() {
    let num_participants = 1000;
    let mut clients: Vec<Client> = Client::initialize_multiple(num_participants);
    let mut server: Server = Server{aggregate_value:Wrapping(0), clients:clients};
    
    let naive_aggregate: Wrapping<u32> = server.clients.iter().map(|c| c.value).sum();
    println!("Naive Aggregate result: {:.2}", naive_aggregate);

    server.initialize();
    println!("Server Aggregate result: {:.2}", server.aggregate());
}

```