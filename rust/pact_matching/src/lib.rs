//! The `pact_matching` crate provides the core logic to performing matching on HTTP requests
//! and responses. It implements the V3 Pact specification (https://github.com/pact-foundation/pact-specification/tree/version-3).
//!
//! ## To use it
//!
//! To use it, add it to your dependencies in your cargo manifest.
//!
//! This crate provides two functions: [`match_request`](fn.match_request.html) and [`match_response`](fn.match_response.html).
//! These functions take an expected and actual request or response
//! model from the [`models`)(models/index.html) module, and return a vector of mismatches.
//!
//! To compare any incoming request, it first needs to be converted to a [`models::Request`](models/struct.Request.html) and then can be compared. Same for
//! any response.
//!
//! ## Reading and writing Pact files
//!
//! The [`Pact`](models/struct.Pact.html) struct in the [`models`)(models/index.html) module has methods to read and write pact JSON files. It supports all the specification
//! versions up to V3, but will converted a V1 and V1.1 spec file to a V2 format.
//!
//! ## Matching request and response parts
//!
//! V3 specification matching is supported for both JSON and XML bodies, headers, query strings and request paths.
//!
//! To understand the basic rules of matching, see [Matching Gotchas](https://github.com/realestate-com-au/pact/wiki/Matching-gotchas).
//! For example test cases for matching, see the [Pact Specification Project, version 3](https://github.com/bethesque/pact-specification/tree/version-3).
//!
//! By default, Pact will use string equality matching following Postel's Law. This means
//! that for an actual value to match an expected one, they both must consist of the same
//! sequence of characters. For collections (basically Maps and Lists), they must have the
//! same elements that match in the same sequence, with cases where the additional elements
//! in an actual Map are ignored.
//!
//! Matching rules can be defined for both request and response elements based on a pseudo JSON-Path
//! syntax.
//!
//! ### Matching Bodies
//!
//! For the most part, matching involves matching request and response bodies in JSON or XML format.
//! Other formats will either have their own matching rules, or will follow the JSON one.
//!
//! #### JSON body matching rules
//!
//! Bodies consist of Objects (Maps of Key-Value pairs), Arrays (Lists) and values (Strings, Numbers, true, false, null).
//! Body matching rules are prefixed with `$`.
//!
//! The following method is used to determine if two bodies match:
//!
//! 1. If both the actual body and expected body are empty, the bodies match.
//! 2. If the actual body is non-empty, and the expected body empty, the bodies match.
//! 3. If the actual body is empty, and the expected body non-empty, the bodies don't match.
//! 4. Otherwise do a comparison on the contents of the bodies.
//!
//! ##### For the body contents comparison:
//!
//! 1. If the actual and expected values are both Objects, compare as Maps.
//! 2. If the actual and expected values are both Arrays, compare as Lists.
//! 3. If the expected value is an Object, and the actual is not, they don't match.
//! 4. If the expected value is an Array, and the actual is not, they don't match.
//! 5. Otherwise, compare the values
//!
//! ##### For comparing Maps
//!
//! 1. If the actual map is non-empty while the expected is empty, they don't match.
//! 2. If we allow unexpected keys, and the number of expected keys is greater than the actual keys,
//! they don't match.
//! 3. If we don't allow unexpected keys, and the expected and actual maps don't have the
//! same number of keys, they don't match.
//! 4. Otherwise, for each expected key and value pair:
//!     1. if the actual map contains the key, compare the values
//!     2. otherwise they don't match
//!
//! Postel's law governs if we allow unexpected keys or not.
//!
//! ##### For comparing lists
//!
//! 1. If there is a body matcher defined that matches the path to the list, default
//! to that matcher and then compare the list contents.
//! 2. If the expected list is empty and the actual one is not, the lists don't match.
//! 3. Otherwise
//!     1. compare the list sizes
//!     2. compare the list contents
//!
//! ###### For comparing list contents
//!
//! 1. For each value in the expected list:
//!     1. If the index of the value is less than the actual list's size, compare the value
//!        with the actual value at the same index using the method for comparing values.
//!     2. Otherwise the value doesn't match
//!
//! ##### For comparing values
//!
//! 1. If there is a matcher defined that matches the path to the value, default to that
//! matcher
//! 2. Otherwise compare the values using equality.
//!
//! #### XML body matching rules
//!
//! Bodies consist of a root element, Elements (Lists with children), Attributes (Maps) and values (Strings).
//! Body matching rules are prefixed with `$`.
//!
//! The following method is used to determine if two bodies match:
//!
//! 1. If both the actual body and expected body are empty, the bodies match.
//! 2. If the actual body is non-empty, and the expected body empty, the bodies match.
//! 3. If the actual body is empty, and the expected body non-empty, the bodies don't match.
//! 4. Otherwise do a comparison on the contents of the bodies.
//!
//! ##### For the body contents comparison:
//!
//! Start by comparing the root element.
//!
//! ##### For comparing elements
//!
//! 1. If there is a body matcher defined that matches the path to the element, default
//! to that matcher on the elements name or children.
//! 2. Otherwise the elements match if they have the same name.
//!
//! Then, if there are no mismatches:
//!
//! 1. compare the attributes of the element
//! 2. compare the child elements
//! 3. compare the text nodes
//!
//! ##### For comparing attributes
//!
//! Attributes are treated as a map of key-value pairs.
//!
//! 1. If the actual map is non-empty while the expected is empty, they don't match.
//! 2. If we allow unexpected keys, and the number of expected keys is greater than the actual keys,
//! they don't match.
//! 3. If we don't allow unexpected keys, and the expected and actual maps don't have the
//! same number of keys, they don't match.
//!
//! Then, for each expected key and value pair:
//!
//! 1. if the actual map contains the key, compare the values
//! 2. otherwise they don't match
//!
//! Postel's law governs if we allow unexpected keys or not. Note for matching paths, attribute names are prefixed with an `@`.
//!
//! ###### For comparing child elements
//!
//! 1. If there is a matcher defined for the path to the child elements, then pad out the expected child elements to have the
//! same size as the actual child elements.
//! 2. Otherwise
//!     1. If the actual children is non-empty while the expected is empty, they don't match.
//!     2. If we allow unexpected keys, and the number of expected children is greater than the actual children,
//!     they don't match.
//!     3. If we don't allow unexpected keys, and the expected and actual children don't have the
//!     same number of elements, they don't match.
//!
//! Then, for each expected and actual element pair, compare them using the rules for comparing elements.
//!
//! ##### For comparing text nodes
//!
//! Text nodes are combined into a single string and then compared as values.
//!
//! 1. If there is a matcher defined that matches the path to the text node (text node paths end with `#text`), default to that
//! matcher
//! 2. Otherwise compare the text using equality.
//!
//!
//! ##### For comparing values
//!
//! 1. If there is a matcher defined that matches the path to the value, default to that
//! matcher
//! 2. Otherwise compare the values using equality.
//!
//! ### Matching Paths
//!
//! Paths are matched by the following:
//!
//! 1. If there is a matcher defined for `path`, default to that matcher.
//! 2. Otherwise paths are compared as Strings
//!
//! ### Matching Queries
//!
//! 1. If the actual and expected query strings are empty, they match.
//! 2. If the actual is not empty while the expected is, they don't match.
//! 3. If the actual is empty while the expected is not, they don't match.
//! 4. Otherwise convert both into a Map of keys mapped to a list values, and compare those.
//!
//! #### Matching Query Maps
//!
//! Query strings are parsed into a Map of keys mapped to lists of values. Key value
//! pairs can be in any order, but when the same key appears more than once the values
//! are compared in the order they appear in the query string.
//!
//! ### Matching Headers
//!
//! 1. Do a case-insensitive sort of the headers by keys
//! 2. For each expected header in the sorted list:
//!     1. If the actual headers contain that key, compare the header values
//!     2. Otherwise the header does not match
//!
//! For matching header values:
//!
//! 1. If there is a matcher defined for `header.<HEADER_KEY>`, default to that matcher
//! 2. Otherwise strip all whitespace after commas and compare the resulting strings.
//!
//! #### Matching Request Headers
//!
//! Request headers are matched by excluding the cookie header.
//!
//! #### Matching Request cookies
//!
//! If the list of expected cookies contains all the actual cookies, the cookies match.
//!
//! ### Matching Status Codes
//!
//! Status codes are compared as integer values.
//!
//! ### Matching HTTP Methods
//!
//! The actual and expected methods are compared as case-insensitive strings.
//!
//! ## Matching Rules
//!
//! Pact supports extending the matching rules on each type of object (Request or Response) with a `matchingRules` element in the pact file.
//! This is a map of JSON path strings to a matcher. When an item is being compared, if there is an entry in the matching
//! rules that corresponds to the path to the item, the comparison will be delegated to the defined matcher. Note that the
//! matching rules cascade, so a rule can be specified on a value and will apply to all children of that value.
//!
//! ## Matcher Path expressions
//!
//! Pact does not support the full JSON path expressions, only ones that match the following rules:
//!
//! 1. All paths start with a dollar (`$`), representing the root.
//! 2. All path elements are separated by periods (`.`), except array indices which use square brackets (`[]`).
//! 3. Path elements represent keys.
//! 4. A star (`*`) can be used to match all keys of a map or all items of an array (one level only).
//!
//! So the expression `$.item1.level[2].id` will match the highlighted item in the following body:
//!
//! ```js,ignore
//! {
//!   "item1": {
//!     "level": [
//!       {
//!         "id": 100
//!       },
//!       {
//!         "id": 101
//!       },
//!       {
//!         "id": 102 // <---- $.item1.level[2].id
//!       },
//!       {
//!         "id": 103
//!       }
//!     ]
//!   }
//! }
//! ```
//!
//! while `$.*.level[*].id` will match all the ids of all the levels for all items.
//!
//! ### Matcher selection algorithm
//!
//! Due to the star notation, there can be multiple matcher paths defined that correspond to an item. The first, most
//! specific expression is selected by assigning weightings to each path element and taking the product of the weightings.
//! The matcher with the path with the largest weighting is used.
//!
//! * The root node (`$`) is assigned the value 2.
//! * Any path element that does not match is assigned the value 0.
//! * Any property name that matches a path element is assigned the value 2.
//! * Any array index that matches a path element is assigned the value 2.
//! * Any star (`*`) that matches a property or array index is assigned the value 1.
//! * Everything else is assigned the value 0.
//!
//! So for the body with highlighted item:
//!
//! ```js,ignore
//! {
//!   "item1": {
//!     "level": [
//!       {
//!         "id": 100
//!       },
//!       {
//!         "id": 101
//!       },
//!       {
//!         "id": 102 // <--- Item under consideration
//!       },
//!       {
//!         "id": 103
//!       }
//!     ]
//!   }
//! }
//! ```
//!
//! The expressions will have the following weightings:
//!
//! | expression | weighting calculation | weighting |
//! |------------|-----------------------|-----------|
//! | $ | $(2) | 2 |
//! | $.item1 | $(2).item1(2) | 4 |
//! | $.item2 | $(2).item2(0) | 0 |
//! | $.item1.level | $(2).item1(2).level(2) | 8 |
//! | $.item1.level[1] | $(2).item1(2).level(2)[1(2)] | 16 |
//! | $.item1.level[1].id | $(2).item1(2).level(2)[1(2)].id(2) | 32 |
//! | $.item1.level[1].name | $(2).item1(2).level(2)[1(2)].name(0) | 0 |
//! | $.item1.level[2] | $(2).item1(2).level(2)[2(0)] | 0 |
//! | $.item1.level[2].id | $(2).item1(2).level(2)[2(0)].id(2) | 0 |
//! | $.item1.level[*].id | $(2).item1(2).level(2)[*(1)].id(2) | 16 |
//! | $.\*.level[\*].id | $(2).*(1).level(2)[*(1)].id(2) | 8 |
//!
//! So for the item with id 102, the matcher with path `$.item1.level[1].id` and weighting 32 will be selected.
//!
//! ## Supported matchers
//!
//! The following matchers are supported:
//!
//! | matcher | example configuration | description |
//! |---------|-----------------------|-------------|
//! | Equality | `{ "match": "equality" }` | This is the default matcher, and relies on the equals operator |
//! | Regex | `{ "match": "regex", "regex": "\\d+" }` | This executes a regular expression match against the string representation of a values. |
//! | Type | `{ "match": "type" }` | This executes a type based match against the values, that is, they are equal if they are the same type. |
//! | MinType | `{ "match": "type", "min": 2 }` | This executes a type based match against the values, that is, they are equal if they are the same type. In addition, if the values represent a collection, the length of the actual value is compared against the minimum. |
//! | MaxType | `{ "match": "type", "max": 10 }` | This executes a type based match against the values, that is, they are equal if they are the same type. In addition, if the values represent a collection, the length of the actual value is compared against the maximum. |
//! | MinMaxType | `{ "match": "type", "min": 1, "max": 10 }` | This executes a type based match against the values, that is, they are equal if they are the same type. In addition, if the values represent a collection, the length of the actual value is compared against the minimum and maximum. |
//!

