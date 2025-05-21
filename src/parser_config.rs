pub struct ParserConfig {
    pub string_separators: Vec<char>,
    pub char_options: Vec<(char, bool)>,
    pub string_options: Vec<(String, bool)>,
    pub has_separatorless_args_for_char_options: bool, //Like "tail -n5 /etc/passwd" instead of "tail -n 5 /etc/passwd"
}

impl ParserConfig {
    pub fn new(
        string_separators: Vec<char>,
        char_options: Vec<(char, bool)>,
        string_options: Vec<(String, bool)>,
        has_separatorless_args_for_char_options: bool,
    ) -> ParserConfig {
        let tmp_parser_cfg: ParserConfig = ParserConfig {
            string_separators: string_separators,
            char_options: char_options,
            string_options: string_options,
            has_separatorless_args_for_char_options: has_separatorless_args_for_char_options,
        };
        tmp_parser_cfg
    }

    pub fn is_separator(self: &ParserConfig, that_char: char) -> bool {
        self.string_separators.contains(&that_char)
    }

    pub fn does_char_option_have_arg(
        self: &ParserConfig,
        option_name: &char,
    ) -> Result<bool, String> {
        for str_opt in &self.char_options {
            if str_opt.0.eq(option_name) {
                println!(
                    "\tFound char option {}, argument : {}",
                    str_opt.0, str_opt.1
                );
                return Ok(str_opt.1);
            }
        }
        return Err(format!("The \"{}\" char option is unknown", option_name));
    }

    pub fn does_string_option_have_arg(
        self: &ParserConfig,
        option_name: &String,
    ) -> Result<bool, String> {
        for str_opt in &self.string_options {
            if str_opt.0.eq(option_name) {
                println!(
                    "\tFound string option {}, argument : {}",
                    str_opt.0, str_opt.1
                );
                return Ok(str_opt.1);
            }
        }
        return Err(format!("The \"{}\" string option is unknown", option_name));
    }
}
