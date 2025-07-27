use std::default;

use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub enum CLElement {
    CLOption((String, Option<CLArgument>)), //Denotes command-line option with an optional argument
    CLArgument(CLArgument), //Denotes a free-standing argument, such as a URI for curl
    CLSep(char),            //Denotes a separator
    ParsingError(Option<String>), //Used to express errors in the parsing process,
    CLDoubleDash, //Used to designate free-standing a double-dash sequence, usually used to indicate that input should be read from stdin
}

#[derive(Debug)]
pub enum CLArgument {
    String(String),
    U16(u16),
    Integer(i64),
    Float(f32),
    Boolean(bool),
    IPAddress(String),
    RemotePath(String),
    LocalPath(String),
}

impl CLArgument {
    pub fn identify_type(&mut self) {
        static REMOTE_PATH_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(([a-zA-Z]+[a-zA-Z0-9]*)@)?(([a-zA-Z0-9]+)(\\.([a-zA-Z0-9]+))*):(\/?)([a-zA-Z]|\\/)+",
            )
            .unwrap()
        });
        static LOCAL_PATH_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^((\.|\.\.)?\/)?([a-zA-Z]+[a-zA-Z0-9]*)(\/([a-zA-Z]+[a-zA-Z0-9]*))*$")
                .unwrap()
        });
        match self {
            CLArgument::String(str_val) => {
                //Check if the string is a valid IP address
                if let Ok(ip) = str_val.parse::<std::net::IpAddr>() {
                    *self = CLArgument::IPAddress(ip.to_string());
                } else if let Ok(num) = str_val.parse::<u16>() {
                    *self = CLArgument::U16(num);
                } else if let Ok(num) = str_val.parse::<i64>() {
                    *self = CLArgument::Integer(num);
                } else if let Ok(num) = str_val.parse::<f32>() {
                    *self = CLArgument::Float(num);
                } else if let Ok(bool) = str_val.parse::<bool>() {
                    *self = CLArgument::Boolean(bool);
                } else if REMOTE_PATH_REGEX.is_match(str_val) {
                    *self = CLArgument::RemotePath(str_val.clone());
                } else if LOCAL_PATH_REGEX.is_match(str_val) {
                    *self = CLArgument::LocalPath(str_val.clone());
                } else {
                    *self = CLArgument::String(str_val.clone());
                }
            }
            _ => {}
        }
    }
}
