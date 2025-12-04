# MCP Wrapper

This is a HTTP component that wraps and routes requests to an MCP (Model Context Protocol) component. It provides HTTP routing for health checks, actions, and MCP protocol handling.

## Prerequisites

- `cargo` 1.82
- [`wash`](https://wasmcloud.com/docs/installation) 0.36.1
- `wasmtime` >=25.0.0 (if running with wasmtime)

## Building

```bash
wash build
```

## Routes

- `GET /health` - Health check endpoint
- `POST /actions` - Actions endpoint (Betty Blocks integration - currently mocked)
- `POST /mcp` - MCP protocol endpoint (delegates to MCP component)

## Running with wasmCloud

```shell
wash dev
```

Test the endpoints:

```shell
# Health check
curl http://127.0.0.1:8000/health

# Actions endpoint
curl -X POST http://127.0.0.1:8000/actions \
  -H "Content-Type: application/json" \
  -d '{"action_id": "test", "payload": {"input": "test data"}}'

# MCP endpoint
curl -X POST http://127.0.0.1:8000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "test", "params": {}}'
```

## Architecture

This component acts as an HTTP router that:
- Handles basic HTTP routes (health, actions)
- Delegates MCP protocol requests to an imported MCP component via WIT bindings
- Uses the WASI HTTP interface for protocol-level request/response handling

## Adding Capabilities

To learn how to extend this example with additional capabilities, see the [Adding Capabilities](https://wasmcloud.com/docs/tour/adding-capabilities?lang=rust) section of the wasmCloud documentation.
