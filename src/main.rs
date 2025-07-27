mod parser_config;
mod parser_structs;
use std::ops::Deref;

use parser_config::ParserConfig;
use parser_structs::CLElement;

use crate::parser_structs::CLArgument;

use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // initialize tracing
    //tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/parse", post(parse_request))
        .route("/behaviours", post(behaviours_request));

    // run our app with hyper, listening globally on port 6880
    let listener = tokio::net::TcpListener::bind("0.0.0.0:6880").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
        .expect("Failed to load config");

    let parsed_cmdline = parse_the_split(args, &parser_config);

    Ok(format!("{:?}", parsed_cmdline))
}

async fn behaviours_request(
    Json(payload): Json<ParseRequest>,
) -> Result<Json<Vec<CLElement>>, StatusCode> {
    let program = payload.program;
    let args = payload.args;

    // Perform parsing logic here
    let parser_config = ParserConfig::from_toml_file(format!("configs/{}.toml", program).as_str())
        .expect("Failed to load config");

    let parsed_cmdline = parse_the_split(args, &parser_config);
    let mut enriched_parsed_cmdline: Vec<CLElement> = Vec::new();

    for elem in &parsed_cmdline {
        let new_element: CLElement;
        //If elem is not a CLOption, just copy it into the new vector
        if let CLElement::CLOption(opt) = elem {
            //Let's get the behaviour of the option
            let behaviours = match parser_config.get_behaviours(&opt.0) {
                Ok(behaviours) => behaviours,
                Err(err) => {
                    let err_msg = format!("Error getting behaviour for option {}: {}", opt.0, err);
                    eprintln!("{}", err_msg);
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
        println!("Current index is {}/{}", idx, split_vec.len());
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
                        parsed_cmdline.push(CLElement::ParsingError(Some(err_msg)));
                        parsed_cmdline.push(CLElement::CLOption((option_name, None)));
                        idx += 1;
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
