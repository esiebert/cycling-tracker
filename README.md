# Cycling Tracker

This system was built for learning purposes, it covers 3 forms of gRPC endpoints:
- One request in, one response out
    - Saving a workout: returns a summary which includes averages from measurements provided
    - Training Plan: returns the training plan token to be used when running a workout plan
- One request in, multiple responses out
    - Loading workout measurements: measurements are sent over individually
- Multiple requests in, multiple responses out
    - Run workout plans: given a training plan, returns the target power that needs to be reached

## Architecture

Currently, the system exposes a gRPC API supporting basic authentication with session tokens over TLS.

Next steps:
- Finish fully implementing service features
- Database connections for logging and persisting workout data
- More tests
- Attaching logs to Grafana and provide tracing logs

## Running the system

1. Given that Rust is installed locally, and dependencies have been installed run:
```
make run
```

2. Run your favorite gRPC tool to make requests to the API. I recommend using [grpcui](https://github.com/fullstorydev/grpcui) and running it with:
```
make ui
```
