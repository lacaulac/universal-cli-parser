use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct ConfigFile {
    name: String,
    has_separatorless_args_for_char_options: bool,
    handle_quotes: bool,
    string_separators: Vec<String>,
    string_options: Vec<StringOption>,
    char_options: Vec<CharOption>,
}

#[derive(Deserialize)]
struct StringOption {
    option_name: String,
    has_arg: Option<bool>,
    behaviours: Vec<String>,
}

#[derive(Deserialize)]
struct CharOption {
    option_name: char,
    has_arg: Option<bool>,
    behaviours: Vec<String>,
}

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