#![warn(missing_docs)]

/// Simple macro to convert a string slice to a `String` struct.
#[macro_export]
macro_rules! s {
    ($e:expr) => ($e.to_string())
}

use std::collections::HashMap;
use std::iter::FromIterator;
use lazy_static::*;
use ansi_term::*;
use ansi_term::Colour::*;
use std::str;
use serde_json::*;
use log::*;

#[macro_use] pub mod models;
mod path_exp;
mod timezone_db;
pub mod time_utils;
mod matchers;
pub mod json;
mod xml;
mod binary_utils;

use crate::models::HttpPart;
use crate::models::matchingrules::*;
use crate::models::generators::*;
use crate::matchers::*;
use std::fmt::Display;
use nom::lib::std::fmt::Formatter;
use crate::models::content_types::ContentType;

fn strip_whitespace<'a, T: FromIterator<&'a str>>(val: &'a String, split_by: &'a str) -> T {
  val.split(split_by).map(|v| v.trim()).collect()
}

lazy_static! {
    static ref BODY_MATCHERS: [
      (fn(content_type: &ContentType) -> bool,
      fn(expected: &dyn models::HttpPart, actual: &dyn models::HttpPart, config: DiffConfig, mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules)); 4]
       = [
        (|content_type| { content_type.is_json() }, json::match_json),
        (|content_type| { content_type.is_xml() }, xml::match_xml),
        (|content_type| { content_type.base_type() == "application/octet-stream" }, binary_utils::match_octet_stream),
        (|content_type| { content_type.base_type() == "multipart/form-data" }, binary_utils::match_mime_multipart)
    ];
}

