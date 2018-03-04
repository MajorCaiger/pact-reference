//! The `models` module provides all the structures required to model a Pact.

use std::collections::HashMap;
use std::collections::BTreeMap;
use serde_json;
use serde_json::Value;
use hex::FromHex;
use super::strip_whitespace;
use regex::Regex;
use semver::Version;
use itertools::Itertools;
use std::io::{self, Error, ErrorKind};
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use hyper::client::Client;
use std::str;
use base64::{encode, decode};

/// Version of the library
pub const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

/// Enum defining the pact specification versions supported by the library
#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum PactSpecification {
    /// Unknown or unsupported specification version
    Unknown,
    /// First version of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-1)
    V1,
    /// Second version of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-1.1)
    V1_1,
    /// Version two of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-2)
    V2,
    /// Version three of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-3)
    V3
}

impl PactSpecification {
    /// Returns the semantic version string of the specification version.
    pub fn version_str(&self) -> String {
        match *self {
            PactSpecification::V1 => s!("1.0.0"),
            PactSpecification::V1_1 => s!("1.1.0"),
            PactSpecification::V2 => s!("2.0.0"),
            PactSpecification::V3 => s!("3.0.0"),
            _ => s!("unknown")
        }
    }

    /// Returns a descriptive string of the specification version.
    pub fn to_string(&self) -> String {
        match *self {
            PactSpecification::V1 => s!("V1"),
            PactSpecification::V1_1 => s!("V1.1"),
            PactSpecification::V2 => s!("V2"),
            PactSpecification::V3 => s!("V3"),
            _ => s!("unknown")
        }
    }
}

/// Struct that defines the consumer of the pact.
#[derive(Debug, Clone)]
pub struct Consumer {
    /// Each consumer should have a unique name to identify it.
    pub name: String
}

impl Consumer {
    /// Builds a `Consumer` from the `Json` struct.
    pub fn from_json(pact_json: &Value) -> Consumer {
        let val = match pact_json.get("name") {
            Some(v) => match v.clone() {
                Value::String(s) => s,
                _ => v.to_string()
            },
            None => "consumer".to_string()
        };
        Consumer { name: val.clone() }
    }

    /// Converts this `Consumer` to a `Value` struct.
    pub fn to_json(&self) -> Value {
        json!({ s!("name") : json!(self.name.clone()) })
    }
}

/// Struct that defines a provider of a pact.
#[derive(Debug, Clone)]
pub struct Provider {
    /// Each provider should have a unique name to identify it.
    pub name: String
}

impl Provider {
    /// Builds a `Provider` from a `Value` struct.
    pub fn from_json(pact_json: &Value) -> Provider {
        let val = match pact_json.get("name") {
            Some(v) => match v.clone() {
                Value::String(s) => s,
                _ => v.to_string()
            },
            None => "provider".to_string()
        };
        Provider { name: val.clone() }
    }

    /// Converts this `Provider` to a `Value` struct.
    pub fn to_json(&self) -> Value {
        json!({ s!("name") : json!(self.name.clone()) })
    }
}

/// Enum that defines the four main states that a body of a request and response can be in a pact
/// file.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum OptionalBody {
    /// A body is missing if it is not present in the pact file
    Missing,
    /// An empty body that is present in the pact file.
    Empty,
    /// A JSON body that is the null value. This state is to protect other language implementations
    /// from null values. It is treated as `Empty`.
    Null,
    /// A non-empty body that is present in the pact file.
    Present(Vec<u8>)
}

impl OptionalBody {

    /// If the body is present in the pact file and not empty or null.
    pub fn is_present(&self) -> bool {
        match *self {
            OptionalBody::Present(_) => true,
            _ => false
        }
    }

    /// Returns the body if present, otherwise returns the empty Vec.
    pub fn value(&self) -> Vec<u8> {
        match *self {
            OptionalBody::Present(ref s) => s.clone(),
            _ => vec![]
        }
    }

  /// Returns the body if present as a string, otherwise returns the empty string.
  pub fn str_value(&self) -> &str {
    match *self {
      OptionalBody::Present(ref s) => str::from_utf8(s).unwrap_or(""),
      _ => ""
    }
  }
}

