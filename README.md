# Cycling Tracker

This system was built for learning purposes, it covers four kinds of service methods:
- Simple RPC
    - Saving a workout: returns a summary which includes averages from measurements provided
- Server-side streaming RPC
    - Loading workout measurements: measurements are sent over individually
- Client-side streaming RPC
    - Recording a workout: measurements are sent over time, and server returns a workout summary
- Bidirectional streaming RPC
    - Get accumulating averages: provides an output stream of updated averages for every measurement streamed in

## Running the system

1. Setup development environment with:
```
make setup-env-linux
# Or
make setup-env-macos
```
2. Given that dependencies have been installed, run:
```
make run
```

3. Run your favorite gRPC tool to make requests to the API. I recommend using [grpcui](https://github.com/fullstorydev/grpcui) and running it with:
```
make ui
```
## Implementing a client

In order to implement a client to this service, you will require the protobuf files which can be found [here](https://github.com/esiebert/cycling-tracker/blob/master/proto/cyclingtracker.proto).
