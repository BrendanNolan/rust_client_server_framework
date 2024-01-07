# Generic Application-Server Framework

To use this framework, you must provide only the data processing function and a very small amount of
plumbing code.

For example, a user who wants a server that provides weather status must only do the following:

```rust
let (tx_shutdown, rx_shutdown) = watch::channel(());
let server_task = tokio::spawn(server_runner::run_server(
    "<IP>:<Port>",
    weather_provider,  // User's data processing function
    ShutdownListener::new(rx_shutdown),
));
handle_shutdown(tx_shutdown).await;
let _ = server_task.await;
```

This repo contains a presentation of the possible use cases, the goals, and the design choices for
this project. Please see `project_description/project_description.pdf`.