lazy_static! {
    static ref XMLREGEXP: Regex = Regex::new(r"^\s*<\?xml\s*version.*").unwrap();
    static ref HTMLREGEXP: Regex = Regex::new(r"^\s*(<!DOCTYPE)|(<HTML>).*").unwrap();
    static ref JSONREGEXP: Regex = Regex::new(r#"^\s*(true|false|null|[0-9]+|"\w*|\{\s*(}|"\w+)|\[\s*)"#).unwrap();
    static ref XMLREGEXP2: Regex = Regex::new(r#"^\s*<\w+\s*(:\w+=["”][^"”]+["”])?.*"#).unwrap();

    static ref JSON_CONTENT_TYPE: Regex = Regex::new("application/.*json.*").unwrap();
    static ref XML_CONTENT_TYPE: Regex = Regex::new("application/.*xml").unwrap();
}

/// Enumeration of general content types
#[derive(PartialEq, Debug, Clone, Eq)]
pub enum DetectedContentType {
    /// Json content types
    Json,
    /// XML content types
    Xml,
    /// All other content types
    Text
}

/// Enumeration of the types of differences between requests and responses
#[derive(PartialEq, Debug, Clone, Eq)]
pub enum DifferenceType {
  /// Methods differ
  Method,
  /// Paths differ
  Path,
  /// Headers differ
  Headers,
  /// Query parameters differ
  QueryParameters,
  /// Bodies differ
  Body,
  /// Matching Rules differ
  MatchingRules,
  /// Response status differ
  Status
}

#[macro_use] pub mod matchingrules;

/// Trait to specify an HTTP part of a message. It encapsulates the shared parts of a request and
/// response.
pub trait HttpPart {
    /// Returns the headers of the HTTP part.
    fn headers(&self) -> &Option<HashMap<String, String>>;
    /// Returns the body of the HTTP part.
    fn body(&self) -> &OptionalBody;
    /// Returns the matching rules of the HTTP part.
    fn matching_rules(&self) -> &matchingrules::MatchingRules;

    /// Determine the content type of the HTTP part. If a `Content-Type` header is present, the
    /// value of that header will be returned. Otherwise, the body will be inspected.
    fn content_type(&self) -> String {
        match *self.headers() {
            Some(ref h) => match h.iter().find(|kv| kv.0.to_lowercase() == s!("content-type")) {
                Some(kv) => match strip_whitespace::<Vec<&str>>(kv.1, ";").first() {
                    Some(v) => s!(*v),
                    None => self.detect_content_type()
                },
                None => self.detect_content_type()
            },
            None => self.detect_content_type()
        }
    }

    /// Tries to detect the content type of the body by matching some regular expressions against
    /// the first 32 characters. Default to `text/plain` if no match is found.
    fn detect_content_type(&self) -> String {
        match *self.body() {
            OptionalBody::Present(ref body) => {
                let s: String = match str::from_utf8(body) {
                  Ok(s) => s.to_string(),
                  Err(_) => String::new()
                };
                debug!("Detecting content type from contents: '{}'", s);
                if XMLREGEXP.is_match(s.as_str()) {
                    s!("application/xml")
                } else if HTMLREGEXP.is_match(s.to_uppercase().as_str()) {
                    s!("text/html")
                } else if XMLREGEXP2.is_match(s.as_str()) {
                    s!("application/xml")
                } else if JSONREGEXP.is_match(s.as_str()) {
                    s!("application/json")
                } else {
                    s!("text/plain")
                }
            },
            _ => s!("text/plain")
        }
    }

    /// Returns the general content type (ignoring subtypes)
    fn content_type_enum(&self) -> DetectedContentType {
        let content_type = self.content_type();
        if JSON_CONTENT_TYPE.is_match(&content_type[..]) {
            DetectedContentType::Json
        } else if XML_CONTENT_TYPE.is_match(&content_type[..]) {
            DetectedContentType::Xml
        } else {
            DetectedContentType::Text
        }
    }
}

/// Struct that defines the request.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq)]
pub struct Request {
    /// Request method
    pub method: String,
    /// Request path
    pub path: String,
    /// Request query string
    pub query: Option<HashMap<String, Vec<String>>>,
    /// Request headers
    pub headers: Option<HashMap<String, String>>,
    /// Request body
    pub body: OptionalBody,
    /// Request matching rules
    pub matching_rules: matchingrules::MatchingRules
}

impl HttpPart for Request {
    fn headers(&self) -> &Option<HashMap<String, String>> {
        &self.headers
    }

    fn body(&self) -> &OptionalBody {
        &self.body
    }

    fn matching_rules(&self) -> &matchingrules::MatchingRules {
        &self.matching_rules
    }
}

impl Hash for Request {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.method.hash(state);
        self.path.hash(state);
        if self.query.is_some() {
            for (k, v) in self.query.clone().unwrap() {
                k.hash(state);
                v.hash(state);
            }
        }
        if self.headers.is_some() {
            for (k, v) in self.headers.clone().unwrap() {
                k.hash(state);
                v.hash(state);
            }
        }
        self.body.hash(state);
        self.matching_rules.hash(state);
    }
}

fn headers_from_json(request: &Value) -> Option<HashMap<String, String>> {
    match request.get("headers") {
        Some(v) => match *v {
            Value::Object(ref m) => Some(m.iter().map(|(key, val)| {
                match val {
                    &Value::String(ref s) => (key.clone(), s.clone()),
                    _ => (key.clone(), val.to_string())
                }
            }).collect()),
            _ => None
        },
        None => None
    }
}

fn headers_to_json(headers: &HashMap<String, String>) -> Value {
    json!(headers.iter().fold(BTreeMap::new(), |mut map, kv| {
        map.insert(kv.0.clone(), Value::String(kv.1.clone()));
        map
    }))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum JsonParsable {
    JsonStringValue(String),
    KeyValue(HashMap<String, Value>)
}

fn body_from_json(request: &Value, fieldname: &str, headers: &Option<HashMap<String, String>>) -> OptionalBody {
    let content_type = match headers {
        &Some(ref h) => match h.iter().find(|kv| kv.0.to_lowercase() == s!("content-type")) {
            Some(kv) => {
                match strip_whitespace::<Vec<&str>>(&kv.1, ";").first() {
                    Some(v) => Some(v.to_lowercase()),
                    None => None
                }
            },
            None => None
        },
        &None => None
    };

    match request.get(fieldname) {
        Some(v) => match *v {
            Value::String(ref s) => {
                if s.is_empty() {
                  OptionalBody::Empty
                } else {
                  let content_type = content_type.unwrap_or(s!("text/plain"));
                  if JSON_CONTENT_TYPE.is_match(&content_type) {
                    match serde_json::from_str::<JsonParsable>(&s) {
                      Ok(_) => OptionalBody::Present(s.clone().into()),
                      Err(_) => OptionalBody::Present(format!("\"{}\"", s).into())
                    }
                  } else if content_type.starts_with("text/") {
                    OptionalBody::Present(s.clone().into())
                  } else {
                    match decode(s) {
                      Ok(bytes) => OptionalBody::Present(bytes.clone()),
                      Err(_) => OptionalBody::Present(s.clone().into())
                    }
                  }
                }
            },
            Value::Null => OptionalBody::Null,
            _ => OptionalBody::Present(v.to_string().into())
        },
        None => OptionalBody::Missing
    }
}

/// Converts a query string map into a query string
pub fn build_query_string(query: HashMap<String, Vec<String>>) -> String {
    query.into_iter()
        .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
        .iter()
        .flat_map(|kv| {
            kv.1.iter()
                .map(|v| format!("{}={}", kv.0, encode_query(v)))
                .collect_vec()
        })
        .join("&")
}

impl Request {
    /// Builds a `Request` from a `Value` struct.
    pub fn from_json(request_json: &Value, spec_version: &PactSpecification) -> Request {
        let method_val = match request_json.get("method") {
            Some(v) => match *v {
                Value::String(ref s) => s.to_uppercase(),
                _ => v.to_string().to_uppercase()
            },
            None => "GET".to_string()
        };
        let path_val = match request_json.get("path") {
            Some(v) => match *v {
                Value::String(ref s) => s.clone(),
                _ => v.to_string()
            },
            None => "/".to_string()
        };
        let query_val = match request_json.get("query") {
            Some(v) => match *v {
                Value::String(ref s) => parse_query_string(s),
                _ => {
                    warn!("Only string versions of request query strings are supported with specification version {}, ignoring.",
                        spec_version.to_string());
                    None
                }
            },
            None => None
        };
        let headers = headers_from_json(request_json);
        Request {
            method: method_val,
            path: path_val,
            query: query_val,
            headers: headers.clone(),
            body: body_from_json(request_json, "body", &headers),
            matching_rules: matchingrules::matchers_from_json(request_json, &Some(s!("requestMatchingRules")))
        }
    }

    /// Converts this `Request` to a `Value` struct.
    pub fn to_json(&self) -> Value {
        let mut json = json!({
            s!("method") : Value::String(self.method.to_uppercase()),
            s!("path") : Value::String(self.path.clone())
        });
        {
            let map = json.as_object_mut().unwrap();
            if self.query.is_some() {
                map.insert(s!("query"), Value::String(build_query_string(self.query.clone().unwrap())));
            }
            if self.headers.is_some() {
                map.insert(s!("headers"), headers_to_json(&self.headers.clone().unwrap()));
            }
            match self.body {
                OptionalBody::Present(ref body) => {
                    if self.content_type() == "application/json" {
                        match serde_json::from_slice(body) {
                            Ok(json_body) => { map.insert(s!("body"), json_body); },
                            Err(err) => {
                                warn!("Failed to parse json body: {}", err);
                                map.insert(s!("body"), Value::String(encode(body)));
                            }
                        }
                    } else {
                      match str::from_utf8(body) {
                        Ok(s) => map.insert(s!("body"), Value::String(s.to_string())),
                        Err(_) => map.insert(s!("body"), Value::String(encode(body)))
                      };
                    }
                },
                OptionalBody::Empty => { map.insert(s!("body"), Value::String(s!(""))); },
                OptionalBody::Missing => (),
                OptionalBody::Null => { map.insert(s!("body"), Value::Null); }
            }
            if self.matching_rules.is_not_empty() {
                map.insert(s!("matchingRules"), matchingrules::matchers_to_json(
                &self.matching_rules.clone(), &PactSpecification::V2));
            }
        }
        json
    }

    /// Returns the default request: a GET request to the root.
    pub fn default_request() -> Request {
        Request {
            method: s!("GET"),
            path: s!("/"),
            query: None,
            headers: None,
            body: OptionalBody::Missing,
            matching_rules: matchingrules::MatchingRules::default()
        }
    }

    /// Return a description of all the differences from the other request
    pub fn differences_from(&self, other: &Request) -> Vec<(DifferenceType, String)> {
        let mut differences = vec![];
        if self.method != other.method {
            differences.push((DifferenceType::Method, format!("Request method {} != {}", self.method, other.method)));
        }
        if self.path != other.path {
            differences.push((DifferenceType::Path, format!("Request path {} != {}", self.path, other.path)));
        }
        if self.query != other.query {
            differences.push((DifferenceType::QueryParameters, format!("Request query {:?} != {:?}", self.query, other.query)));
        }
        if self.headers != other.headers {
            differences.push((DifferenceType::Headers, format!("Request headers {:?} != {:?}", self.headers, other.headers)));
        }
        if self.body != other.body {
            differences.push((DifferenceType::Body, format!("Request body '{:?}' != '{:?}'", self.body, other.body)));
        }
        if self.matching_rules != other.matching_rules {
            differences.push((DifferenceType::MatchingRules, format!("Request matching rules {:?} != {:?}", self.matching_rules, other.matching_rules)));
        }
        differences
    }
}

/// Struct that defines the response.
#[derive(PartialEq, Debug, Clone, Eq)]
pub struct Response {
    /// Response status
    pub status: u16,
    /// Response headers
    pub headers: Option<HashMap<String, String>>,
    /// Response body
    pub body: OptionalBody,
    /// Response matching rules
    pub matching_rules: matchingrules::MatchingRules
}

impl Response {

    /// Build a `Response` from a `Value` struct.
    pub fn from_json(response: &Value, _: &PactSpecification) -> Response {
        let status_val = match response.get("status") {
            Some(v) => v.as_u64().unwrap() as u16,
            None => 200
        };
        let headers = headers_from_json(response);
        Response {
            status: status_val,
            headers: headers.clone(),
            body: body_from_json(response, "body", &headers),
            matching_rules:  matchingrules::matchers_from_json(response, &Some(s!("responseMatchingRules")))
        }
    }

    /// Returns a default response: Status 200
    pub fn default_response() -> Response {
        Response {
            status: 200,
            headers: None,
            body: OptionalBody::Missing,
            matching_rules: matchingrules::MatchingRules::default()
        }
    }

    /// Converts this response to a `Value` struct.
    pub fn to_json(&self) -> Value {
        let mut json = json!({
            s!("status") : json!(self.status)
        });
        {
            let map = json.as_object_mut().unwrap();
            if self.headers.is_some() {
                map.insert(s!("headers"), headers_to_json(&self.headers.clone().unwrap()));
            }
            match self.body {
                OptionalBody::Present(ref body) => {
                    if self.content_type() == "application/json" {
                        match serde_json::from_slice(body) {
                            Ok(json_body) => { map.insert(s!("body"), json_body); },
                            Err(err) => {
                                warn!("Failed to parse json body: {}", err);
                                map.insert(s!("body"), Value::String(encode(body)));
                            }
                        }
                    } else {
                      match str::from_utf8(body) {
                        Ok(s) => map.insert(s!("body"), Value::String(s.to_string())),
                        Err(_) => map.insert(s!("body"), Value::String(encode(body)))
                      };
                    }
                },
                OptionalBody::Empty => { map.insert(s!("body"), Value::String(s!(""))); },
                OptionalBody::Missing => (),
                OptionalBody::Null => { map.insert(s!("body"), Value::Null); }
            }
            if self.matching_rules.is_not_empty() {
                map.insert(s!("matchingRules"), matchingrules::matchers_to_json(
              &self.matching_rules.clone(), &PactSpecification::V2));
            }
        }
        json
    }

    /// Return a description of all the differences from the other response
    pub fn differences_from(&self, other: &Response) -> Vec<(DifferenceType, String)> {
        let mut differences = vec![];
        if self.status != other.status {
            differences.push((DifferenceType::Status, format!("Response status {} != {}", self.status, other.status)));
        }
        if self.headers != other.headers {
            differences.push((DifferenceType::Headers, format!("Response headers {:?} != {:?}", self.headers, other.headers)));
        }
        if self.body != other.body {
            differences.push((DifferenceType::Body, format!("Response body '{:?}' != '{:?}'", self.body, other.body)));
        }
        if self.matching_rules != other.matching_rules {
            differences.push((DifferenceType::MatchingRules, format!("Response matching rules {:?} != {:?}", self.matching_rules, other.matching_rules)));
        }
        differences
    }
}

impl HttpPart for Response {
    fn headers(&self) -> &Option<HashMap<String, String>> {
        &self.headers
    }

    fn body(&self) -> &OptionalBody {
        &self.body
    }

    fn matching_rules(&self) -> &matchingrules::MatchingRules {
        &self.matching_rules
    }
}

impl Hash for Response {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.status.hash(state);
        if self.headers.is_some() {
            for (k, v) in self.headers.clone().unwrap() {
                k.hash(state);
                v.hash(state);
            }
        }
        self.body.hash(state);
        self.matching_rules.hash(state);
    }
}

/// Struct that defined an interaction conflict
#[derive(Debug, Clone)]
pub struct PactConflict {
    /// Description of the interactions
    pub interaction: String,
    /// Conflict description
    pub description: String
}

/// Struct that defines an interaction (request and response pair)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Interaction {
    /// Description of this interaction. This needs to be unique in the pact file.
    pub description: String,
    /// Optional provider state for the interaction.
    /// See http://docs.pact.io/documentation/provider_states.html for more info on provider states.
    pub provider_state: Option<String>,
    /// Request of the interaction
    pub request: Request,
    /// Response of the interaction
    pub response: Response
}

