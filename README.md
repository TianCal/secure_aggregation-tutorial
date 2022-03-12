# 1.3 Server implementation

Now we have a clear abstraction of clients' and server's interfaces. However, in real-world applications, clients and aggregation servers will not reside in one single machine, and thus we need to have a network-based implementation for secure aggregation protocol.

[warp](https://docs.rs/warp/latest/warp/) is a easy-to-use web server framework for rust. In this chapter we will use warp to implement the servers for both clients and aggregation server. For each server, we will have three modules, which are **filters**, **handlers** and **models**. **Filters** are used to describe endpoints in our web service, and it will pass the requests into **handlers**. Both **filters** and **handlers** might use structs and constructors defined in **models**. To learn more about warp, here are some official [example web servers](https://github.com/seanmonstar/warp/tree/master/examples).

### Client

* Models

  First , we have the struct `Client` defined the same as the last chapter. On top of that, we have `Client_Async`, which is the same as `Client` but supports asynchronous operations. `new_Client` creates a new Client_Async instance.

  ```rust
  #[derive(Debug, Clone)]
  pub struct Client {
      pub value: Wrapping<u32>,
      pub masked_value: Wrapping<u32>,
      pub name: String,
  }
  
  pub type Client_Async = Arc<Mutex<Client>>;
  
  pub fn new_Client(sending_value: u32, name:String) -> Client_Async {
      Arc::new(Mutex::new(Client {value: Wrapping(sending_value), name:name, masked_value: Wrapping(sending_value)}))
  }
  ```

  Then we define a new struct, `Collaborator_list`. It contains a list of collaborating clients' ports and the number of collaborating clients. Remember in last chapter we passed a vector of collaborating clients into `interact_with_others`. `Collaborator_list` is for the same purpose. 

  ```rust
  #[derive(Debug, Deserialize, Serialize)]
  pub struct Collaborator_list {
      pub port_list: Vec<u32>,
      pub num_collaborators: u32,
  }
  ```

* Filters and Handlers

  The server for client will support three kinds of http requests.

  * The aggregation server sends a PUT request with a json body parsed as `Collaborator_list`. It will hit the `interact_with_others` filter function. The function will catch the request and pass it to the `interact_with_others` handler function. The handler will do the same thing as `interact_with_others` does in the last chapter, but instead of calling collaborators' `mask_by_adding` function, the Client with send another POST request to ask the collaborator to add the masking value to their `masked_value`. 

    ```rust
    // Filter PUT /interact
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
    // Handler
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
        }
        Ok(Response::new(format!("Interaction successful")))
    }
    ```

    ```rust
    // Filter POST /maskbyadding
    pub fn mask_by_adding(
        client: Client_Async,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let with_client = warp::any().map(move || client.clone());
        warp::path!("maskbyadding" / u32)
            .and(warp::post())
            .and(with_client)
            .and_then(handlers::mask_by_adding)
    }
    
    // Handler
    pub async fn mask_by_adding(
        masking_val: u32,
        client_async: Client_Async,
    ) -> Result<impl warp::Reply, Infallible> {
        let mut client = client_async.lock().await;
        client.masked_value = client.masked_value + Wrapping(masking_val);
        Ok(StatusCode::OK)
    }
    ```

  * When clients finish masking their values and are ready to share the masked value to the server, the aggregation server will send a GET request to each client's port. The `share_val` filter function catches the request and passes it to `share_val` handler.

    ```rust
    //  Filter GET /shareval
    pub fn share_val(
        client: Client_Async,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let with_client = warp::any().map(move || client.clone());
        warp::path!("sharevalue")
            .and(warp::get())
            .and(with_client)
            .and_then(handlers::share_val)
    }
    
    // Handler
    pub async fn share_val(client_async: Client_Async) -> Result<impl warp::Reply, Infallible> {
        let mut client = client_async.lock().await;
        println!("Shared masked val: {}", client.masked_value);
        Ok(Response::new(format!("{}", client.masked_value)))
    }
    ```

* Main

  Finally, in client's main function,  we let our program serve localhost's port 3000+N for the Nth client. A new instance of `Client_Async` is created by `new_Client`. This instance will then be used by every filter and handler when processing requests.

  ```rust
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
  ```

###  Server

* Models

  First, we have a `Server` struct which has only one field `client_ports` to record the ports of its clients. Similar as `Client`, we also have a `Server_Async` on top of `Server`. `new_Server` creates a new server instance with empty `client_ports` vector. The other struct is `Collaborator_list` as in the client's models. 

  ```rust
  #[derive(Debug, Clone)]
  pub struct Server {
      pub client_ports: Vec<u32>,
  }
  
  pub type Server_Async = Arc<Mutex<Server>>;
  
  pub fn new_Server() -> Server_Async {
      Arc::new(Mutex::new(Server {client_ports: Vec::new()}))
  }
  
  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Collaborator_list {
      pub port_list: Vec<u32>,
      pub num_collaborators: usize,
  }
  
  ```

* Filters and Handlers

  * `Initialize` will send PUT requests to each of the clients in the `Collaborator_list` to call their `interact_with_others` api.  When the server passes `Collaborator_list` to a client, the server will delete that client from the list so the client doesn't have to collaborate with itself.

    The server will also assign its `client_ports` vector to be the same as `port_list` in `Collabortaor_list` for further aggregation.

    ```rust
    // Filter PUT /initialize
    pub fn initialize(
        server: Server_Async,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let with_server = warp::any().map(move || server.clone());
    
        warp::path!("initialize")
            .and(warp::put())
            .and(warp::body::json())
            .and(with_server)
            .and_then(handlers::initialize)
    }
    
    // Handler
    pub async fn initialize(
        clients_list: Collaborator_list,
        server_async: Server_Async,
    ) -> Result<impl warp::Reply, Infallible> {
        let mut server = server_async.lock().await;
        server.client_ports = clients_list.port_list.clone();
        let mut collaborator_list = clients_list.clone();
        let http_client = reqwest::Client::new();
        for i in 0..collaborator_list.num_collaborators {
            let curr_client = collaborator_list.port_list.remove(i);
            collaborator_list.num_collaborators -= 1;
            let res = http_client
                .put(format!("http://localhost:{}/interact", curr_client))
                .json(&collaborator_list)
                .send()
                .await;
            collaborator_list.port_list.insert(i, curr_client);
            collaborator_list.num_collaborators += 1;
        }
        Ok(Response::new(format!(
            "Initialized Clients: {:#?}",
            collaborator_list.port_list
        )))
    }
    
    ```

  * `aggregate_val` will send a GET request to each of the client in server's `client_ports` vector. This will call every client's `share_val` api and return their masked values. Then the server will sum up all the values to get the final aggregate value.

    ```rust
    // Filter GET /aggregate
    pub fn aggregate_val(
        server: Server_Async,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let with_server = warp::any().map(move || server.clone());
    
        warp::path!("aggregate")
            .and(warp::get())
            .and(with_server)
            .and_then(handlers::aggregate_val)
    }
    
    // Handler
    pub async fn aggregate_val(server_async: Server_Async) -> Result<impl warp::Reply, Infallible> {
        let mut server = server_async.lock().await;
        let http_client = reqwest::Client::new();
        let mut aggregate_val: Wrapping<u32> = Wrapping(0);
        for i in 0..server.client_ports.len() {
            let res = http_client
                .get(format!(
                    "http://localhost:{}/sharevalue",
                    server.client_ports[i]
                ))
                .send()
                .await;
            let masked_val = res.unwrap().text().await.unwrap().parse::<u32>().unwrap();
            aggregate_val += Wrapping(masked_val);
        }
        Ok(Response::new(format!(
            "Server Aggregate Result: {} \n",
            aggregate_val
        )))
    }
    ```

* Main

  In server's main function, we initialize a new instance of `Server_Async` and pass it to all filters and handlers of the server. Similar to client, we will serve at localhost's (3000+N) port, where N is the input argument to the program.

  ```rust
  #[tokio::main]
  async fn main() {
      let args: Vec<String> = env::args().collect();
      println!("{:?}", args);
  
      let server = models::new_Server();
  
      let apis = filters::server_ops(server);
      // Let Server serve port (3000+N)
      warp::serve(apis)
          .run(([127, 0, 0, 1], 3000 + args[1].parse::<u16>().unwrap()))
          .await;
  }
  ```

### Demo

Different from last two chapters, we will have to run multiple programs to test the correctness of our code.

* **Step 1: Create clients**

  For convenience, let's just create 5 clients. Open five different terminals and run the following commands one by one, which will create 5 clients listening on localhost's port 3001, 3002, 3003, 3004, and 3005. Their actual values will also be printed in the terminal so you can sum them up by hand.

  ```
  cargo run --bin client 1
  cargo run --bin client 2
  cargo run --bin client 3
  cargo run --bin client 4
  cargo run --bin client 5
  ```

* **Step 2: Create a server**

  Then we are gonna need a server. Open another terminal and run the following command, which will create a server listening on port 3100.

  ```
  cargo run --bin server 100
  ```

* **Step 3: Initialize and check the aggregate result**

  We are gonna use curl to send http requests.

  ```
  curl -X PUT -H "Content-Type: application/json" -d '{"port_list":[3001,3002,3003,3004,3005], "num_collaborators":5}' http://localhost:3100/initialize
  
  curl http://localhost:3100/aggregate
  ```

  and you will see the result. Compare them to the sum of initial values, and we will see they are the same

### Full Code

View at [Github](https://github.com/TianCal/secure_aggregation-tutorial/tree/http)