static PARAMETERISED_HEADER_TYPES: [&'static str; 2] = ["accept", "content-type"];

/// Enum that defines the different types of mismatches that can occur.
#[derive(Debug, Clone)]
pub enum Mismatch {
    /// Request Method mismatch
    MethodMismatch {
        /// Expected request method
        expected: String,
        /// Actual request method
        actual: String
    },
    /// Request Path mismatch
    PathMismatch {
        /// expected request path
        expected: String,
        /// actual request path
        actual: String,
        /// description of the mismatch
        mismatch: String
    },
    /// Response status mismatch
    StatusMismatch {
        /// expected response status
        expected: u16,
        /// actual response status
        actual: u16
    },
    /// Request query mismatch
    QueryMismatch {
        /// query parameter name
        parameter: String,
        /// expected value
        expected: String,
        /// actual value
        actual: String,
        /// description of the mismatch
        mismatch: String
    },
    /// Header mismatch
    HeaderMismatch {
        /// header key
        key: String,
        /// expected value
        expected: String,
        /// actual value
        actual: String,
        /// description of the mismatch
        mismatch: String
    },
    /// Mismatch in the content type of the body
    BodyTypeMismatch {
        /// expected content type of the body
        expected: String,
        /// actual content type of the body
        actual: String,
        /// description of the mismatch
        mismatch: String
    },
    /// Body element mismatch
    BodyMismatch {
        /// path expression to where the mismatch occurred
        path: String,
        /// expected value
        expected: Option<Vec<u8>>,
        /// actual value
        actual: Option<Vec<u8>>,
        /// description of the mismatch
        mismatch: String
    }
}

impl Mismatch {
    /// Converts the mismatch to a `Value` struct.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            &Mismatch::MethodMismatch { expected: ref e, actual: ref a } => {
                json!({
                    s!("type") : json!("MethodMismatch"),
                    s!("expected") : json!(e),
                    s!("actual") : json!(a)
                })
            },
            &Mismatch::PathMismatch { expected: ref e, actual: ref a, mismatch: ref m } => {
                json!({
                    s!("type") : json!("PathMismatch"),
                    s!("expected") : json!(e),
                    s!("actual") : json!(a),
                    s!("mismatch") : json!(m)
                })
            },
            &Mismatch::StatusMismatch { expected: ref e, actual: ref a } => {
                json!({
                    s!("type") : json!("StatusMismatch"),
                    s!("expected") : json!(e),
                    s!("actual") : json!(a)
                })
            },
            &Mismatch::QueryMismatch { parameter: ref p, expected: ref e, actual: ref a, mismatch: ref m } => {
                json!({
                    s!("type") : json!("QueryMismatch"),
                    s!("parameter") : json!(p),
                    s!("expected") : json!(e),
                    s!("actual") : json!(a),
                    s!("mismatch") : json!(m)
                })
            },
            &Mismatch::HeaderMismatch { key: ref k, expected: ref e, actual: ref a, mismatch: ref m } => {
                json!({
                    s!("type") : json!("HeaderMismatch"),
                    s!("key") : json!(k),
                    s!("expected") : json!(e),
                    s!("actual") : json!(a),
                    s!("mismatch") : json!(m)
                })
            },
            &Mismatch::BodyTypeMismatch { expected: ref e, actual: ref a, mismatch: ref m } => {
                json!({
                    s!("type") : json!("BodyTypeMismatch"),
                    s!("expected") : json!(e),
                    s!("actual") : json!(a),
                    s!("mismatch") : json!(m)
                })
            },
            &Mismatch::BodyMismatch { path: ref p, expected: ref e, actual: ref a, mismatch: ref m } => {
                 json!({
                    s!("type") : json!("BodyMismatch"),
                    s!("path") : json!(p),
                    s!("expected") : match e {
                        &Some(ref v) => json!(str::from_utf8(v).unwrap_or("ERROR: could not convert from bytes")),
                        &None => serde_json::Value::Null
                    },
                    s!("actual") : match a {
                        &Some(ref v) => json!(str::from_utf8(v).unwrap_or("ERROR: could not convert from bytes")),
                        &None => serde_json::Value::Null
                    },
                    s!("mismatch") : json!(m)
                })
            }
        }
    }

    /// Returns the type of the mismatch as a string
    pub fn mismatch_type(&self) -> String {
        match *self {
            Mismatch::MethodMismatch { .. } => s!("MethodMismatch"),
            Mismatch::PathMismatch { .. } => s!("PathMismatch"),
            Mismatch::StatusMismatch { .. } => s!("StatusMismatch"),
            Mismatch::QueryMismatch { .. } => s!("QueryMismatch"),
            Mismatch::HeaderMismatch { .. } => s!("HeaderMismatch"),
            Mismatch::BodyTypeMismatch { .. } => s!("BodyTypeMismatch"),
            Mismatch::BodyMismatch { .. } => s!("BodyMismatch")
        }
    }

    /// Returns a summary string for this mismatch
    pub fn summary(&self) -> String {
        match *self {
            Mismatch::MethodMismatch { expected: ref e, .. } => format!("is a {} request", e),
            Mismatch::PathMismatch { expected: ref e, .. } => format!("to path '{}'", e),
            Mismatch::StatusMismatch { expected: ref e, .. } => format!("has status code {}", e),
            Mismatch::QueryMismatch { ref parameter, expected: ref e, .. } => format!("includes parameter '{}' with value '{}'", parameter, e),
            Mismatch::HeaderMismatch { ref key, expected: ref e, .. } => format!("includes header '{}' with value '{}'", key, e),
            Mismatch::BodyTypeMismatch { .. } => s!("has a matching body"),
            Mismatch::BodyMismatch { .. } => s!("has a matching body")
        }
    }

    /// Returns a formated string for this mismatch
    pub fn description(&self) -> String {
        match *self {
            Mismatch::MethodMismatch { expected: ref e, actual: ref a } => format!("expected {} but was {}", e, a),
            Mismatch::PathMismatch { ref mismatch, .. } => mismatch.clone(),
            Mismatch::StatusMismatch { expected: ref e, actual: ref a } => format!("expected {} but was {}", e, a),
            Mismatch::QueryMismatch { ref mismatch, .. } => mismatch.clone(),
            Mismatch::HeaderMismatch { ref mismatch, .. } => mismatch.clone(),
            Mismatch::BodyTypeMismatch {  expected: ref e, actual: ref a, .. } => format!("expected '{}' body but was '{}'", e, a),
            Mismatch::BodyMismatch { ref path, ref mismatch, .. } => format!("{} -> {}", path, mismatch)
        }
    }

    /// Returns a formatted string with ansi escape codes for this mismatch
    pub fn ansi_description(&self) -> String {
        match *self {
            Mismatch::MethodMismatch { expected: ref e, actual: ref a } => format!("expected {} but was {}", Red.paint(e.clone()), Green.paint(a.clone())),
            Mismatch::PathMismatch { expected: ref e, actual: ref a, .. } => format!("expected '{}' but was '{}'", Red.paint(e.clone()), Green.paint(a.clone())),
            Mismatch::StatusMismatch { expected: ref e, actual: ref a } => format!("expected {} but was {}", Red.paint(e.to_string()), Green.paint(a.to_string())),
            Mismatch::QueryMismatch { expected: ref e, actual: ref a, parameter: ref p, .. } => format!("Expected '{}' but received '{}' for query parameter '{}'",
                Red.paint(e.to_string()), Green.paint(a.to_string()), Style::new().bold().paint(p.clone())),
            Mismatch::HeaderMismatch { expected: ref e, actual: ref a, key: ref k, .. } => format!("Expected header '{}' to have value '{}' but was '{}'",
                Style::new().bold().paint(k.clone()), Red.paint(e.to_string()), Green.paint(a.to_string())),
            Mismatch::BodyTypeMismatch {  expected: ref e, actual: ref a, .. } => format!("expected '{}' body but was '{}'", Red.paint(e.clone()), Green.paint(a.clone())),
            Mismatch::BodyMismatch { ref path, ref mismatch, .. } => format!("{} -> {}", Style::new().bold().paint(path.clone()), mismatch)
        }
    }
}