impl Interaction {
    /// Constructs an `Interaction` from the `Value` struct.
    pub fn from_json(index: usize, pact_json: &Value, spec_version: &PactSpecification) -> Interaction {
        let description = match pact_json.get("description") {
            Some(v) => match *v {
                Value::String(ref s) => s.clone(),
                _ => v.to_string()
            },
            None => format!("Interaction {}", index)
        };
        let provider_state = match pact_json.get("providerState").or(pact_json.get("provider_state")) {
            Some(v) => match *v {
                Value::String(ref s) => if s.is_empty() {
                    None
                } else {
                    Some(s.clone())
                },
                Value::Null => None,
                _ => Some(v.to_string())
            },
            None => None
        };
        let request = match pact_json.get("request") {
            Some(v) => Request::from_json(v, spec_version),
            None => Request::default_request()
        };
        let response = match pact_json.get("response") {
            Some(v) => Response::from_json(v, spec_version),
            None => Response::default_response()
        };
        Interaction {
             description,
             provider_state,
             request,
             response
        }
    }

    /// Converts this interaction to a `Value` struct.
    pub fn to_json(&self) -> Value {
        let mut value = json!({
            s!("description") : Value::String(self.description.clone()),
            s!("request") : self.request.to_json(),
            s!("response") : self.response.to_json()
        });
        if self.provider_state.is_some() {
            let map = value.as_object_mut().unwrap();
            map.insert(s!("providerState"), json!(self.provider_state.clone().unwrap()));
        }
        value
    }

