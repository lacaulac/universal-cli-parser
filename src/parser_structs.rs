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

use std::default;

use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub enum CLElement {
    CLOption((String, Option<CLArgument>)), //Denotes command-line option with an optional argument
    CLBehaviouredOption((String, Vec<String>, Option<CLArgument>)), //Denotes command-line option with a list of behaviours and an optional argument
    CLInherentBehaviour(Vec<String>), //Denotes an inherent behaviour of the program
    CLArgument(CLArgument),           //Denotes a free-standing argument, such as a URI for curl
    CLSep(char),                      //Denotes a separator
    ParsingError(Option<String>),     //Used to express errors in the parsing process,
    CLDoubleDash, //Used to designate free-standing a double-dash sequence, usually used to indicate that input should be read from stdin
}

#[derive(Debug, Serialize, Clone)]
pub enum CLArgument {
    String(String),
    U16(u16),
    Integer(i64),
    Float(f32),
    Boolean(bool),
    IPAddress(String),
    RemotePath(String),
    LocalPath(String),
    URL(String),
}

impl CLArgument {
    pub fn identify_type(&mut self) {
        static REMOTE_PATH_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(([a-zA-Z]+[a-zA-Z0-9]*)@)?(([a-zA-Z0-9]+)(\\.([a-zA-Z0-9]+))*):(\/?)([a-zA-Z]|\\/)+",
            )
            .unwrap()
        });
        //FIXME Very approximate regex. Matches absolute paths, some relative paths (with either ../ or ./ at the beginning) as well as "path-less" filenames with a few file extensions
        static LOCAL_PATH_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(((\.|\.\.)?\/)([a-zA-Z]+[a-zA-Z0-9]*)(\/([a-zA-Z]+[a-zA-Z0-9]*))*)|(([a-zA-Z]+[a-zA-Z0-9]*)\.((tar)|(tar\.gz)|(tar\.bz2)|(png)|(sh)))$")
                .unwrap()
        });

        static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"https?:\/\/([a-z][a-z0-9-]*\.)?([a-z][a-z0-9-]*\.)([a-z][a-z0-9-]*)(\/([a-zA-Z0-9-.]*))*")
                .unwrap()
        });

        static IPV4_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}(:[0-9]{1,5})?$").unwrap() // From https://stackoverflow.com/a/36760050
        });

        static IPV6_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))(:[0-9]{1,5})?$")
                .unwrap() //From https://stackoverflow.com/a/17871737
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
                } else if URL_REGEX.is_match(str_val) {
                    *self = CLArgument::URL(str_val.clone());
                } else if IPV4_REGEX.is_match(str_val) || IPV6_REGEX.is_match(str_val) {
                    *self = CLArgument::IPAddress(str_val.clone());
                } else {
                    *self = CLArgument::String(str_val.clone());
                }
            }
            _ => {}
        }
    }
}
