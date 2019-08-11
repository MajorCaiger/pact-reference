//! The `pact_verifier` crate provides the core logic to performing verification of providers.
//! It implements the V2 Pact specification (https://github.com/pact-foundation/pact-specification/tree/version-2).

#![warn(missing_docs)]

#[macro_use] extern crate pact_matching;
extern crate ansi_term;
#[macro_use] extern crate log;
extern crate hyper;
extern crate tokio;
extern crate futures;
extern crate bytes;
extern crate reqwest;
extern crate mime;
#[macro_use] extern crate maplit;
extern crate itertools;
extern crate regex;
extern crate difference;
#[macro_use] extern crate serde_json;

#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;
#[cfg(test)]
#[macro_use]
extern crate pact_consumer;
#[cfg(test)]
extern crate env_logger;
#[cfg(test)]
extern crate http;

mod provider_client;
mod pact_broker;

use std::path::Path;
use std::io;
use std::fs;
use pact_matching::*;
use pact_matching::models::*;
use pact_matching::models::provider_states::*;
use pact_matching::models::http_utils::UrlAuth;
use ansi_term::*;
use ansi_term::Colour::*;
use std::collections::HashMap;
use provider_client::{make_provider_request, make_state_change_request, ProviderClientError};
use regex::Regex;
use serde_json::Value;
use tokio::runtime::current_thread::Runtime;

/// Source for loading pacts
#[derive(Debug, Clone)]
pub enum PactSource {
    /// Load the pact from a pact file
    File(String),
    /// Load all the pacts from a Directory
    Dir(String),
    /// Load the pact from a URL
    URL(String, Option<UrlAuth>),
    /// Load all pacts with the provider name from the pact broker url
    BrokerUrl(String, String)
}

/// Information about the Provider to verify
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// Provider Name
    pub name: String,
    /// Provider protocol, defaults to HTTP
    pub protocol: String,
    /// Hostname of the provider
    pub host: String,
    /// Port the provider is running on, defaults to 8080
    pub port: u16,
    /// Base path for the provider, defaults to /
    pub path: String,
    /// URL to post state change requests to
    pub state_change_url: Option<String>,
    /// If teardown state change requests should be made (default is false)
    pub state_change_teardown: bool,
    /// If state change request data should be sent in the body (true) or as query parameters (false)
    pub state_change_body: bool
}

impl ProviderInfo {
    /// Create a default provider info
    pub fn default() -> ProviderInfo {
        ProviderInfo {
            name: s!("provider"),
            protocol: s!("http"),
            host: s!("localhost"),
            port: 8080,
            path: s!("/"),
            state_change_url: None,
            state_change_teardown: false,
            state_change_body: true
        }
    }
}

/// Result of performing a match
#[derive(Debug, Clone)]
pub enum MismatchResult {
    /// Response mismatches
    Mismatches(Vec<Mismatch>, Response, Response),
    /// Error occurred
    Error(String)
}

fn provider_client_error_to_string(err: ProviderClientError) -> String {
    match err {
        ProviderClientError::RequestMethodError(ref method, _) =>
            format!("Invalid request method: '{}'", method),
        ProviderClientError::RequestHeaderNameError(ref name, _) =>
            format!("Invalid header name: '{}'", name),
        ProviderClientError::RequestHeaderValueError(ref value, _) =>
            format!("Invalid header value: '{}'", value),
        ProviderClientError::RequestBodyError(ref message) =>
            format!("Invalid request body: '{}'", message),
        ProviderClientError::ResponseError(ref message) =>
            format!("Invalid response: {}", message),
        ProviderClientError::ResponseStatusCodeError(ref code) =>
            format!("Invalid status code: {}", code)
    }
}

fn verify_response_from_provider(provider: &ProviderInfo, interaction: &Interaction, runtime: &mut Runtime) -> Result<(), MismatchResult> {
  let ref expected_response = interaction.response;
  match runtime.block_on(make_provider_request(provider, &pact_matching::generate_request(&interaction.request))) {
      Ok(ref actual_response) => {
          let mismatches = match_response(expected_response.clone(), actual_response.clone());
          if mismatches.is_empty() {
              Ok(())
          } else {
              Err(MismatchResult::Mismatches(mismatches, expected_response.clone(), actual_response.clone()))
          }
      },
      Err(err) => {
          Err(MismatchResult::Error(provider_client_error_to_string(err)))
      }
  }
}

