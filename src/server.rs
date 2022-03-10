use warp::Filter;
use rand::{distributions::Uniform, Rng};
use std::env;
#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"

    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let server = models::new_Server();

    let apis = filters::server_ops(server);
    // Let Client #N serve port (3000+N)
    warp::serve(apis)
        .run(([127, 0, 0, 1], 3000 + args[1].parse::<u16>().unwrap()))
        .await;
}


mod filters {
    use warp::Filter;
    use super::handlers;
    use super::models::{Server_Async};

    pub fn server_ops(
        server: Server_Async,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        initialize(server.clone()).
            or(aggregate_val(server.clone()))
    }

    /// PUT /initialize
    pub fn initialize (
        server: Server_Async
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let with_server = warp::any().map(move || server.clone());

        warp::path!("initialize")
            .and(warp::put())
            .and(warp::body::json())
            .and(with_server)
            .and_then(handlers::initialize)
    }

    /// GET /aggregate
    pub fn aggregate_val (
        server: Server_Async
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let with_server = warp::any().map(move || server.clone());

        warp::path!("aggregate")
            .and(warp::get())
            .and(with_server)
            .and_then(handlers::aggregate_val)
    }
}

mod handlers {
    use std::num::Wrapping;
    use super::models::{Server_Async, Collaborator_list};
    use std::convert::Infallible;
    use warp::http::{StatusCode, Response};

    pub async fn initialize(clients_list: Collaborator_list, server_async: Server_Async) -> Result<impl warp::Reply, Infallible>{
        let mut server = server_async.lock().await;
        server.client_ports = clients_list.port_list.clone();
        let mut collaborator_list = clients_list.clone();
        let http_client = reqwest::Client::new();
        for i in 0..collaborator_list.num_collaborators{
            let curr_client = collaborator_list.port_list.remove(i);
            collaborator_list.num_collaborators -= 1;
            let res = http_client.put(format!("http://localhost:{}/interact", curr_client))
                .json(&collaborator_list)
                .send()
                .await;
            collaborator_list.port_list.insert(i, curr_client);
            collaborator_list.num_collaborators += 1;
        }
        Ok(Response::new(format!("Initialized Clients: {:#?}", collaborator_list.port_list)))
    }

    pub async fn aggregate_val(server_async: Server_Async) -> Result<impl warp::Reply, Infallible>{
        let mut server = server_async.lock().await;
        let http_client = reqwest::Client::new();
        let mut aggregate_val: Wrapping<u32> = Wrapping(0);
        for i in 0..server.client_ports.len() {
            let res = http_client.get(format!("http://localhost:{}/sharevalue", server.client_ports[i]))
                .send()
                .await;
            let masked_val = res.unwrap().text().await.unwrap().parse::<u32>().unwrap();
            println!("Got {} from Client {}", masked_val, server.client_ports[i]);
            aggregate_val += Wrapping(masked_val);
            println!("Now has aggregate value {}", aggregate_val);
        }
        Ok(Response::new(format!("Server Aggregate Result: {} \n", aggregate_val)))
    }
}

mod models {
    use std::num::Wrapping;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use serde_derive::{Deserialize, Serialize};

    pub fn new_Server() -> Server_Async {
        Arc::new(Mutex::new(Server {client_ports: Vec::new()}))
    }

    #[derive(Debug, Clone)]
    pub struct Server {
        pub client_ports: Vec<u32>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Collaborator_list {
        pub port_list: Vec<u32>,
        pub num_collaborators: usize,
    }
    pub type Server_Async = Arc<Mutex<Server>>;
}
