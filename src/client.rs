use rand::{distributions::Uniform, Rng};
use std::env;
use warp::Filter;
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let client_val: u32 = rand::thread_rng().gen_range(5..12);
    // Let Client #N serve port (3000+N)
    let port = 3000 + args[1].parse::<u16>().unwrap();
    println!("Client {} with Value: {}", port, client_val);

    let client = models::new_Client(client_val, String::from(format!("Client #{}", port)));
    let apis = filters::client_ops(client);
    warp::serve(apis).run(([127, 0, 0, 1], port)).await;
}

mod filters {
    use super::handlers;
    use super::models::Client_Async;
    use warp::Filter;

    pub fn client_ops(
        client: Client_Async,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        mask_by_adding(client.clone())
            .or(share_val(client.clone()))
            .or(interact_with_others(client.clone()))
    }
    /// GET /shareval
    pub fn share_val(
        client: Client_Async,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let with_client = warp::any().map(move || client.clone());
        warp::path!("sharevalue")
            .and(warp::get())
            .and(with_client)
            .and_then(handlers::share_val)
    }

    /// POST /maskbyadding
    pub fn mask_by_adding(
        client: Client_Async,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let with_client = warp::any().map(move || client.clone());
        warp::path!("maskbyadding" / u32)
            .and(warp::post())
            .and(with_client)
            .and_then(handlers::mask_by_adding)
    }

    /// PUT /interact
    pub fn interact_with_others(
        client: Client_Async,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let with_client = warp::any().map(move || client.clone());
        warp::path!("interact")
            .and(warp::put())
            .and(warp::body::json())
            .and(with_client)
            .and_then(handlers::interact_with_others)
    }
}

mod handlers {
    use super::models::{Client_Async, Collaborator_list};
    use rand::{distributions::Uniform, Rng};
    use std::convert::Infallible;
    use std::num::Wrapping;
    use warp::http::{Response, StatusCode};

    pub async fn mask_by_adding(
        masking_val: u32,
        client_async: Client_Async,
    ) -> Result<impl warp::Reply, Infallible> {
        let mut client = client_async.lock().await;
        client.masked_value = client.masked_value + Wrapping(masking_val);
        println!(
            "Added {} to masked val: {}",
            masking_val, client.masked_value
        );
        Ok(StatusCode::OK)
    }

    pub async fn share_val(client_async: Client_Async) -> Result<impl warp::Reply, Infallible> {
        let mut client = client_async.lock().await;
        println!("Shared masked val: {}", client.masked_value);
        Ok(Response::new(format!("{}", client.masked_value)))
    }

    pub async fn interact_with_others(
        collaborator_port_list: Collaborator_list,
        client_async: Client_Async,
    ) -> Result<impl warp::Reply, Infallible> {
        let mut client = client_async.lock().await;
        let http_client = reqwest::Client::new();
        for curr_collaborator in collaborator_port_list.port_list {
            let masking_val: Wrapping<u32> = Wrapping(rand::thread_rng().gen());
            client.masked_value = client.masked_value - masking_val;
            let res = http_client
                .post(format!(
                    "http://localhost:{}/maskbyadding/{}",
                    curr_collaborator, masking_val
                ))
                .send()
                .await;
            println!(
                "---------\n \
                    Interacted with port: {} with masking value {}, \n \
                    and now has masked val {} \n \
                    ---------",
                curr_collaborator, masking_val, client.masked_value
            );
        }
        Ok(Response::new(format!("Interaction successful")))
    }
}

mod models {
    use serde_derive::{Deserialize, Serialize};
    use std::num::Wrapping;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    pub fn new_Client(sending_value: u32, name: String) -> Client_Async {
        Arc::new(Mutex::new(Client {
            value: Wrapping(sending_value),
            name: name,
            masked_value: Wrapping(sending_value),
        }))
    }

    #[derive(Debug, Clone)]
    pub struct Client {
        pub value: Wrapping<u32>,
        pub masked_value: Wrapping<u32>,
        pub name: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Collaborator_list {
        pub port_list: Vec<u32>,
        pub num_collaborators: u32,
    }
    pub type Client_Async = Arc<Mutex<Client>>;
}

