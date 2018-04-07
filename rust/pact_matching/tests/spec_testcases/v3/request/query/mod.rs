#[allow(unused_imports)]
use env_logger;
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
fn different_order() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries are the same but in different key order",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "hippo": ["John"],
            "alligator": ["Mary"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
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
fn different_params() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Queries are not the same - hippo is Fred instead of John",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["Fred"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
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
fn matches_with_equals_in_the_query_value() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries are equivalent",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "options": ["delete.topic.enable=true"],
            "broker": ["1"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "options": ["delete.topic.enable=true"],
            "broker": ["1"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
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
fn matches_with_regex() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries match with regex",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"]
          },
          "headers": {},
          "matchingRules": {
            "query": {
              "hippo": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\w+"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["Fred"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
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
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries are the same",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
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
fn missing_params() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Queries are not the same - elephant is missing",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["Fred"],
            "elephant": ["missing"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["Fred"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
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
fn same_parameter_different_values() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Queries are not the same - animals are alligator, hippo versus alligator, elephant",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "animal": ["alligator", "hippo"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "animal": ["alligator", "elephant"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
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
fn same_parameter_multiple_times_in_different_order() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Queries are not the same - values are in different order",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "animal": ["alligator", "hippo", "elephant"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "animal": ["hippo", "alligator", "elephant"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
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
fn same_parameter_multiple_times() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries are the same - multiple values are in same order",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "animal": ["alligator", "hippo", "elephant"],
            "hippo": ["Fred"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "hippo": ["Fred"],
            "animal": ["alligator", "hippo", "elephant"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
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
fn unexpected_param() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Queries are not the same - elephant is not expected",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"]
          },
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": {
            "alligator": ["Mary"],
            "hippo": ["John"],
            "elephant": ["unexpected"]
          },
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_request(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