fn execute_state_change(provider_state: &ProviderState, provider: &ProviderInfo, setup: bool, runtime: &mut Runtime) -> Result<(), MismatchResult> {
    if setup {
        println!("  Given {}", Style::new().bold().paint(provider_state.name.clone()));
    }
    let result = match provider.state_change_url {
        Some(_) => {
            let mut state_change_request = Request { method: s!("POST"), .. Request::default_request() };
            if provider.state_change_body {
              let mut json_body = json!({
                  s!("state") : json!(provider_state.name.clone()),
                  s!("action") : json!(if setup {
                    s!("setup")
                  } else {
                    s!("teardown")
                  })
              });
              {
                let json_body_mut = json_body.as_object_mut().unwrap();
                for (k, v) in provider_state.params.clone() {
                  json_body_mut.insert(k, v);
                }
              }
              state_change_request.body = OptionalBody::Present(json_body.to_string().into());
              state_change_request.headers = Some(hashmap!{ s!("Content-Type") => vec![s!("application/json")] });
            } else {
              let mut query = hashmap!{ s!("state") => vec![provider_state.name.clone()] };
              if setup {
                query.insert(s!("action"), vec![s!("setup")]);
              } else {
                query.insert(s!("action"), vec![s!("teardown")]);
              }
              for (k, v) in provider_state.params.clone() {
                query.insert(k, vec![match v {
                  Value::String(ref s) => s.clone(),
                  _ => v.to_string()
                }]);
              }
              state_change_request.query = Some(query);
            }
            match runtime.block_on(make_state_change_request(provider, &state_change_request)) {
                Ok(_) => Ok(()),
                Err(err) => Err(MismatchResult::Error(provider_client_error_to_string(err)))
            }
        },
        None => {
            if setup {
                println!("    {}", Yellow.paint("WARNING: State Change ignored as there is no state change URL"));
            }
            Ok(())
        }
    };

    debug!("State Change: \"{:?}\" -> {:?}", provider_state, result);
    result
}

fn verify_interaction(provider: &ProviderInfo, interaction: &Interaction, runtime: &mut Runtime) -> Result<(), MismatchResult> {
    for state in interaction.provider_states.clone() {
      execute_state_change(&state, provider, true, runtime)?
    }

    let result = verify_response_from_provider(provider, interaction, runtime);

    if provider.state_change_teardown {
      for state in interaction.provider_states.clone() {
        execute_state_change(&state, provider, false, runtime)?
      }
    }

    result
}

fn display_result(status: u16, status_result: ANSIGenericString<str>,
    header_results: Option<Vec<(String, String, ANSIGenericString<str>)>>,
    body_result: ANSIGenericString<str>) {
    println!("    returns a response which");
    println!("      has status code {} ({})", Style::new().bold().paint(format!("{}", status)),
        status_result);
    match header_results {
        Some(header_results) => {
            println!("      includes headers");
            for (key, value, result) in header_results {
                println!("        \"{}\" with value \"{}\" ({})", Style::new().bold().paint(key),
                    Style::new().bold().paint(value), result);
            }
        },
        None => ()
    }
    println!("      has a matching body ({})", body_result);
}

fn walkdir(dir: &Path) -> io::Result<Vec<io::Result<Pact>>> {
    let mut pacts = vec![];
    debug!("Scanning {:?}", dir);
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walkdir(&path)?;
        } else {
            pacts.push(Pact::read_pact(&path))
        }
    }
    Ok(pacts)
}

fn display_body_mismatch(expected: &Response, actual: &Response, path: &String) {
    match expected.content_type_enum() {
        DetectedContentType::Json => println!("{}", pact_matching::json::display_diff(&expected.body.str_value().to_string(),
            &actual.body.str_value().to_string(), path)),
        _ => ()
    }
}

/// Filter information used to filter the interactions that are verified
pub enum FilterInfo {
    /// No filter, all interactions will be verified
    None,
    /// Filter on the interaction description
    Description(String),
    /// Filter on the interaction provider state
    State(String),
    /// Filter on both the interaction description and provider state
    DescriptionAndState(String, String)
}

