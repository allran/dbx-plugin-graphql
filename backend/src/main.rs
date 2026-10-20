mod graphql;
mod session;

use std::sync::{Arc, Mutex};

use dbx_plugin_sdk::{
    PluginEmitter, PluginError, PluginHandler, PluginMetadata, PluginServer, RequestContext,
};
use serde_json::{json, Value};

use graphql::{execute_request, introspect, parse_headers_json};
use session::{ConnectionSession, SessionRegistry};

struct Plugin {
    sessions: Arc<Mutex<SessionRegistry>>,
}

impl Plugin {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(SessionRegistry::default())),
        }
    }

    fn with_sessions<R>(
        &self,
        f: impl FnOnce(&mut SessionRegistry) -> Result<R, PluginError>,
    ) -> Result<R, PluginError> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| PluginError::new(-32000, "Session registry is poisoned"))?;
        f(&mut sessions)
    }
}

impl PluginHandler for Plugin {
    fn handle(
        &self,
        _context: RequestContext,
        method: &str,
        params: Value,
        _emitter: &PluginEmitter,
    ) -> Result<Value, PluginError> {
        match method {
            "connection/test" => {
                let session = session_from_connection(&params)?;
                let result = introspect(&session)?;
                Ok(json!({
                    "success": true,
                    "message": format!(
                        "GraphQL endpoint OK ({} types).",
                        result.get("typeCount").and_then(Value::as_u64).unwrap_or(0)
                    )
                }))
            }
            "connection/connect" => {
                let connection_id = connection_id(&params)?.to_string();
                let session = session_from_connection(&params)?;
                self.with_sessions(|registry| {
                    registry.insert(connection_id, session);
                    Ok(json!({ "success": true }))
                })
            }
            "connection/disconnect" => {
                let connection_id = connection_id(&params)?.to_string();
                self.with_sessions(|registry| {
                    registry.remove(&connection_id);
                    Ok(json!({ "success": true }))
                })
            }
            "graphql/execute" => {
                let connection_id = require_str(&params, "connectionId")?;
                let query = require_str(&params, "query")?;
                let operation_name = optional_str(&params, "operationName");
                let variables = params.get("variables").cloned().unwrap_or(Value::Null);
                let override_headers = params.get("headers").cloned().unwrap_or(Value::Null);

                let session = self.with_sessions(|registry| {
                    registry
                        .get(connection_id)
                        .cloned()
                        .ok_or_else(|| {
                            PluginError::new(
                                -32602,
                                "Connection is not open. Open the GraphQL connection first.",
                            )
                        })
                })?;

                execute_request(&session, query, operation_name, variables, override_headers)
            }
            "graphql/introspect" => {
                let connection_id = require_str(&params, "connectionId")?;
                let session = self.with_sessions(|registry| {
                    registry
                        .get(connection_id)
                        .cloned()
                        .ok_or_else(|| {
                            PluginError::new(
                                -32602,
                                "Connection is not open. Open the GraphQL connection first.",
                            )
                        })
                })?;
                introspect(&session)
            }
            "graphql/ping" => Ok(json!({
                "ok": true,
                "plugin": "io.dbx.graphql",
                "connectionId": params.get("connectionId").cloned().unwrap_or(Value::Null)
            })),
            _ => Err(PluginError::method_not_found(method)),
        }
    }
}

fn connection_id(params: &Value) -> Result<&str, PluginError> {
    params
        .get("connection")
        .and_then(|connection| connection.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| PluginError::new(-32602, "Missing connection id"))
}

fn require_str<'a>(params: &'a Value, key: &str) -> Result<&'a str, PluginError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PluginError::new(-32602, format!("Missing {key}")))
}

fn optional_str<'a>(params: &'a Value, key: &str) -> Option<&'a str> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn session_from_connection(params: &Value) -> Result<ConnectionSession, PluginError> {
    let connection = params
        .get("connection")
        .ok_or_else(|| PluginError::new(-32602, "Missing connection"))?;

    let endpoint = connection
        .pointer("/external_config/endpoint")
        .or_else(|| connection.pointer("/config/endpoint"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PluginError::new(-32602, "Missing endpoint URL"))?
        .to_string();

    url::Url::parse(&endpoint).map_err(|_| PluginError::new(-32602, "Invalid endpoint URL"))?;
    if !(endpoint.starts_with("https://") || endpoint.starts_with("http://")) {
        return Err(PluginError::new(
            -32602,
            "Endpoint must start with http:// or https://",
        ));
    }

    let auth_token = connection
        .pointer("/connection_secrets/auth_token")
        .or_else(|| connection.pointer("/secrets/auth_token"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let extra_headers_raw = connection
        .pointer("/external_config/extra_headers")
        .or_else(|| connection.pointer("/config/extra_headers"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let extra_headers = parse_headers_json(extra_headers_raw)?;

    Ok(ConnectionSession {
        endpoint,
        auth_token,
        extra_headers,
    })
}

fn main() -> std::io::Result<()> {
    let metadata =
        PluginMetadata::new("io.dbx.graphql", env!("CARGO_PKG_VERSION")).with_capability("connections");
    PluginServer::new(metadata, Plugin::new()).serve()
}
