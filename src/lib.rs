use bindings::wasi::http::types::{
    Fields, IncomingRequest, Method, OutgoingBody, OutgoingResponse, ResponseOutparam,
};
use matchit::{Match, Router};

pub mod bindings {
    wit_bindgen::generate!({ generate_all });
}

use crate::bindings::exports::wasi::http::incoming_handler::Guest;
use crate::bindings::wasmcloud::mcp::mcp_handler::mcp_handle;
// use crate::bindings::betty_blocks::types::actions::{Input, Payload, call, health};

struct Component;

#[derive(serde::Deserialize, Debug)]
struct PayloadWrapper {
    input: String,
}

#[derive(serde::Deserialize, Debug)]
struct InputWrapper {
    action_id: String,
    payload: PayloadWrapper,
}

// 2**24 = 16mb
const MAX_READ: u64 = 2u64.pow(24);

#[derive(Debug)]
enum Routes {
    Health,
    Actions,
    Mcp,
}

enum Error {
    InvalidInput(String),
    FailedToReadBody(String),
    ActionCallFailed(String),
    HealthCheckFailed(String),
    McpForwardFailed(String),
    DelegateToMcp,
}

enum HandleResult {
    Response(HttpResponse),
    DelegateToMcp,
}

struct HttpResponse {
    status: u16,
    body: String,
}

impl HttpResponse {
    fn new(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }
}

fn send_response(response_out: ResponseOutparam, response: HttpResponse) {
    let headers = Fields::new();
    let outgoing_response = OutgoingResponse::new(headers);
    outgoing_response.set_status_code(response.status).unwrap();

    let response_body = outgoing_response.body().unwrap();
    ResponseOutparam::set(response_out, Ok(outgoing_response));

    let output_stream = response_body.write().unwrap();
    output_stream
        .blocking_write_and_flush(response.body.as_bytes())
        .unwrap();
    drop(output_stream);
    OutgoingBody::finish(response_body, None).unwrap();
}

fn inner_handle(request: &IncomingRequest) -> Result<HandleResult, Error> {
    // Setup router
    let mut router = Router::new();
    router.insert("/health", Routes::Health).unwrap();
    router.insert("/actions", Routes::Actions).unwrap();
    router.insert("/mcp", Routes::Mcp).unwrap();

    let path_with_query = request.path_with_query().unwrap_or_default();
    let path = path_with_query.split('?').next().unwrap_or("/");
    let method = request.method();

    match (&method, router.at(path)) {
        // Health check route
        (
            &Method::Get,
            Ok(Match {
                value: Routes::Health,
                ..
            }),
        ) => {
            // Comment out the actual health check call
            // let health_status = health().map_err(Error::HealthCheckFailed)?;
            // Ok(HandleResult::Response(HttpResponse::new(200, health_status)))

            // Return a simple health status instead
            Ok(HandleResult::Response(HttpResponse::new(200, "healthy")))
        }

        // MCP route: delegate to MCP handler
        (
            &Method::Post,
            Ok(Match {
                value: Routes::Mcp, ..
            }),
        ) => Ok(HandleResult::DelegateToMcp),

        // Actions route
        (
            &Method::Post,
            Ok(Match {
                value: Routes::Actions,
                ..
            }),
        ) => {
            let body = request.consume().unwrap();
            let input_stream = body.stream().unwrap();

            let body_bytes = input_stream
                .blocking_read(MAX_READ)
                .map_err(|e| Error::FailedToReadBody(format!("{:?}", e)))?;

            let input_wrapper = serde_json::from_slice::<InputWrapper>(&body_bytes)
                .map_err(|e| Error::InvalidInput(e.to_string()))?;

            // Comment out the actual action call
            // let input = Input {
            //     action_id: input_wrapper.action_id,
            //     payload: Payload {
            //         input: input_wrapper.payload.input,
            //     },
            // };
            // let result = call(&input).map_err(Error::ActionCallFailed)?;
            // Ok(HandleResult::Response(HttpResponse::new(200, result.result)))

            // Return a mock response instead
            let response_body = format!(
                "Action {} would be called with payload: {}",
                input_wrapper.action_id, input_wrapper.payload.input
            );
            Ok(HandleResult::Response(HttpResponse::new(
                200,
                response_body,
            )))
        }

        // Default: route not found
        _ => Ok(HandleResult::Response(HttpResponse::new(
            404,
            format!("Route not found: {:?} {}", method, path),
        ))),
    }
}

impl Guest for Component {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        // Check route first without consuming the request
        match inner_handle(&request) {
            Ok(HandleResult::Response(response)) => {
                send_response(response_out, response);
            }
            Ok(HandleResult::DelegateToMcp) => {
                // MCP route: delegate directly to MCP handler
                mcp_handle(request, response_out);
            }
            Err(e) => {
                let (status, message) = match e {
                    Error::InvalidInput(msg) => (400, msg),
                    Error::FailedToReadBody(msg) => (500, msg),
                    Error::ActionCallFailed(msg) => (400, msg),
                    Error::HealthCheckFailed(msg) => (400, msg),
                    Error::McpForwardFailed(msg) => (500, msg),
                    Error::DelegateToMcp => {
                        (500, "Internal error: MCP delegation failed".to_string())
                    }
                };
                send_response(response_out, HttpResponse::new(status, message));
            }
        }
    }
}

bindings::export!(Component with_types_in bindings);
