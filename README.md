# Rustopoly

<https://digipedia.dq.skoda.vwg/display/CISDD/Rustopoly>

## Deployment

### Running the executable

Print help information:
: `cargo run -p rustopoly-server -- --help`

Run with default arguments:
: `cargo run -p rustopoly-client`
default values: ip: `localhost`, port: `11111`

Run executable with some arguments:
: `cargo run -p rustopoly-server -- --ip 192.168.1.1 --port 22222`
undefined arguments will be set to the default value

### Running entire project

1. Run the server (default on `127.0.0.1:11111`):

    ``` bash
    cargo run -p rustopoly-server -- [--ip <IP>] [--port <PORT>]
    ```

    Make sure, the requested address/port can be assigned.

2. Run one or more clients (each in separate terminal):

    ``` bash
    cargo run -p rustopoly-client -- [--ip <IP>] [--port <PORT>]
    ```

    Make sure the server is running when spawning client(s).
    Make sure the client connects to server's socket address.

## Development

### Pre-commit checks

To install pre-commit checks with pipx run

``` bash
pipx run pre-commit install
```

To run pre-commit checks run:

``` bash
pipx run pre-commit run
```

to run on all files instead on changes add `--all-files` flag.

Alternatively create a virtual environment and install and run pre-commit locally.