    /// Returns list of conflicts if this interaction conflicts with the other interaction.
    ///
    /// Two interactions conflict if they have the same description and provider state, but they request and
    /// responses are not equal
    pub fn conflicts_with(&self, other: &Interaction) -> Vec<PactConflict> {
        if self.description == other.description && self.provider_state == other.provider_state {
            let mut conflicts = self.request.differences_from(&other.request).iter()
                .filter(|difference| match difference.0 {
                  DifferenceType::MatchingRules | DifferenceType::Body => false,
                  _ => true
                })
                .map(|difference| PactConflict { interaction: self.description.clone(), description: difference.1.clone() } )
                .collect::<Vec<PactConflict>>();
            for difference in self.response.differences_from(&other.response) {
              match difference.0 {
                DifferenceType::MatchingRules | DifferenceType::Body => (),
                _ => conflicts.push(PactConflict { interaction: self.description.clone(), description: difference.1.clone() })
              };
            }
            conflicts
        } else {
            vec![]
        }
    }

    /// Creates a default interaction
    pub fn default() -> Interaction {
        Interaction {
             description: s!("Default Interaction"),
             provider_state: None,
             request: Request::default_request(),
             response: Response::default_response()
        }
    }
}

pub mod message;

/// Struct that represents a pact between the consumer and provider of a service.
#[derive(Debug, Clone)]
pub struct Pact {
    /// Consumer side of the pact
    pub consumer: Consumer,
    /// Provider side of the pact
    pub provider: Provider,
    /// List of interactions between the consumer and provider.
    pub interactions: Vec<Interaction>,
    /// Metadata associated with this pact file.
    pub metadata: BTreeMap<String, BTreeMap<String, String>>,
    /// Specification version of this pact
    pub specification_version: PactSpecification
}