impl PartialEq for Mismatch {
    fn eq(&self, other: &Mismatch) -> bool {
        match (self, other) {
            (&Mismatch::MethodMismatch{ expected: ref e1, actual: ref a1 },
                &Mismatch::MethodMismatch{ expected: ref e2, actual: ref a2 }) => {
                e1 == e2 && a1 == a2
            },
            (&Mismatch::PathMismatch{ expected: ref e1, actual: ref a1, mismatch: _ },
                &Mismatch::PathMismatch{ expected: ref e2, actual: ref a2, mismatch: _ }) => {
                e1 == e2 && a1 == a2
            },
            (&Mismatch::StatusMismatch{ expected: ref e1, actual: ref a1 },
                &Mismatch::StatusMismatch{ expected: ref e2, actual: ref a2 }) => {
                e1 == e2 && a1 == a2
            },
            (&Mismatch::BodyTypeMismatch{ expected: ref e1, actual: ref a1, mismatch: _  },
                &Mismatch::BodyTypeMismatch{ expected: ref e2, actual: ref a2, mismatch: _ }) => {
                e1 == e2 && a1 == a2
            },
            (&Mismatch::QueryMismatch{ parameter: ref p1, expected: ref e1, actual: ref a1, mismatch: _ },
                &Mismatch::QueryMismatch{ parameter: ref p2, expected: ref e2, actual: ref a2, mismatch: _ }) => {
                p1 == p2 && e1 == e2 && a1 == a2
            },
            (&Mismatch::HeaderMismatch{ key: ref p1, expected: ref e1, actual: ref a1, mismatch: _ },
                &Mismatch::HeaderMismatch{ key: ref p2, expected: ref e2, actual: ref a2, mismatch: _ }) => {
                p1 == p2 && e1 == e2 && a1 == a2
            },
            (&Mismatch::BodyMismatch{ path: ref p1, expected: ref e1, actual: ref a1, mismatch: _ },
                &Mismatch::BodyMismatch{ path: ref p2, expected: ref e2, actual: ref a2, mismatch: _ }) => {
                p1 == p2 && e1 == e2 && a1 == a2
            },
            (_, _) => false
        }
    }
}