impl FilterInfo {

    /// If this filter is filtering on description
    pub fn has_description(&self) -> bool {
        match self {
            &FilterInfo::Description(_) => true,
            &FilterInfo::DescriptionAndState(_, _) => true,
            _ => false
        }
    }

    /// If this filter is filtering on provider state
    pub fn has_state(&self) -> bool {
        match self {
            &FilterInfo::State(_) => true,
            &FilterInfo::DescriptionAndState(_, _) => true,
            _ => false
        }
    }

    /// Value of the state to filter
    pub fn state(&self) -> String {
        match self {
            &FilterInfo::State(ref s) => s.clone(),
            &FilterInfo::DescriptionAndState(_, ref s) => s.clone(),
            _ => s!("")
        }
    }

    /// Value of the description to filter
    pub fn description(&self) -> String {
        match self {
            &FilterInfo::Description(ref s) => s.clone(),
            &FilterInfo::DescriptionAndState(ref s, _) => s.clone(),
            _ => s!("")
        }
    }

    /// If the filter matches the interaction provider state using a regular expression. If the
    /// filter value is the empty string, then it will match interactions with no provider state.
    ///
    /// # Panics
    /// If the state filter value can't be parsed as a regular expression
    pub fn match_state(&self, interaction: &Interaction) -> bool {
        if !interaction.provider_states.is_empty() {
            if self.state().is_empty() {
                false
            } else {
                let re = Regex::new(&self.state()).unwrap();
                interaction.provider_states.iter().any(|state| re.is_match(&state.name))
            }
        } else {
            self.has_state() && self.state().is_empty()
        }
    }

    /// If the filter matches the interaction description using a regular expression
    ///
    /// # Panics
    /// If the description filter value can't be parsed as a regular expression
    pub fn match_description(&self, interaction: &Interaction) -> bool {
        let re = Regex::new(&self.description()).unwrap();
        re.is_match(&interaction.description)
    }

}

fn filter_interaction(interaction: &Interaction, filter: &FilterInfo) -> bool {
    if filter.has_description() && filter.has_state() {
      filter.match_description(interaction) && filter.match_state(interaction)
    } else if filter.has_description() {
      filter.match_description(interaction)
    } else if filter.has_state() {
      filter.match_state(interaction)
    } else {
      true
    }
}

fn filter_consumers(consumers: &Vec<String>, res: &Result<Pact, String>) -> bool {
    consumers.is_empty() || res.is_err() || consumers.contains(&res.clone().unwrap().consumer.name)
}

