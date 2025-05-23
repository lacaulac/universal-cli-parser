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
}

#[derive(Deserialize)]
struct CharOption {
    option_name: char,
    has_arg: Option<bool>,
}

pub struct ParserConfig {
    pub string_separators: Vec<char>,
    pub char_options: Vec<(char, bool)>,
    pub string_options: Vec<(String, bool)>,
    pub has_separatorless_args_for_char_options: bool,
    pub handle_quotes: bool,
}

impl ParserConfig {
    pub fn new(
        string_separators: Vec<char>,
        char_options: Vec<(char, bool)>,
        string_options: Vec<(String, bool)>,
        has_separatorless_args_for_char_options: bool,
        handle_quotes: bool,
    ) -> ParserConfig {
        ParserConfig {
            string_separators,
            char_options,
            string_options,
            has_separatorless_args_for_char_options,
            handle_quotes,
        }
    }

    pub fn is_separator(&self, that_char: char) -> bool {
        self.string_separators.contains(&that_char)
    }

    pub fn does_char_option_have_arg(&self, option_name: &char) -> Result<bool, String> {
        for str_opt in &self.char_options {
            if str_opt.0.eq(option_name) {
                println!(
                    "\tFound char option {}, argument : {}",
                    str_opt.0, str_opt.1
                );
                return Ok(str_opt.1);
            }
        }
        Err(format!("The \"{}\" char option is unknown", option_name))
    }

    pub fn does_string_option_have_arg(&self, option_name: &String) -> Result<bool, String> {
        for str_opt in &self.string_options {
            if str_opt.0.eq(option_name) {
                println!(
                    "\tFound string option {}, argument : {}",
                    str_opt.0, str_opt.1
                );
                return Ok(str_opt.1);
            }
        }
        Err(format!("The \"{}\" string option is unknown", option_name))
    }

    pub fn from_toml_file(file_path: &str) -> Result<ParserConfig, String> {
        let contents = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        let config_file: ConfigFile = toml::from_str(&contents).map_err(|e| e.to_string())?;

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
            string_separators,
            char_options,
            string_options,
            has_separatorless_args_for_char_options: config_file
                .has_separatorless_args_for_char_options,
            handle_quotes: config_file.handle_quotes,
        })
    }
}