impl Display for Mismatch {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.description())
  }
}

/// Enum that defines the configuration options for performing a match.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffConfig {
    /// If unexpected keys are allowed and ignored during matching.
    AllowUnexpectedKeys,
    /// If unexpected keys cause a mismatch.
    NoUnexpectedKeys
}

/// Matches the actual text body to the expected one.
pub fn match_text(expected: &Vec<u8>, actual: &Vec<u8>, mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
  let path = vec![s!("$")];
  if matchers.matcher_is_defined("body", &path) {
    if let Err(messages) = match_values("body", &path, matchers.clone(), expected, actual) {
      for message in messages {
        mismatches.push(Mismatch::BodyMismatch {
          path: s!("$"),
          expected: Some(expected.clone()),
          actual: Some(actual.clone()),
          mismatch: message.clone()
        })
      }
    }
  } else if expected != actual {
    mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.clone()),
      actual: Some(actual.clone()),
      mismatch: format!("Expected text '{:?}' but received '{:?}'", expected, actual) });
  };
}

/// Matches the actual request method to the expected one.
pub fn match_method(expected: String, actual: String, mismatches: &mut Vec<Mismatch>) {
    if expected.to_lowercase() != actual.to_lowercase() {
        mismatches.push(Mismatch::MethodMismatch { expected: expected, actual: actual });
    }
}

/// Matches the actual request path to the expected one.
pub fn match_path(expected: String, actual: String, mismatches: &mut Vec<Mismatch>,
    matchers: &MatchingRules) {
    let path = vec![];
    let matcher_result = if matchers.matcher_is_defined("path", &path) {
      matchers::match_values("path", &path, matchers.clone(), &expected, &actual)
    } else {
      expected.matches(&actual, &MatchingRule::Equality).map_err(|err| vec![err])
    };
    match matcher_result {
        Err(messages) => {
          for message in messages {
            mismatches.push(Mismatch::PathMismatch {
              expected: expected.clone(),
              actual: actual.clone(), mismatch: message.clone()
            })
          }
        },
        Ok(_) => ()
    }
}

