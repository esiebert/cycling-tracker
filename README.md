# Cycling Tracker

This system was built for learning purposes, it simulates a cycling workout tracker which is able to:
- Save and load workouts
- Create workout plans
- Run workout plans

## Architecture

The system has a gRPC API supporting basic authentication with session tokens over TLS, connected to a database, and implementing the actor design pattern.

## Running the system

1. Given that Rust is installed locally, run:
```
make run
```

2. Run your favorite gRPC tool to make requests to the API. I recommend using [grpcui](https://github.com/fullstorydev/grpcui) and running it with:
```
make ui
```