/// Verify the provider with the given pact sources
pub fn verify_provider(provider_info: &ProviderInfo, source: Vec<PactSource>, filter: &FilterInfo,
    consumers: &Vec<String>, runtime: &mut Runtime) -> bool {
    let pacts = source.iter().flat_map(|s| {
        match s {
            &PactSource::File(ref file) => vec![Pact::read_pact(Path::new(&file))
                .map_err(|err| format!("Failed to load pact '{}' - {}", file, err))],
            &PactSource::Dir(ref dir) => match walkdir(Path::new(dir)) {
                Ok(ref pacts) => pacts.iter().map(|p| {
                        match p {
                            &Ok(ref pact) => Ok(pact.clone()),
                            &Err(ref err) => Err(format!("Failed to load pact from '{}' - {}", dir, err))
                        }
                    }).collect(),
                Err(err) => vec![Err(format!("Could not load pacts from directory '{}' - {}", dir, err))]
            },
            &PactSource::URL(ref url, ref auth) => vec![Pact::from_url(url, auth)
                .map_err(|err| format!("Failed to load pact '{}' - {}", url, err))],
            &PactSource::BrokerUrl(ref provider_name, ref broker_url) => {
                let future = pact_broker::fetch_pacts_from_broker(broker_url.clone(), provider_name.clone());
                match runtime.block_on(future) {
                Ok(ref pacts) => pacts.iter().map(|p| {
                        match p {
                            &Ok(ref pact) => Ok(pact.clone()),
                            &Err(ref err) => Err(format!("Failed to load pact from '{}' - {:?}", broker_url, err))
                        }
                    }).collect(),
                Err(err) => vec![Err(format!("Could not load pacts from the pact broker '{}' - {:?}", broker_url, err))]
            }}
        }
    })
    .filter(|res| filter_consumers(consumers, res))
    .collect::<Vec<Result<Pact, String>>>();

    let mut verify_provider_result = true;
    let mut all_errors: Vec<(String, MismatchResult)> = vec![];
    for pact in pacts {
        match pact {
            Ok(ref pact) => {
                println!("\nVerifying a pact between {} and {}",
                    Style::new().bold().paint(pact.consumer.name.clone()),
                    Style::new().bold().paint(pact.provider.name.clone()));

                if pact.interactions.is_empty() {
                    println!("         {}", Yellow.paint("WARNING: Pact file has no interactions"));
                } else {
                    let results: HashMap<Interaction, Result<(), MismatchResult>> = pact.interactions.iter()
                    .filter(|interaction| filter_interaction(interaction, filter))
                    .map(|interaction| {
                        (interaction.clone(), verify_interaction(provider_info, interaction, runtime))
                    }).collect();

                    for (interaction, result) in results.clone() {
                        let mut description = format!("Verifying a pact between {} and {}",
                            pact.consumer.name.clone(), pact.provider.name.clone());
                        if let Some((first, elements)) = interaction.provider_states.split_first() {
                            description.push_str(&format!(" Given {}", first.name));
                            for state in elements {
                                description.push_str(&format!(" And {}", state.name));
                            }
                        }
                        description.push_str(" - ");
                        description.push_str(&interaction.description);
                        println!("  {}", interaction.description);
                        match result {
                            Ok(()) => {
                              display_result(interaction.response.status, Green.paint("OK"),
                                interaction.response.headers.map(|h| h.iter().map(|(k, v)| {
                                  (k.clone(), v.join(", "), Green.paint("OK"))
                                }).collect()), Green.paint("OK"))
                            },
                            Err(ref err) => match err {
                                &MismatchResult::Error(ref err_des) => {
                                    println!("      {}", Red.paint(format!("Request Failed - {}", err_des)));
                                    all_errors.push((description, MismatchResult::Error(err_des.clone())));
                                    verify_provider_result = false;
                                },
                                &MismatchResult::Mismatches(ref mismatches, ref expected_response, ref actual_response) => {
                                    description.push_str(" returns a response which ");
                                    let status_result = if mismatches.iter().any(|m| m.mismatch_type() == s!("StatusMismatch")) {
                                        verify_provider_result = false;
                                        Red.paint("FAILED")
                                    } else {
                                        Green.paint("OK")
                                    };
                                    let header_results = match interaction.response.headers {
                                        Some(ref h) => Some(h.iter().map(|(k, v)| {
                                          (k.clone(), v.join(", "), if mismatches.iter().any(|m| {
                                            match m {
                                              &Mismatch::HeaderMismatch{ ref key, .. } => k == key,
                                              _ => false
                                            }
                                          }) {
                                            verify_provider_result = false;
                                            Red.paint("FAILED")
                                          } else {
                                            Green.paint("OK")
                                          })
                                        }).collect()),
                                        None => None
                                    };
                                    let body_result = if mismatches.iter().any(|m| m.mismatch_type() == s!("BodyMismatch") ||
                                        m.mismatch_type() == s!("BodyTypeMismatch")) {
                                        verify_provider_result = false;
                                        Red.paint("FAILED")
                                    } else {
                                        Green.paint("OK")
                                    };

                                    display_result(interaction.response.status, status_result, header_results,
                                        body_result);

                                    for mismatch in mismatches.clone() {
                                        all_errors.push((description.clone(),
                                            MismatchResult::Mismatches(vec![mismatch.clone()],
                                                expected_response.clone(), actual_response.clone())));
                                    }
                                }
                            }
                        }
                    }
                    println!();
                }
            },
            Err(err) => {
                error!("Failed to load pact - {}", Red.paint(format!("{}", err)));
                verify_provider_result = false;
                all_errors.push((s!("Failed to load pact"), MismatchResult::Error(format!("{}", err))));
            }
        }
    };

    if !all_errors.is_empty() {
        println!("\nFailures:\n");

        for (i, &(ref description, ref mismatch)) in all_errors.iter().enumerate() {
            match mismatch {
                &MismatchResult::Error(ref err) => println!("{}) {} - {}\n", i, description, err),
                &MismatchResult::Mismatches(ref mismatch, ref expected_response, ref actual_response) => {
                    let mismatch = mismatch.first().unwrap();
                    println!("{}) {}{}", i, description, mismatch.summary());
                    println!("    {}\n", mismatch.ansi_description());

                    match mismatch {
                        &Mismatch::BodyMismatch{ref path, ..} => display_body_mismatch(expected_response, actual_response, path),
                        _ => ()
                    }
                }
            }
        }

        println!("\nThere were {} pact failures\n", all_errors.len());
    }

    verify_provider_result
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use super::{FilterInfo, filter_interaction, filter_consumers, execute_state_change, ProviderInfo};
  use pact_matching::models::*;
  use pact_matching::models::provider_states::*;
  use pact_consumer::prelude::*;
  use env_logger::*;
  use tokio::runtime::current_thread::Runtime;

    #[test]
    fn if_no_interaction_filter_is_defined_returns_true() {
        let interaction = Interaction::default();
        expect!(filter_interaction(&interaction, &FilterInfo::None)).to(be_true());
    }

    #[test]
    fn if_an_interaction_filter_is_defined_returns_false_if_the_description_does_not_match() {
        let interaction = Interaction { description: s!("bob"), .. Interaction::default() };
        expect!(filter_interaction(&interaction, &FilterInfo::Description(s!("fred")))).to(be_false());
    }

    #[test]
    fn if_an_interaction_filter_is_defined_returns_true_if_the_description_does_match() {
        let interaction = Interaction { description: s!("bob"), .. Interaction::default() };
        expect!(filter_interaction(&interaction, &FilterInfo::Description(s!("bob")))).to(be_true());
    }

    #[test]
    fn uses_regexs_to_match_the_description() {
        let interaction = Interaction { description: s!("bobby"), .. Interaction::default() };
        expect!(filter_interaction(&interaction, &FilterInfo::Description(s!("bob.*")))).to(be_true());
    }

    #[test]
    fn if_an_interaction_state_filter_is_defined_returns_false_if_the_state_does_not_match() {
        let interaction = Interaction { provider_states: vec![ ProviderState::default(&s!("bob")) ], .. Interaction::default() };
        expect!(filter_interaction(&interaction, &FilterInfo::State(s!("fred")))).to(be_false());
    }

    #[test]
    fn if_an_interaction_state_filter_is_defined_returns_true_if_the_state_does_match() {
        let interaction = Interaction { provider_states: vec![ ProviderState::default(&s!("bob")) ], .. Interaction::default() };
        expect!(filter_interaction(&interaction, &FilterInfo::State(s!("bob")))).to(be_true());
    }

    #[test]
    fn uses_regexs_to_match_the_state() {
        let interaction = Interaction { provider_states: vec![ ProviderState::default(&s!("bobby")) ], .. Interaction::default() };
        expect!(filter_interaction(&interaction, &FilterInfo::State(s!("bob.*")))).to(be_true());
    }

    #[test]
    fn if_the_state_filter_is_empty_returns_false_if_the_interaction_state_is_defined() {
        let interaction = Interaction { provider_states: vec![ ProviderState::default(&s!("bobby")) ], .. Interaction::default() };
        expect!(filter_interaction(&interaction, &FilterInfo::State(s!("")))).to(be_false());
    }

    #[test]
    fn if_the_state_filter_is_empty_returns_true_if_the_interaction_state_is_not_defined() {
        let interaction = Interaction { provider_states: vec![], .. Interaction::default() };
        expect!(filter_interaction(&interaction, &FilterInfo::State(s!("")))).to(be_true());
    }

    #[test]
    fn if_the_state_filter_and_interaction_filter_is_defined_must_match_both() {
        let interaction = Interaction { description: s!("freddy"), provider_states: vec![ ProviderState::default(&s!("bobby")) ], .. Interaction::default() };
        expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(s!(".*ddy"), s!("bob.*")))).to(be_true());
    }

    #[test]
    fn if_the_state_filter_and_interaction_filter_is_defined_is_false_if_the_provider_state_does_not_match() {
        let interaction = Interaction { description: s!("freddy"), provider_states: vec![ ProviderState::default(&s!("boddy")) ], .. Interaction::default() };
        expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(s!(".*ddy"), s!("bob.*")))).to(be_false());
    }

    #[test]
    fn if_the_state_filter_and_interaction_filter_is_defined_is_false_if_the_description_does_not_match() {
        let interaction = Interaction { description: s!("frebby"), provider_states: vec![ ProviderState::default(&s!("bobby")) ], .. Interaction::default() };
        expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(s!(".*ddy"), s!("bob.*")))).to(be_false());
    }

    #[test]
    fn if_the_state_filter_and_interaction_filter_is_defined_is_false_if_both_do_not_match() {
        let interaction = Interaction { description: s!("joe"), provider_states: vec![ ProviderState::default(&s!("author")) ], .. Interaction::default() };
        expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(s!(".*ddy"), s!("bob.*")))).to(be_false());
    }

    #[test]
    fn if_no_consumer_filter_is_defined_returns_true() {
        let consumers = vec![];
        let result = Err(s!(""));
        expect!(filter_consumers(&consumers, &result)).to(be_true());
    }

    #[test]
    fn if_a_consumer_filter_is_defined_returns_false_if_the_consumer_name_does_not_match() {
        let consumers = vec![s!("fred"), s!("joe")];
        let result = Ok(Pact { consumer: Consumer { name: s!("bob") }, .. Pact::default() });
        expect!(filter_consumers(&consumers, &result)).to(be_false());
    }

    #[test]
    fn if_a_consumer_filter_is_defined_returns_true_if_the_result_is_an_error() {
        let consumers = vec![s!("fred"), s!("joe")];
        let result = Err(s!(""));
        expect!(filter_consumers(&consumers, &result)).to(be_true());
    }

    #[test]
    fn if_a_consumer_filter_is_defined_returns_true_if_the_consumer_name_does_match() {
        let consumers = vec![s!("fred"), s!("joe"), s!("bob")];
        let result = Ok(Pact { consumer: Consumer { name: s!("bob") }, .. Pact::default() });
        expect!(filter_consumers(&consumers, &result)).to(be_true());
    }

  #[test]
  fn test_state_change_with_parameters() {
    init().unwrap_or(());

    let server = PactBuilder::new("RustPactVerifier", "SomeRunningProvider")
      .interaction("a state change request", |i| {
        i.request.method("POST");
        i.request.path("/");
        i.request.header("Content-Type", "application/json");
        i.request.body("{\"A\":\"1\",\"B\":\"2\",\"action\":\"setup\",\"state\":\"TestState\"}");
        i.response.status(200);
      })
      .start_mock_server();

    let provider_state = ProviderState {
      name: s!("TestState"),
      params: hashmap!{
        s!("A") => json!("1"),
        s!("B") => json!("2")
      }
    };

    let provider = ProviderInfo { state_change_url: Some(server.url().to_string()), .. ProviderInfo::default() };
    let result = execute_state_change(&provider_state, &provider, true, &mut Runtime::new().unwrap());
    expect!(result.clone()).to(be_ok());
  }

  #[test]
  fn test_state_change_with_parameters_in_query() {
    init().unwrap_or(());

    let server = PactBuilder::new("RustPactVerifier", "SomeRunningProvider")
      .interaction("a state change request with params in the query string", |i| {
        i.request.method("POST");
        i.request.path("/");
        i.request.query_param("state", "TestState");
        i.request.query_param("action", "setup");
        i.request.query_param("A", "1");
        i.request.query_param("B", "2");
        i.response.status(200);
      })
      .start_mock_server();

    let provider_state = ProviderState {
      name: s!("TestState"),
      params: hashmap!{
        s!("A") => json!("1"),
        s!("B") => json!("2")
      }
    };

    let provider = ProviderInfo { state_change_url: Some(server.url().to_string()),
      state_change_body: false, .. ProviderInfo::default() };
    let result = execute_state_change(&provider_state, &provider, true, &mut Runtime::new().unwrap());
    expect!(result.clone()).to(be_ok());
  }
}
