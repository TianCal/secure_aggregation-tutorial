# Tutorial: Basic Secure Aggregation

## 1.0 Use Case

People fill out online surveys from time to time, and a majority of the surveys claim that they will guarantee **anonymities** to their participants. People also are confident that privacy will not be invaded, since they don't write their names or addresses in the survey. However, their responses are sent to the organizers **intact**, without any processing, along with their IP addresses. 

In this tutorial, we propose a privacy-preserving way for servers(survey organizers) and clients(participants) to transfer data to each other.  We ensure that the survey organizer receive the overall results of the survey, but not know about each individual questionnaire. To accomplish this goal, we need to make use of **Secure Aggregation protocols**.

> **Secure Aggregation protocols** allow a collection of mutually distrust parties, each holding a private value, to collaboratively compute the sum of those values without revealing the values themselves. ([Source](https://research.google/pubs/pub45808/))

However, secure aggregation protocols need all clients(participants) to be online when the server(survey organizer) is collecting the survey answers. With this condition, we propose the following toy scenario for the tutorial.

University wants to estimate students' pressure by collecting their average sleep time per day. All participants receive the survey on Day 1, and have to fill out the survey with their sleeping time before Day 2. On Day 2, all participants need to tell University their IP addresses. On Day 3, University will collect the results from participants and all participants need to be online this day. University will then use secure aggregation to collect the sum of all participants' sleeping time without sacrificing the privacy of any participant. University can then calculate the average sleeping time by dividing with the participant count.

## 1.1 Naive Simulation
We begin by discussing one simplest secure aggregation protocol. The protocol is to **add masking values before sending numbers from client (participant) to the server (survey organizer)**.  Each pair of users generate a random masking number, and one user adds the number to his value, while the other subtracts the number from his value. When they send their values to the server, the inputs will be random when viewed alone, but summing them up will cancel out the masking values and give the correct summed value.

Let's imagine the university wants to estimate the average hours of sleep its students have everyday, and it presents a survey to students, asking them to fill in the number.  

Let's try to simulate with **Rust**, we first take 1000 random sleep hours from range 5 to 12, which means 100 students take the survey, and their answers are uniformly distributed in the range [5, 12].

```rust
let num_participants = 1000;
let range = Uniform::from(5..12);
let client_vals: Vec<Wrapping<u32>> = rand::thread_rng().sample_iter(&range).take(num_participants).map(|x| Wrapping(x)).collect();
let mut masked_vals: Vec<Wrapping<u32>> = client_vals.clone();
```

Then, to conduct the secure aggregation protocol above, we mask these values before sending them to the server. For each pair of students, we generate a random masking value in range of u32 values. 

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

In the last chapter, we showed the correctness of the basic secure aggregation protocol. However, in the real world, to support the application that's mentioned in chapter 1.0, we need to use separate servers for different clients and aggregation server.

In this chapter,  we first illustrate the abstraction for the interface for both the client and the aggregation server. We will extend it to a real-world network-based solution in the next chapter.

### Client

First, let's define a struct called `Client` , with one field representing the original value it wants to send to the server, one field representing the masked value, and last field representing it's name.

```rust
struct Client {
    value: Wrapping<u32>,
    masked_value: Wrapping<u32>,
    name: String,
}
```

We have 4 functions for the interface of the client:

* `new` is used to initialize a new client with a sleeping time passed in as `sending_value` and a name passed in as `name`

  ```rust
  pub fn new (name: String, sending_value: Wrapping<u32>) -> Client {
      Client {value: sending_value, name:name, masked_value: sending_value}
  }
  ```
  
* The aggregation server calls `interact_with_others` to provide a client with lists of its collaborating clients, and the client will generate a masking value for each of its collaborator, subtract the masking value from the masked value, and call `mask_by_adding` for each of its collaborators to let them add the masking value to their masked value.

  ```rust
  fn mask_by_adding (&mut self, masking_val: Wrapping<u32>) {
      self.masked_value = self.masked_value + masking_val;
  }
  
  fn interact_with_others (&mut self, other_clients: &mut Vec<Client>) {
      for curr_collaborator in other_clients {
          let masking_val: Wrapping<u32> = Wrapping(rand::thread_rng().gen());
          self.masked_value = self.masked_value - masking_val;
          curr_collaborator.mask_by_adding(masking_val);
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

We have two functions for server's interface:

* `initialize` tells its clients to prepare for sharing the masked value by calling `interact_with_others` for each of its client.

  ```rust
  fn initialize(&mut self) {
      for i in 0..self.clients.len() {
          let mut current_client = self.clients.remove(i);
          current_client.interact_with_others(&mut self.clients);
          self.clients.insert(i, current_client);
      }
  }
  ```

* `aggregate` asks for server's clients to share their masked values, and sums up all the values to get the final aggregate value

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

We assume there are 1000 participants (clients), and we initialize 1000 clients with a for loop and initialize a server instance by passing in the vector of clients and initial aggregate value of zero to the server constructor.

```rust
let num_participants = 1000;
let mut clients: Vec<Client> = Vec::new();
// Generate all the clients with a for loop.
for i in 0..num_participants {
    let client_name: String = String::from(format!("Client #{}", i+1));
    let client_value: Wrapping<u32> = Wrapping(rand::thread_rng().gen_range(5..12));
    let mut curr_client = Client::new(client_name, client_value);
    clients.push(curr_client);
}
// Generate a server and tells it all the clients we just generated.
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

With a clearer abstraction, the server only gets the masked value from its clients, and each client only knows its own value.  We can see that two results are the same. These are the interfaces for clients and the aggregation server to implement secure aggregation.

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
    pub fn new (name: String, sending_value: Wrapping<u32>) -> Client {
        Client {value: sending_value, name:name, masked_value: sending_value}
    }

    fn mask_by_adding (&mut self, masking_val: Wrapping<u32>) {
        self.masked_value = self.masked_value + masking_val;
    }

    fn interact_with_others (&mut self, other_clients: &mut Vec<Client>) {
        for curr_collaborator in other_clients {
            let masking_val: Wrapping<u32> = Wrapping(rand::thread_rng().gen());
            self.masked_value = self.masked_value - masking_val;
            curr_collaborator.mask_by_adding(masking_val);
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
    let mut clients: Vec<Client> = Vec::new();
    // Generate all the clients with a for loop.
    for i in 0..num_participants {
        let client_name: String = String::from(format!("Client #{}", i+1));
        let client_value: Wrapping<u32> = Wrapping(rand::thread_rng().gen_range(5..12));
        let mut curr_client = Client::new(client_name, client_value);
        clients.push(curr_client);
    }
    // Generate a server and tells it all the clients we just generated.
    let mut server: Server = Server{aggregate_value:Wrapping(0), clients:clients};
    
    let naive_aggregate: Wrapping<u32> = server.clients.iter().map(|c| c.value).sum();
    println!("Naive Aggregate result: {:.2}", naive_aggregate);

    server.initialize();
    println!("Server Aggregate result: {:.2}", server.aggregate());
}



```

## 1.3 Server implementation

Now we have a clear abstraction of clients' and server's interfaces. However, in real-world applications, clients and aggregation servers will not reside in one single machine, and thus we need to have a network-based implementation for secure aggregation protocol.