fn parse_meta_data(pact_json: &Value) -> BTreeMap<String, BTreeMap<String, String>> {
    match pact_json.get("metadata") {
        Some(v) => match *v {
            Value::Object(ref obj) => obj.iter().map(|(k, v)| {
                let val = match *v {
                    Value::Object(ref obj) => obj.iter().map(|(k, v)| {
                        match *v {
                            Value::String(ref s) => (k.clone(), s.clone()),
                            _ => (k.clone(), v.to_string())
                        }
                    }).collect(),
                    _ => btreemap!{}
                };
                (k.clone(), val)
            }).collect(),
            _ => btreemap!{}
        },
        None => btreemap!{}
    }
}

fn parse_interactions(pact_json: &Value, spec_version: PactSpecification) -> Vec<Interaction> {
    match pact_json.get("interactions") {
        Some(v) => match *v {
            Value::Array(ref array) => array.iter().enumerate().map(|(index, ijson)| {
                Interaction::from_json(index, ijson, &spec_version)
            }).collect(),
            _ => vec![]
        },
        None => vec![]
    }
}

fn determin_spec_version(file: &String, metadata: &BTreeMap<String, BTreeMap<String, String>>) -> PactSpecification {
    let specification = if metadata.get("pact-specification").is_none()
        { metadata.get("pactSpecification") } else { metadata.get("pact-specification") };
    match specification {
        Some(spec) => {
            match spec.get("version") {
                Some(ver) => match Version::parse(ver) {
                    Ok(ver) => match ver.major {
                        1 => match ver.minor {
                            0 => PactSpecification::V1,
                            1 => PactSpecification::V1_1,
                            _ => {
                                warn!("Unsupported specification version '{}' found in the metadata in the pact file {:?}, will try load it as a V2 specification", ver, file);
                                PactSpecification::Unknown
                            }
                        },
                        2 => PactSpecification::V2,
                        _ => {
                            warn!("Unsupported specification version '{}' found in the metadata in the pact file {:?}, will try load it as a V2 specification", ver, file);
                            PactSpecification::Unknown
                        }
                    },
                    Err(err) => {
                        warn!("Could not parse specification version '{}' found in the metadata in the pact file {:?}, assuming V2 specification - {}", ver, file, err);
                        PactSpecification::Unknown
                    }
                },
                None => {
                    warn!("No specification version found in the metadata in the pact file {:?}, assuming V2 specification", file);
                    PactSpecification::V2
                }
            }
        },
        None => {
            warn!("No metadata found in pact file {:?}, assuming V2 specification", file);
            PactSpecification::V2
        }
    }
}