fn compare_query_parameter_value(key: &String, expected: &String, actual: &String, index: usize,
    mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
    let path = vec![s!("$"), key.clone(), format!("{}", index)];
    let matcher_result = if matchers.matcher_is_defined("query", &path) {
      matchers::match_values("query", &path, matchers.clone(), expected, actual)
    } else {
      expected.matches(actual, &MatchingRule::Equality).map_err(|err| vec![err])
    };
    match matcher_result {
        Err(messages) => {
          for message in messages {
            mismatches.push(Mismatch::QueryMismatch {
              parameter: key.clone(),
              expected: expected.clone(),
              actual: actual.clone(),
              mismatch: message
            })
          }
        },
        Ok(_) => ()
    }
}

fn compare_query_parameter_values(key: &String, expected: &Vec<String>, actual: &Vec<String>,
    mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
    for (index, val) in expected.iter().enumerate() {
        if index < actual.len() {
            compare_query_parameter_value(key, val, &actual[index], index, mismatches, matchers);
        } else {
            mismatches.push(Mismatch::QueryMismatch { parameter: key.clone(),
                expected: format!("{:?}", expected),
                actual: format!("{:?}", actual),
                mismatch: format!("Expected query parameter '{}' value '{}' but was missing", key, val) });
        }
    }
}

fn match_query_values(key: &String, expected: &Vec<String>, actual: &Vec<String>,
    mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
    if expected.is_empty() && !actual.is_empty() {
        mismatches.push(Mismatch::QueryMismatch { parameter: key.clone(),
            expected: format!("{:?}", expected),
            actual: format!("{:?}", actual),
            mismatch: format!("Expected an empty parameter list for '{}' but received {:?}", key, actual) });
    } else {
        if expected.len() != actual.len() {
            mismatches.push(Mismatch::QueryMismatch { parameter: key.clone(),
                expected: format!("{:?}", expected),
                actual: format!("{:?}", actual),
                mismatch: format!(
                    "Expected query parameter '{}' with {} value(s) but received {} value(s)",
                    key, expected.len(), actual.len()) });
        }
        compare_query_parameter_values(key, expected, actual, mismatches, matchers);
    }
}

fn match_query_maps(expected: HashMap<String, Vec<String>>, actual: HashMap<String, Vec<String>>,
    mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
    for (key, value) in &expected {
        match actual.get(key) {
            Some(actual_value) => match_query_values(key, value, actual_value, mismatches, matchers),
            None => mismatches.push(Mismatch::QueryMismatch { parameter: key.clone(),
                expected: format!("{:?}", value),
                actual: "".to_string(),
                mismatch: format!("Expected query parameter '{}' but was missing", key) })
        }
    }
    for (key, value) in &actual {
        match expected.get(key) {
            Some(_) => (),
            None => mismatches.push(Mismatch::QueryMismatch { parameter: key.clone(),
                expected: "".to_string(),
                actual: format!("{:?}", value),
                mismatch: format!("Unexpected query parameter '{}' received", key) })
        }
    }
}

/// Matches the actual query parameters to the expected ones.
pub fn match_query(expected: Option<HashMap<String, Vec<String>>>,
    actual: Option<HashMap<String, Vec<String>>>, mismatches: &mut Vec<Mismatch>,
    matchers: &MatchingRules) {
    match (actual, expected) {
        (Some(aqm), Some(eqm)) => match_query_maps(eqm, aqm, mismatches, matchers),
        (Some(aqm), None) => for (key, value) in &aqm {
            mismatches.push(Mismatch::QueryMismatch { parameter: key.clone(),
                expected: "".to_string(),
                actual: format!("{:?}", value),
                mismatch: format!("Unexpected query parameter '{}' received", key) });
        },
        (None, Some(eqm)) => for (key, value) in &eqm {
            mismatches.push(Mismatch::QueryMismatch { parameter: key.clone(),
                expected: format!("{:?}", value),
                actual: "".to_string(),
                mismatch: format!("Expected query parameter '{}' but was missing", key) });
        },
        (None, None) => (),
    };
}

fn parse_charset_parameters(parameters: &[&str]) -> HashMap<String, String> {
    parameters.iter().map(|v| v.split("=").map(|p| p.trim()).collect::<Vec<&str>>())
        .fold(HashMap::new(), |mut map, name_value| {
            map.insert(name_value[0].to_string(), name_value[1].to_string());
            map
        })
}

fn match_parameter_header(expected: &String, actual: &String, mismatches: &mut Vec<Mismatch>, header: &String) {
    let expected_values: Vec<&str> = strip_whitespace(expected, ";");
    let actual_values: Vec<&str> = strip_whitespace(actual, ";");
    let expected_parameters = expected_values.as_slice().split_first().unwrap();
    let actual_parameters = actual_values.as_slice().split_first().unwrap();
    let header_mismatch = Mismatch::HeaderMismatch { key: header.clone(),
        expected: format!("{}", expected),
        actual: format!("{}", actual),
        mismatch: format!("Expected header '{}' to have value '{}' but was '{}'",
            header, expected, actual) };

    if expected_parameters.0 == actual_parameters.0 {
        let expected_parameter_map = parse_charset_parameters(expected_parameters.1);
        let actual_parameter_map = parse_charset_parameters(actual_parameters.1);
        for (k, v) in expected_parameter_map {
            if actual_parameter_map.contains_key(&k) {
                if v != *actual_parameter_map.get(&k).unwrap() {
                    mismatches.push(header_mismatch.clone());
                }
            } else {
                mismatches.push(header_mismatch.clone());
            }
        }
    } else {
        mismatches.push(header_mismatch.clone());
    }
}

