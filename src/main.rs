/*
   Copyright (C) 2025  Antonin Verdier & Institut de Recherche en Informatique de Toulouse

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

mod config_cache;
mod parser_config;
mod parser_structs;
use config_cache::ParserConfigCache;
use std::ops::Deref;

use parser_config::ParserConfig;
use parser_structs::CLElement;

use crate::parser_structs::CLArgument;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

const APP_NAME: &str = "universal-cli-parser";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const PORT_NUMBER: u16 = 6880;

// ... cache moved to `src/config_cache.rs`

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    println!("{APP_NAME} version: {APP_VERSION}");

    // Create the parser config cache
    let config_cache = ParserConfigCache::new();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/parse", post(parse_request))
        .route("/behaviours", post(behaviours_request))
        .with_state(config_cache);

    // run our app with hyper, listening globally on port 6880
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{PORT_NUMBER}"))
        .await
        .unwrap();
    println!("Starting server on port {PORT_NUMBER}");
    axum::serve(listener, app).await.unwrap();
    println!("Bye !");
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn parse_request(Json(payload): Json<ParseRequest>) -> Result<String, StatusCode> {
    let program = payload.program;
    let args = payload.args;

    // Perform parsing logic here
    let parser_config = ParserConfig::from_toml_file(format!("configs/{}.toml", program).as_str())
        .expect(format!("Failed to load config for program {}", program).as_str());

    let parsed_cmdline = parse_the_split(args, &parser_config);

    Ok(format!("{:?}", parsed_cmdline))
}

async fn behaviours_request(
    State(cache): State<ParserConfigCache>,
    Json(payload): Json<ParseRequest>,
) -> Result<Json<Vec<CLElement>>, StatusCode> {
    let start_time = std::time::Instant::now();
    let program = payload.program;
    let args = payload.args;

    // Get parser config from cache or load from filesystem (Arc<ParserConfig>)
    let parser_config_arc = match cache.get_config(&program) {
        Ok(arc) => arc,
        Err(_err) => {
            let elapsed = start_time.elapsed();
            tracing::warn!(
                duration_us = elapsed.as_micros(),
                program = %program,
                "/behaviours : Failed to load config for program"
            );
            return Err(StatusCode::NOT_FOUND);
        }
    };

    //Debug display of the request contents
    tracing::debug!(program = %program, args = ?args, "/behaviours : Received request");
    // Use a reference to the ParserConfig inside the Arc
    let parser_config_ref: &ParserConfig = parser_config_arc.as_ref();
    let parsed_cmdline = parse_the_split(args, parser_config_ref);
    let mut enriched_parsed_cmdline: Vec<CLElement> = Vec::new();

    //Add the inherent behaviours of the program
    let mut inherent_behaviours: Vec<String> = vec![];

    if parser_config_ref.config_file.behaviours.len() > 0 {
        for behaviour in &parser_config_ref.config_file.behaviours {
            inherent_behaviours.push(behaviour.clone());
        }

        let new_element = CLElement::CLInherentBehaviour(inherent_behaviours);

        enriched_parsed_cmdline.push(new_element);
    }

    for elem in &parsed_cmdline {
        let new_element: CLElement;
        //If elem is not a CLOption, just copy it into the new vector
        if let CLElement::CLOption(opt) = elem {
            //Let's get the behaviour of the option
            let behaviours = match parser_config_ref.get_behaviours(&opt.0) {
                Ok(behaviours) => behaviours,
                Err(err) => {
                    let elapsed = start_time.elapsed();
                    tracing::error!(
                        duration_us = elapsed.as_micros(),
                        program = %program,
                        option = %opt.0,
                        error = %err,
                        "/behaviours : Error getting behaviour for option"
                    );
                    return Err(StatusCode::IM_A_TEAPOT);
                }
            };
            new_element =
                CLElement::CLBehaviouredOption((opt.0.clone(), behaviours, opt.1.clone()));
        } else {
            new_element = elem.clone();
        }
        enriched_parsed_cmdline.push(new_element);
    }

    let elapsed = start_time.elapsed();
    tracing::info!(
        duration_us = elapsed.as_micros(),
        program = %program,
        "/behaviours : SUCCESS"
    );
    // println!("Dealt with a behaviour parsing request");

    Ok(Json(enriched_parsed_cmdline))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct ParseRequest {
    program: String,
    args: Vec<String>,
}

fn old_main() {
    let cmd_line = "-xvf remotehost:test.tar.gz --rsh-command=/bin/ssh";

    let parser_config =
        ParserConfig::from_toml_file("configs/tar.toml").expect("Failed to load config");

    let split_vec = get_split_vec(cmd_line, &parser_config);

    let parsed_cmdline = parse_the_split(split_vec, &parser_config);
    println!("{} =>", cmd_line);
    for elem in &parsed_cmdline {
        println!("\t\"{:?}\"", elem);
    }
}

pub fn parse_the_split(split_vec: Vec<String>, parser_config: &ParserConfig) -> Vec<CLElement> {
    let mut idx = 0; //Index into the split
    let mut parsed_cmdline: Vec<CLElement> = vec![];
    loop {
        //println!("Current index is {}/{}", idx, split_vec.len());
        if idx >= split_vec.len() {
            break;
        }
        let pointed_str: &String = split_vec
            .get(idx)
            .expect("split_vec.len() must have returned a wrong value");
        //If there's a single dash
        //TODO Watch out for anomic CLI programs (e.g., "-cache" instead of "--cache")
        let first_char_is_dash: bool = pointed_str.starts_with("-");
        let two_first_char_are_dashes: bool = pointed_str.starts_with("--");

        //Is it an option?
        if first_char_is_dash {
            if !two_first_char_are_dashes {
                //It's a char option block
                //Let's get the individual chars
                let mut option_names_vec = Vec::new();
                option_names_vec.extend(pointed_str.clone().split_off(1).chars());
                let mut opt_idx: usize = 0;
                let mut idx_replacement = None; //Can be assigned something when a char option has an argument => We should consume it
                loop {
                    if opt_idx >= option_names_vec.len() {
                        break;
                    }
                    let option_name: &char = option_names_vec.get(opt_idx).unwrap();
                    let does_opt_have_arg = parser_config.does_char_option_have_arg(option_name);
                    match does_opt_have_arg {
                        Ok(has_arg) => {
                            if has_arg {
                                if opt_idx + 1 < option_names_vec.len() {
                                    if parser_config.has_separatorless_args_for_char_options {
                                        todo!("Handle stuff like \"-n5\"")
                                    } else {
                                        parsed_cmdline.push(CLElement::ParsingError(Some(format!("The following char option ('{}') was supposed to take an argument but was not the last char option of its option list (\"{}\"", option_name, pointed_str))));
                                        parsed_cmdline.push(CLElement::CLOption((
                                            option_name.to_string(),
                                            None,
                                        )));
                                    }
                                } else {
                                    //Get the argument that comes after it like we would for a string option
                                    let arg_str_res =
                                        get_argument_string(&parser_config, &split_vec, idx + 1);
                                    match arg_str_res {
                                        Ok((argument_string, new_idx)) => {
                                            idx_replacement = Some(new_idx);
                                            let mut argument = CLArgument::String(argument_string);
                                            argument.identify_type();
                                            parsed_cmdline.push(CLElement::CLOption((
                                                option_name.to_string(),
                                                Some(argument),
                                            )));
                                        }
                                        Err(err_msg) => {
                                            parsed_cmdline
                                                .push(CLElement::ParsingError(Some(err_msg)));
                                            parsed_cmdline.push(CLElement::CLOption((
                                                option_name.to_string(),
                                                None,
                                            )));
                                            idx_replacement = Some(idx + 3);
                                        }
                                    }
                                }
                            } else {
                                parsed_cmdline
                                    .push(CLElement::CLOption((option_name.to_string(), None)))
                            }
                        }
                        Err(err_msg) => {
                            parsed_cmdline.push(CLElement::ParsingError(Some(err_msg)));
                            parsed_cmdline
                                .push(CLElement::CLOption((option_name.to_string(), None)));
                        }
                    }

                    opt_idx += 1;
                }
                idx = idx_replacement.unwrap_or(idx + 1);
            } else {
                if pointed_str.len() == 2 {
                    //This is a double-dash
                    parsed_cmdline.push(CLElement::CLDoubleDash);
                    idx += 1;
                    continue;
                }
                //It's a string option, get everything after the two dashes
                let option_name: String = pointed_str.clone().split_off(2);
                let does_opt_have_arg = parser_config.does_string_option_have_arg(&option_name);
                match does_opt_have_arg {
                    Ok(has_argument) => {
                        if has_argument {
                            idx += 1; //Skip the separator
                            let arg_str_res = get_argument_string(&parser_config, &split_vec, idx);
                            match arg_str_res {
                                Ok((argument_string, new_idx)) => {
                                    idx = new_idx;
                                    let mut argument = CLArgument::String(argument_string);
                                    argument.identify_type();
                                    parsed_cmdline
                                        .push(CLElement::CLOption((option_name, Some(argument))));
                                }
                                Err(err_msg) => {
                                    parsed_cmdline.push(CLElement::ParsingError(Some(err_msg)));
                                    parsed_cmdline.push(CLElement::CLOption((option_name, None)));
                                    idx += 1;
                                }
                            }
                        } else {
                            parsed_cmdline.push(CLElement::CLOption((option_name, None)));
                            idx += 1;
                        }
                    }
                    Err(err_msg) => {
                        tracing::debug!(option_name = %option_name, "String option not recognized, trying embedded separator split");

                        // Try to split by embedded separators before giving up
                        if let Some((split_option, split_arg)) =
                            parser_config.try_split_embedded_option(&option_name)
                        {
                            tracing::debug!(
                                original_option = %option_name,
                                split_option = %split_option,
                                split_arg = %split_arg,
                                "Successfully split embedded option"
                            );

                            // Found a valid split! Check if this split option expects an argument
                            match parser_config.does_string_option_have_arg(&split_option) {
                                Ok(true) => {
                                    // The split option expects an argument, use the split result
                                    let mut argument = CLArgument::String(split_arg);
                                    argument.identify_type();
                                    parsed_cmdline
                                        .push(CLElement::CLOption((split_option, Some(argument))));
                                    idx += 1;
                                }
                                Ok(false) => {
                                    // The split option doesn't expect an argument, but we found one embedded
                                    // This is likely a parsing error, but let's be permissive and use the split anyway
                                    let mut argument = CLArgument::String(split_arg);
                                    argument.identify_type();
                                    parsed_cmdline
                                        .push(CLElement::CLOption((split_option, Some(argument))));
                                    idx += 1;
                                }
                                Err(_) => {
                                    // This shouldn't happen since try_split_embedded_option already checked it
                                    parsed_cmdline.push(CLElement::ParsingError(Some(err_msg)));
                                    parsed_cmdline.push(CLElement::CLOption((option_name, None)));
                                    idx += 1;
                                }
                            }
                        } else {
                            // No valid split found, proceed with original error
                            tracing::debug!(option_name = %option_name, "No valid embedded split found, treating as unknown option");
                            parsed_cmdline.push(CLElement::ParsingError(Some(err_msg)));
                            parsed_cmdline.push(CLElement::CLOption((option_name, None)));
                            idx += 1;
                        }
                    }
                }
            }
        } else if pointed_str.len() == 1
            && parser_config.is_separator(pointed_str.chars().nth(0).unwrap())
        {
            //This is a separator
            parsed_cmdline.push(CLElement::CLSep(pointed_str.chars().nth(0).unwrap()));
            idx += 1;
        } else {
            //It's a free-standing argument, let's retrieve it
            match get_argument_string(&parser_config, &split_vec, idx) {
                Ok((arg_str, new_idx)) => {
                    let mut argument = CLArgument::String(arg_str);
                    argument.identify_type();
                    parsed_cmdline.push(CLElement::CLArgument(argument));
                    idx = new_idx;
                }
                Err(err_str) => parsed_cmdline.push(CLElement::ParsingError(Some(err_str))),
            }
        }
    }
    parsed_cmdline
}

pub fn get_split_vec(cmd_line: &str, parser_config: &ParserConfig) -> Vec<String> {
    let mut split_vec: Vec<String> = vec![];

    let mut current_str_buffer: String = String::new();
    for current_char in cmd_line.chars() {
        //If the current char is a separator
        //Or if it is a space character
        if parser_config.is_separator(current_char) || current_char == ' ' {
            //Push the buffer into split_vec
            split_vec.push(current_str_buffer);
            //Create a string with only our separator and push it into split_vec
            current_str_buffer = String::new();
            current_str_buffer.push(current_char);
            split_vec.push(current_str_buffer);
            //Reset the buffer
            current_str_buffer = String::new();
        } else {
            current_str_buffer.push(current_char);
        }
    }
    if current_str_buffer.len() > 0 {
        split_vec.push(current_str_buffer);
    }
    split_vec
}

pub fn get_argument_string(
    parser_config: &ParserConfig,
    split_vec: &Vec<String>,
    idx: usize,
) -> Result<(String, usize), String> {
    let mut arg_string_buffer: String = String::new();
    let mut idx = idx;
    if idx >= split_vec.len() {
        let err = format!(
            "get_argument_string attempted to index element {} of a split_vec of size {}",
            idx,
            split_vec.len()
        );
        eprintln!("{}", err);
        return Err(err);
    }

    //TODO Handle quotes
    let obtained_string = split_vec.get(idx).unwrap();
    if !obtained_string.starts_with("\"") || !parser_config.handle_quotes {
        arg_string_buffer.push_str(obtained_string.as_str());
        idx += 1;
    } else {
        arg_string_buffer.push_str(obtained_string.as_str().split_at(1).1); //Push the first part without its quote
        idx += 1;
        loop {
            let next_string = split_vec.get(idx).unwrap();
            let next_string = next_string.as_str();
            if next_string.ends_with("\"") {
                //This is the last one, copy it without its last char
                let next_string = next_string.split_at(next_string.len() - 1).0;
                arg_string_buffer.push_str(next_string);
                idx += 1;
                break;
            } else {
                arg_string_buffer.push_str(next_string);
                idx += 1;
            }
        }
    }

    Ok((arg_string_buffer, idx))
}