impl Pact {

    /// Creates a `Pact` from a `Value` struct.
    pub fn from_json(file: &String, pact_json: &Value) -> Pact {
        let metadata = parse_meta_data(pact_json);
        let spec_version = determin_spec_version(file, &metadata);

        let consumer = match pact_json.get("consumer") {
            Some(v) => Consumer::from_json(v),
            None => Consumer { name: s!("consumer") }
        };
        let provider = match pact_json.get("provider") {
            Some(v) => Provider::from_json(v),
            None => Provider { name: s!("provider") }
        };
        Pact {
            consumer,
            provider,
            interactions: parse_interactions(pact_json, spec_version.clone()),
            metadata,
            specification_version: spec_version.clone()
        }
    }

    /// Converts this pact to a `Value` struct.
    pub fn to_json(&self) -> Value {
        json!({
            s!("consumer"): self.consumer.to_json(),
            s!("provider"): self.provider.to_json(),
            s!("interactions"): Value::Array(self.interactions.iter().map(|i| i.to_json()).collect()),
            s!("metadata"): json!(self.metadata_to_json())
        })
    }

    /// Creates a BTreeMap of the metadata of this pact.
    pub fn metadata_to_json(&self) -> BTreeMap<String, Value> {
        let mut md_map: BTreeMap<String, Value> = self.metadata.iter()
            .map(|(k, v)| {
                (k.clone(), json!(v.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<BTreeMap<String, String>>()))
            })
            .collect();
        md_map.insert(s!("pact-specification"), json!({"version" : PactSpecification::V2.version_str()}));

        md_map.insert(s!("pact-rust"), json!({"version" : s!(VERSION.unwrap_or("unknown"))}));
        md_map
    }

