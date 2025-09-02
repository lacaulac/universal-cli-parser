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

use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Clone)]
pub struct ConfigFile {
    name: String,
    has_separatorless_args_for_char_options: bool,
    handle_quotes: bool,
    string_separators: Vec<String>,
    string_options: Vec<StringOption>,
    char_options: Vec<CharOption>,
    pub(crate) behaviours: Vec<String>,
}

#[derive(Deserialize, Clone)]
struct StringOption {
    option_name: String,
    has_arg: Option<bool>,
    behaviours: Vec<String>,
}

#[derive(Deserialize, Clone)]
struct CharOption {
    option_name: char,
    has_arg: Option<bool>,
    behaviours: Vec<String>,
}

#[derive(Clone)]
pub struct ParserConfig {
    pub name: String,
    pub string_separators: Vec<char>,
    pub char_options: Vec<(char, bool)>,
    pub string_options: Vec<(String, bool)>,
    pub has_separatorless_args_for_char_options: bool,
    pub handle_quotes: bool,
    pub config_file: ConfigFile,
}

impl ParserConfig {
    pub fn new(
        name: String,
        string_separators: Vec<char>,
        char_options: Vec<(char, bool)>,
        string_options: Vec<(String, bool)>,
        has_separatorless_args_for_char_options: bool,
        handle_quotes: bool,
        config_file: ConfigFile,
    ) -> ParserConfig {
        ParserConfig {
            name,
            string_separators,
            char_options,
            string_options,
            has_separatorless_args_for_char_options,
            handle_quotes,
            config_file,
        }
    }

    pub fn is_separator(&self, that_char: char) -> bool {
        self.string_separators.contains(&that_char)
    }

    pub fn does_char_option_have_arg(&self, option_name: &char) -> Result<bool, String> {
        for str_opt in &self.char_options {
            if str_opt.0.eq(option_name) {
                //println!(
                //    "\tFound char option {}, argument : {}",
                //    str_opt.0, str_opt.1
                //);
                return Ok(str_opt.1);
            }
        }
        Err(format!("The \"{}\" char option is unknown", option_name))
    }

    pub fn does_string_option_have_arg(&self, option_name: &String) -> Result<bool, String> {
        for str_opt in &self.string_options {
            if str_opt.0.eq(option_name) {
                //println!(
                //    "\tFound string option {}, argument : {}",
                //    str_opt.0, str_opt.1
                //);
                return Ok(str_opt.1);
            }
        }
        Err(format!("The \"{}\" string option is unknown", option_name))
    }

    pub fn get_behaviours_for_char_option(
        &self,
        option_name: &char,
    ) -> Result<Vec<String>, String> {
        //Look in the config_file member
        for char_opt in &self.config_file.char_options {
            if char_opt.option_name.eq(option_name) {
                //println!(
                //    "\tFound char option {}, behaviours : {:?}",
                //    char_opt.option_name, char_opt.behaviours
                //);
                return Ok(char_opt.behaviours.clone());
            }
        }
        Err(format!("The \"{}\" char option is unknown", option_name))
    }

    pub fn get_behaviours_for_string_option(
        &self,
        option_name: &str,
    ) -> Result<Vec<String>, String> {
        //Look in the config_file member
        for str_opt in &self.config_file.string_options {
            if str_opt.option_name.eq(option_name) {
                //println!(
                //    "\tFound string option {}, behaviours : {:?}",
                //    str_opt.option_name, str_opt.behaviours
                //);
                return Ok(str_opt.behaviours.clone());
            }
        }
        Err(format!("The \"{}\" string option is unknown", option_name))
    }

    pub fn get_behaviours(&self, name: &String) -> Result<Vec<String>, String> {
        if name.len() == 1 {
            //It's a char option
            self.get_behaviours_for_char_option(&name.chars().next().unwrap())
        } else {
            self.get_behaviours_for_string_option(name.as_str())
        }
    }

    /// Try to split an option string by configured separators and return (option_name, argument) if found
    /// Will try all possible splits and return the longest matching option name
    pub fn try_split_embedded_option(&self, option_str: &str) -> Option<(String, String)> {
        tracing::debug!(option_str = %option_str, separators = ?self.string_separators, "Attempting to split embedded option");
        
        let mut best_match: Option<(String, String)> = None;
        let mut longest_option_len = 0;
        
        for separator in &self.string_separators {
            // Find all positions where this separator appears
            let mut start_pos = 0;
            while let Some(split_pos) = option_str[start_pos..].find(*separator) {
                let actual_split_pos = start_pos + split_pos;
                let (option_part, remaining) = option_str.split_at(actual_split_pos);
                let arg_part = &remaining[1..]; // Remove the separator character
                
                tracing::debug!(
                    option_part = %option_part, 
                    arg_part = %arg_part, 
                    separator = %separator,
                    "Found split candidate"
                );
                
                // Check if the option part (before separator) is a known string option
                if self.does_string_option_have_arg(&option_part.to_string()).is_ok() {
                    tracing::debug!(option_part = %option_part, "Split option is known");
                    
                    // Keep track of the longest matching option
                    if option_part.len() > longest_option_len {
                        longest_option_len = option_part.len();
                        best_match = Some((option_part.to_string(), arg_part.to_string()));
                        tracing::debug!(
                            option_part = %option_part, 
                            arg_part = %arg_part,
                            "New best match found (longer option name)"
                        );
                    }
                } else {
                    tracing::debug!(option_part = %option_part, "Split option is not known");
                }
                
                // Move to the next potential split position
                start_pos = actual_split_pos + 1;
            }
        }
        
        if let Some((option, arg)) = &best_match {
            tracing::debug!(option = %option, arg = %arg, "Best embedded split found");
        } else {
            tracing::debug!(option_str = %option_str, "No valid embedded split found");
        }
        
        best_match
    }

    pub fn from_toml_file(file_path: &str) -> Result<ParserConfig, String> {
        let contents = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        let config_file: ConfigFile = toml::from_str(&contents).map_err(|e| e.to_string())?;

        let name = config_file.name.clone();

        let string_separators: Vec<char> = config_file
            .string_separators
            .iter()
            .map(|s| s.chars().next().unwrap())
            .collect();

        let char_options: Vec<(char, bool)> = config_file
            .char_options
            .iter()
            .map(|opt| (opt.option_name, opt.has_arg.unwrap_or(false)))
            .collect();

        let string_options: Vec<(String, bool)> = config_file
            .string_options
            .iter()
            .map(|opt| (opt.option_name.clone(), opt.has_arg.unwrap_or(false)))
            .collect();

        Ok(ParserConfig {
            name,
            string_separators,
            char_options,
            string_options,
            has_separatorless_args_for_char_options: config_file
                .has_separatorless_args_for_char_options,
            handle_quotes: config_file.handle_quotes,
            config_file,
        })
    }
}
