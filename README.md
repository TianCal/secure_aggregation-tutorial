# How to demo

## Step 1: Create clients

For convenience, let's just create 5 clients. Open five different terminals and run the following commands one by one, which will create 5 clients listening on localhost's port 3001, 3002, 3003, 3004, and 3005. Their actual values will also be printed in the terminal so you can sum them up by hand.
```console
cargo run --bin client 1
cargo run --bin client 2
cargo run --bin client 3
cargo run --bin client 4
cargo run --bin client 5
```

## Step 2: Create a server
Then we are gonna need a server. Open another terminal and run the following command, which will create a server listening on port 3100.
```console
cargo run --bin server 100
```

## Step 3: Initialize and Check the aggregate result
We are gonna use curl to send http requests.
```console
curl -X PUT -H "Content-Type: application/json" -d '{"port_list":[3001,3002,3003,3004,3005], "num_collaborators":5}' http://localhost:3100/initialize

curl http://localhost:3100/aggregate
```
and you will see the result