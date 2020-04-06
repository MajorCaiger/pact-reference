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
fn unexpected_trailing_slash_in_path() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Path has unexpected trailing slash, trailing slashes can matter",
        "expected" : {
          "method": "POST",
          "path": "/path/to/something",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "/path/to/something/",
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

#[test]
fn missing_trailing_slash_in_path() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Path is missing trailing slash, trailing slashes can matter",
        "expected" : {
          "method": "POST",
          "path": "/path/to/something/",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "/path/to/something",
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

#[test]
fn matches() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Paths match",
        "expected" : {
          "method": "POST",
          "path": "/path/to/something",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "/path/to/something",
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

#[test]
fn incorrect_path() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Paths do not match",
        "expected" : {
          "method": "POST",
          "path": "/path/to/something",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "/path/to/something/else",
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

#[test]
fn forward_slash_found_when_empty_path_expected() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Foward slash found when empty path expected",
        "expected" : {
          "method": "POST",
          "path": "",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "/",
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

#[test]
fn empty_path_found_when_forward_slash_expected() {
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Empty path found when forward slash expected",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "",
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
