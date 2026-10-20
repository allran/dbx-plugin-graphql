use std::collections::HashMap;
use std::time::Instant;

use dbx_plugin_sdk::PluginError;
use serde_json::{json, Value};

use crate::session::ConnectionSession;

const INTROSPECTION_QUERY: &str = r#"
query IntrospectionQuery {
  __schema {
    queryType { name }
    mutationType { name }
    subscriptionType { name }
    types {
      ...FullType
    }
  }
}

fragment FullType on __Type {
  kind
  name
  description
  fields(includeDeprecated: true) {
    name
    description
    args {
      ...InputValue
    }
    type {
      ...TypeRef
    }
    isDeprecated
    deprecationReason
  }
  inputFields {
    ...InputValue
  }
  interfaces {
    ...TypeRef
  }
  enumValues(includeDeprecated: true) {
    name
    description
    isDeprecated
    deprecationReason
  }
  possibleTypes {
    ...TypeRef
  }
}

fragment InputValue on __InputValue {
  name
  description
  type { ...TypeRef }
  defaultValue
}

fragment TypeRef on __Type {
  kind
  name
  ofType {
    kind
    name
    ofType {
      kind
      name
      ofType {
        kind
        name
        ofType {
          kind
          name
          ofType {
            kind
            name
            ofType {
              kind
              name
              ofType {
                kind
                name
              }
            }
          }
        }
      }
    }
  }
}
"#;

pub fn parse_headers_json(raw: &str) -> Result<HashMap<String, String>, PluginError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(HashMap::new());
    }
    let value: Value = serde_json::from_str(trimmed)
        .map_err(|err| PluginError::new(-32602, format!("Invalid extra headers JSON: {err}")))?;
    let object = value
        .as_object()
        .ok_or_else(|| PluginError::new(-32602, "Extra headers must be a JSON object"))?;
    let mut headers = HashMap::new();
    for (key, item) in object {
        let header_value = item
            .as_str()
            .ok_or_else(|| {
                PluginError::new(
                    -32602,
                    format!("Header `{key}` must be a string value"),
                )
            })?
            .to_string();
        headers.insert(key.clone(), header_value);
    }
    Ok(headers)
}

pub fn execute_request(
    session: &ConnectionSession,
    query: &str,
    operation_name: Option<&str>,
    variables: Value,
    override_headers: Value,
) -> Result<Value, PluginError> {
    let mut body = json!({ "query": query });
    if let Some(name) = operation_name {
        body["operationName"] = Value::String(name.to_string());
    }
    if !variables.is_null() {
        body["variables"] = variables;
    }

    let started = Instant::now();
    let (status, response_headers, response_body) =
        post_graphql(session, &body, override_headers)?;
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let http_ok = (200..300).contains(&status);
    let has_graphql_errors = response_body
        .get("errors")
        .and_then(Value::as_array)
        .map(|items| !items.is_empty())
        .unwrap_or(false);

    Ok(json!({
        "ok": http_ok && !has_graphql_errors,
        "httpOk": http_ok,
        "status": status,
        "elapsedMs": elapsed_ms,
        "headers": response_headers,
        "body": response_body,
        "hasGraphqlErrors": has_graphql_errors
    }))
}

pub fn introspect(session: &ConnectionSession) -> Result<Value, PluginError> {
    let result = execute_request(
        session,
        INTROSPECTION_QUERY,
        Some("IntrospectionQuery"),
        Value::Null,
        Value::Null,
    )?;
    let status = result
        .get("status")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if !(200..300).contains(&status) {
        return Err(PluginError::new(
            -32000,
            format!("Introspection HTTP {status}"),
        ));
    }

    let body = result.get("body").cloned().unwrap_or(Value::Null);
    if let Some(errors) = body.get("errors") {
        if errors.as_array().map(|items| !items.is_empty()).unwrap_or(false) {
            return Err(PluginError::new(
                -32000,
                format!("Introspection GraphQL errors: {errors}"),
            ));
        }
    }

    let schema = body
        .pointer("/data/__schema")
        .cloned()
        .ok_or_else(|| PluginError::new(-32000, "Introspection response missing __schema"))?;

    let type_count = schema
        .get("types")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);

    Ok(json!({
        "ok": true,
        "status": status,
        "elapsedMs": result.get("elapsedMs").cloned().unwrap_or(json!(0)),
        "typeCount": type_count,
        "schema": schema
    }))
}

fn post_graphql(
    session: &ConnectionSession,
    body: &Value,
    override_headers: Value,
) -> Result<(u16, Value, Value), PluginError> {
    let mut request = ureq::post(&session.endpoint)
        .set("Content-Type", "application/json")
        .set("Accept", "application/json");

    if let Some(token) = session.auth_token.as_deref() {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }

    for (key, value) in &session.extra_headers {
        request = request.set(key, value);
    }

    if let Some(object) = override_headers.as_object() {
        for (key, value) in object {
            if let Some(text) = value.as_str() {
                request = request.set(key, text);
            }
        }
    }

    match request.send_json(body.clone()) {
        Ok(response) => Ok(read_response(response)),
        Err(ureq::Error::Status(_code, response)) => Ok(read_response(response)),
        Err(ureq::Error::Transport(transport)) => {
            Err(PluginError::new(-32000, format!("Transport error: {transport}")))
        }
    }
}

fn read_response(response: ureq::Response) -> (u16, Value, Value) {
    let status = response.status();
    let mut headers = serde_json::Map::new();
    for name in response.headers_names() {
        if let Some(value) = response.header(&name) {
            headers.insert(name, Value::String(value.to_string()));
        }
    }
    let response_headers = Value::Object(headers);
    let response_body: Value = response.into_json().unwrap_or_else(|_| {
        json!({ "message": "Non-JSON response body" })
    });
    (status, response_headers, response_body)
}