fn match_header_value(key: &String, expected: &String, actual: &String, mismatches: &mut Vec<Mismatch>,
    matchers: &MatchingRules) {
    let path = vec![s!("$"), key.clone()];
    let expected = strip_whitespace::<String>(expected, ",");
    let actual = strip_whitespace::<String>(actual, ",");

    let matcher_result = if matchers.matcher_is_defined("header", &path) {
        matchers::match_values("header",&path, matchers.clone(), &expected, &actual)
    } else if PARAMETERISED_HEADER_TYPES.contains(&key.to_lowercase().as_str()) {
        match_parameter_header(&expected, &actual, mismatches, &key);
        Ok(())
    } else {
      expected.matches(&actual, &MatchingRule::Equality).map_err(|err| vec![err])
    };
    match matcher_result {
        Err(messages) => {
          for message in messages {
            mismatches.push(Mismatch::HeaderMismatch {
              key: key.clone(),
              expected: expected.clone(),
              actual: actual.clone(),
              mismatch: format!("Mismatch with header '{}': {}", key.clone(), message)
            })
          }
        },
        Ok(_) => ()
    }
}

fn find_entry<T>(map: &HashMap<String, T>, key: &String) -> Option<(String, T)> where T: Clone {
    match map.keys().find(|k| k.to_lowercase() == key.to_lowercase() ) {
        Some(k) => map.get(k).map(|v| (key.clone(), v.clone()) ),
        None => None
    }
}

fn match_header_maps(expected: HashMap<String, Vec<String>>, actual: HashMap<String, Vec<String>>,
  mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
  for (key, value) in &expected {
    match find_entry(&actual, key) {
      Some((_, actual_value)) => for (index, val) in value.iter().enumerate() {
        match_header_value(key, val, actual_value.get(index).unwrap_or(&s!("")), mismatches, matchers)
      },
      None => mismatches.push(Mismatch::HeaderMismatch { key: key.clone(),
        expected: format!("{:?}", value.join(", ")),
        actual: "".to_string(),
        mismatch: format!("Expected header '{}' but was missing", key) })
    }
  }
}

/// Matches the actual headers to the expected ones.
pub fn match_headers(expected: Option<HashMap<String, Vec<String>>>,
  actual: Option<HashMap<String, Vec<String>>>, mismatches: &mut Vec<Mismatch>,
  matchers: &MatchingRules) {
  match (actual, expected) {
    (Some(aqm), Some(eqm)) => match_header_maps(eqm, aqm, mismatches, matchers),
    (Some(_), None) => (),
    (None, Some(eqm)) => for (key, value) in &eqm {
      mismatches.push(Mismatch::HeaderMismatch { key: key.clone(),
        expected: format!("{:?}", value.join(", ")),
        actual: "".to_string(),
        mismatch: format!("Expected header '{}' but was missing", key) });
    },
    (None, None) => (),
  };
}

fn compare_bodies(content_type: &ContentType, expected: &dyn models::HttpPart, actual: &dyn models::HttpPart, config: DiffConfig,
                  mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
  match BODY_MATCHERS.iter().find(|mt| mt.0(&content_type)) {
    Some(ref match_fn) => {
      debug!("Using body matcher for content type '{}'", content_type);
      match_fn.1(expected, actual, config, mismatches, matchers)
    },
    None => {
      debug!("No body matcher defined for content type '{}', using plain text matcher", content_type);
      match_text(&expected.body().value(), &actual.body().value(), mismatches, matchers)
    }
  }
}

fn match_body_content(content_type: &ContentType, expected: &dyn models::HttpPart, actual: &dyn models::HttpPart,
    config: DiffConfig, mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
    match (expected.body(), actual.body()) {
        (&models::OptionalBody::Missing, _) => (),
        (&models::OptionalBody::Null, &models::OptionalBody::Present(ref b, _)) => {
            mismatches.push(Mismatch::BodyMismatch { expected: None, actual: Some(b.clone()),
                mismatch: format!("Expected empty body but received '{:?}'", b.clone()),
                path: s!("/")});
        },
        (&models::OptionalBody::Empty, &models::OptionalBody::Present(ref b, _)) => {
            mismatches.push(Mismatch::BodyMismatch { expected: None, actual: Some(b.clone()),
                mismatch: format!("Expected empty body but received '{:?}'", b.clone()),
                path: s!("/")});
        },
        (&models::OptionalBody::Null, _) => (),
        (&models::OptionalBody::Empty, _) => (),
        (e, &models::OptionalBody::Missing) => {
            mismatches.push(Mismatch::BodyMismatch { expected: Some(e.value()), actual: None,
                mismatch: format!("Expected body '{:?}' but was missing", e.value()),
                path: s!("/")});
        },
        (_, _) => {
          compare_bodies(content_type, expected, actual, config, mismatches, matchers);
        }
    }
}

