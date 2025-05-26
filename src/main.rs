mod parser_config;
mod parser_structs;
use parser_config::ParserConfig;
use parser_structs::CLElement;

fn main() {
    let cmd_line = "-kv \"haaaa haaa\" --data thisistestdata";

    let parser_config =
        ParserConfig::from_toml_file("configs/curl.toml").expect("Failed to load config");

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
                                        get_argument_string(&parser_config, &split_vec, idx + 2);
                                    match arg_str_res {
                                        Ok((argument_string, new_idx)) => {
                                            idx_replacement = Some(new_idx);
                                            parsed_cmdline.push(CLElement::CLOption((
                                                option_name.to_string(),
                                                Some(argument_string),
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
                            idx += 2; //Skip the separator
                            let arg_str_res = get_argument_string(&parser_config, &split_vec, idx);
                            match arg_str_res {
                                Ok((argument_string, new_idx)) => {
                                    idx = new_idx;
                                    parsed_cmdline.push(CLElement::CLOption((
                                        option_name,
                                        Some(argument_string),
                                    )));
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
                    parsed_cmdline.push(CLElement::CLArgument(arg_str));
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
        return Err(format!(
            "get_argument_string attempted to index element {} of a split_vec of size {}",
            idx,
            split_vec.len()
        ));
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
                arg_string_buffer.push_str((next_string));
                idx += 1;
            }
        }
    }

    Ok((arg_string_buffer, idx))
}
