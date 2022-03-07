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