/// Matches the actual body to the expected one. This takes into account the content type of each.
pub fn match_body(expected: &dyn models::HttpPart, actual: &dyn models::HttpPart, config: DiffConfig,
                  mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
  let expected_content_type = expected.content_type_struct().unwrap_or_default();
  let actual_content_type = actual.content_type_struct().unwrap_or_default();
  debug!("expected content type = '{}', actual content type = '{}'", expected_content_type,
         actual_content_type);
  if expected_content_type.is_unknown() || actual_content_type.is_unknown() || expected_content_type == actual_content_type {
    match_body_content(&expected_content_type, expected, actual, config, mismatches, matchers)
  } else if expected.body().is_present() {
    mismatches.push(Mismatch::BodyTypeMismatch {
      expected: expected_content_type.to_string(),
      actual: actual_content_type.to_string(),
      mismatch: format!("Expected body with content type {} but was {}", expected_content_type,
                        actual_content_type),
    });
  }
}

/// Matches the expected and actual requests.
pub fn match_request(expected: models::Request, actual: models::Request) -> Vec<Mismatch> {
    let mut mismatches = vec![];

    log::info!("comparing to expected {}", expected);
    log::debug!("     body: '{}'", expected.body.str_value());
    log::debug!("     matching_rules: {:?}", expected.matching_rules);
    log::debug!("     generators: {:?}", expected.generators);
    match_method(expected.method.clone(), actual.method.clone(), &mut mismatches);
    match_path(expected.path.clone(), actual.path.clone(), &mut mismatches, &expected.matching_rules);
    match_body(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &expected.matching_rules);
    match_query(expected.query, actual.query, &mut mismatches, &expected.matching_rules);
    match_headers(expected.headers, actual.headers, &mut mismatches, &expected.matching_rules);

    log::debug!("--> Mismatches: {:?}", mismatches);
    mismatches
}

/// Matches the actual response status to the expected one.
pub fn match_status(expected: u16, actual: u16, mismatches: &mut Vec<Mismatch>) {
    if expected != actual {
        mismatches.push(Mismatch::StatusMismatch { expected: expected, actual: actual });
    }
}

/// Matches the actual and expected responses.
pub fn match_response(expected: models::Response, actual: models::Response) -> Vec<Mismatch> {
    let mut mismatches = vec![];

    log::info!("comparing to expected response: {}", expected);
    match_body(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &expected.matching_rules);
    match_status(expected.status, actual.status, &mut mismatches);
    match_headers(expected.headers, actual.headers, &mut mismatches, &expected.matching_rules);

    mismatches
}

/// Matches the actual message contents to the expected one. This takes into account the content type of each.
pub fn match_message_contents(expected: &models::message::Message, actual: &models::message::Message, config: DiffConfig,
                              mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
  let expected_content_type = expected.content_type().unwrap_or_default();
  let actual_content_type = actual.content_type().unwrap_or_default();
  if expected_content_type == actual_content_type {
    match_body_content(&expected_content_type, expected, actual, config, mismatches, matchers)
  } else if expected.contents.is_present() {
    mismatches.push(Mismatch::BodyTypeMismatch {
      expected: expected_content_type.to_string(),
      actual: actual_content_type.to_string(),
      mismatch: format!("Expected message with content type {} but was {}",
                        expected_content_type, actual_content_type),
    });
  }
}

/// Matches the actual and expected messages.
pub fn match_message(expected: models::message::Message, actual: models::message::Message) -> Vec<Mismatch> {
    let mut mismatches = vec![];

    log::info!("comparing to expected message: {:?}", expected);
    match_message_contents(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &expected.matching_rules);

    mismatches
}

/// Generates the request by applying any defined generators
pub fn generate_request(request: &models::Request, context: &HashMap<String, Value>) -> models::Request {
    let generators = request.generators.clone();
    let mut request = request.clone();
    generators.apply_generator(&GeneratorCategory::PATH, |_, generator| {
        match generator.generate_value(&request.path, context) {
            Some(v) => request.path = v,
            None => ()
        }
    });
    generators.apply_generator(&GeneratorCategory::HEADER, |key, generator| {
        match request.headers {
            Some(ref mut headers) => if headers.contains_key(key) {
                match generator.generate_value(&headers.get(key).unwrap().clone(), context) {
                    Some(v) => headers.insert(key.clone(), v),
                    None => None
                };
            },
            None => ()
        }
    });
    generators.apply_generator(&GeneratorCategory::QUERY, |key, generator| {
      match request.query {
        Some(ref mut parameters) => match parameters.get_mut(key) {
          Some(parameter) => {
            let mut generated = parameter.clone();
            for (index, val) in parameter.iter().enumerate() {
              match generator.generate_value(val, context) {
                Some(v) => generated[index] = v,
                None => ()
              };
            }
            *parameter = generated;
          },
          None => ()
        },
        None => ()
      }
    });
    request.body = generators.apply_body_generators(&request.body, request.content_type_struct(),
        context);
    request
}

/// Generates the response by applying any defined generators
pub fn generate_response(response: &models::Response, context: &HashMap<String, Value>) -> models::Response {
  let generators = response.generators.clone();
  let mut response = response.clone();
  generators.apply_generator(&GeneratorCategory::STATUS, |_, generator| {
    match generator.generate_value(&response.status, context) {
      Some(v) => response.status = v,
      None => ()
    }
  });
  generators.apply_generator(&GeneratorCategory::HEADER, |key, generator| {
    match response.headers {
      Some(ref mut headers) => if headers.contains_key(key) {
        match generator.generate_value(&headers.get(key).unwrap().clone(), context) {
          Some(v) => headers.insert(key.clone(), v),
          None => None
        };
      },
      None => ()
    }
  });
  response.body = generators.apply_body_generators(&response.body, response.content_type_struct(),
    context);
  response
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod generator_tests;