    /// Merges this pact with the other pact, and returns a new Pact with the interactions sorted.
    /// Returns an error if there is a merge conflict, which will occur if any interaction has the
    /// same description and provider state and the requests and responses are different.
    pub fn merge(&self, pact: &Pact) -> Result<Pact, String> {
        if self.consumer.name == pact.consumer.name && self.provider.name == pact.provider.name {
            let conflicts = iproduct!(self.interactions.clone(), pact.interactions.clone())
                .map(|i| i.0.conflicts_with(&i.1))
                .filter(|conflicts| !conflicts.is_empty())
                .collect::<Vec<Vec<PactConflict>>>();
            let num_conflicts = conflicts.len();
            if num_conflicts > 0 {
                warn!("The following conflicting interactions where found:");
                for interaction_conflicts in conflicts {
                    warn!(" Interaction '{}':", interaction_conflicts.first().unwrap().interaction);
                    for conflict in interaction_conflicts {
                        warn!("   {}", conflict.description);
                    }
                }
                Err(format!("Unable to merge pacts, as there were {} conflict(s) between the interactions",
                    num_conflicts))
            } else {
                Ok(Pact {
                    provider: self.provider.clone(),
                    consumer: self.consumer.clone(),
                    interactions: self.interactions.iter()
                        .chain(pact.interactions.iter())
                        .cloned()
                        .sorted_by(|a, b| {
                            let cmp = Ord::cmp(&a.provider_state, &b.provider_state);
                            if cmp == Ordering::Equal {
                                Ord::cmp(&a.description, &b.description)
                            } else {
                                cmp
                            }
                        }).into_iter()
                        .unique()
                        .collect(),
                    metadata: self.metadata.clone(),
                    specification_version: self.specification_version.clone()
                })
            }
        } else {
            Err(s!("Unable to merge pacts, as they have different consumers or providers"))
        }
    }

    /// Determins the default file name for the pact. This is based on the consumer and
    /// provider names.
    pub fn default_file_name(&self) -> String {
        format!("{}-{}.json", self.consumer.name, self.provider.name)
    }

    /// Reads the pact file and parses the resulting JSON into a `Pact` struct
    pub fn read_pact(file: &Path) -> io::Result<Pact> {
        let mut f = File::open(file)?;
        let pact_json = serde_json::from_reader(&mut f);
        match pact_json {
            Ok(ref json) => Ok(Pact::from_json(&format!("{:?}", file), json)),
            Err(err) => Err(Error::new(ErrorKind::Other, format!("Failed to parse Pact JSON - {}", err)))
        }
    }

