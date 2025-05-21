#[derive(Debug)]
pub enum CLElement {
    CLOption((String, Option<String>)), //Denotes command-line option with an optional argument
    CLArgument(String),                 //Denotes a free-standin argument, such as a URI for curl
    CLSep(char),                        //Denotes a separator
    ParsingError(Option<String>),       //Used to express errors in the parsing process,
    CLDoubleDash, //Used to designate free-standing a double-dash sequence, usually used to indicate that input should be read from stdin
}
