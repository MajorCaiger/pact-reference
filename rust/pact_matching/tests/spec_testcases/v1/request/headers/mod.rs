#[allow(unused_imports)]
use test_env_log::test;
#[allow(unused_imports)]
use pact_matching::models::PactSpecification;
#[allow(unused_imports)]
use pact_matching::models::Request;
#[allow(unused_imports)]
use pact_matching::match_request;
#[allow(unused_imports)]
use expectest::prelude::*;
#[allow(unused_imports)]
use serde_json;

#[test]
fn whitespace_after_comma_different() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Whitespace between comma separated headers does not matter",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators,hippos"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators, hippos"
          }
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_request(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn unexpected_header_found() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Extra headers allowed",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {}
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_request(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn order_of_comma_separated_header_values_different() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Comma separated headers out of order, order can matter http://tools.ietf.org/html/rfc2616",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators, hippos"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "hippos, alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_request(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Headers match",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators",
            "Content-Type": "hippos"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Content-Type": "hippos",
            "Accept": "alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_request(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn header_value_is_different_case() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Headers values are case sensitive",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "Alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_request(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn header_name_is_different_case() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Header name is case insensitive",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "ACCEPT": "alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_request(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn empty_headers() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Empty headers match",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_request(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