    /// Reads the pact file from a URL and parses the resulting JSON into a `Pact` struct
    pub fn from_url(url: &String) -> Result<Pact, String> {
        let client = Client::new();
        match client.get(url).send() {
            Ok(mut res) => if res.status.is_success() {
                    let pact_json = serde_json::de::from_reader(&mut res);
                    match pact_json {
                        Ok(ref json) => Ok(Pact::from_json(url, json)),
                        Err(err) => Err(format!("Failed to parse Pact JSON - {}", err))
                    }
                } else {
                    Err(format!("Request failed with status - {}", res.status))
                },
            Err(err) => Err(format!("Request failed - {}", err))
        }
    }

    /// Writes this pact out to the provided file path. All directories in the path will
    /// automatically created. If an existing pact is found at the path, this pact will be
    /// merged into the pact file.
    pub fn write_pact(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path.parent().unwrap())?;
        if path.exists() {
            let existing_pact = Pact::read_pact(path)?;
            match existing_pact.merge(self) {
                Ok(ref merged_pact) => {
                    let mut file = File::create(path)?;
                    file.write_all(format!("{}", serde_json::to_string_pretty(&merged_pact.to_json()).unwrap()).as_bytes())?;
                    Ok(())
                },
                Err(ref message) => Err(Error::new(ErrorKind::Other, message.clone()))
            }
        } else {
            let mut file = File::create(path)?;
            file.write_all(format!("{}", serde_json::to_string_pretty(&self.to_json()).unwrap()).as_bytes())?;
            Ok(())
        }
    }

    /// Returns a default Pact struct
    pub fn default() -> Pact {
        Pact {
            consumer: Consumer { name: s!("default_consumer") },
            provider: Provider { name: s!("default_provider") },
            interactions: Vec::new(),
            metadata: btreemap!{
                s!("pact-specification") => btreemap!{ s!("version") => PactSpecification::V1_1.version_str() },
                s!("pact-rust") => btreemap!{ s!("version") => s!(VERSION.unwrap_or("unknown")) }
            },
            specification_version: PactSpecification::V2
        }
    }
}

fn decode_query(query: &str) -> String {
    let mut chars = query.chars();
    let mut ch = chars.next();
    let mut result = String::new();

    while ch.is_some() {
        let c = ch.unwrap();
        if c == '%' {
            let c1 = chars.next();
            let c2 = chars.next();
            match (c1, c2) {
                (Some(v1), Some(v2)) => {
                    let mut s = String::new();
                    s.push(v1);
                    s.push(v2);
                    let decoded: Result<Vec<u8>, _> = FromHex::from_hex(s.into_bytes());
                    match decoded {
                        Ok(n) => result.push(n[0] as char),
                        Err(_) => {
                            result.push('%');
                            result.push(v1);
                            result.push(v2);
                        }
                    }
                },
                (Some(v1), None) => {
                    result.push('%');
                    result.push(v1);
                },
                _ => result.push('%')
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }

        ch = chars.next();
    }

    result
}

fn encode_query(query: &str) -> String {
    query.chars().map(|ch| {
        match ch {
            ' ' => s!("+"),
            '-' => ch.to_string(),
            'a'...'z' => ch.to_string(),
            'A'...'Z' => ch.to_string(),
            '0'...'9' => ch.to_string(),
            _ => ch.escape_unicode()
                .filter(|u| u.is_digit(16))
                .batching(|it| {
                    match it.next() {
                        None => None,
                        Some(x) => Some((x, it.next().unwrap()))
                    }
                })
                .map(|u| format!("%{}{}", u.0, u.1))
                .collect()
        }
    }).collect()
}

/// Parses a query string into an optional map. The query parameter name will be mapped to
/// a list of values. Where the query parameter is repeated, the order of the values will be
/// preserved.
pub fn parse_query_string(query: &String) -> Option<HashMap<String, Vec<String>>> {
    if !query.is_empty() {
        Some(query.split("&").map(|kv| {
            if kv.is_empty() {
                vec![]
            } else if kv.contains("=") {
                kv.splitn(2, "=").collect::<Vec<&str>>()
            } else {
                vec![kv]
            }
        }).fold(HashMap::new(), |mut map, name_value| {
            if !name_value.is_empty() {
                let name = decode_query(name_value[0]);
                let value = if name_value.len() > 1 {
                    decode_query(name_value[1])
                } else {
                    s!("")
                };
                map.entry(name).or_insert(vec![]).push(value);
            }
            map
        }))
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
