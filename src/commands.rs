use std::str;
use regex::{Regex, Captures};

#[derive(Debug, PartialEq)]
pub enum Command {
    List,
    Get(String),
    Put(String, String),
    Quit,
}

struct Rule {
    regex: Regex,
    builder : fn(Captures) -> Command,
}

impl Rule {
    fn new(regex : &str, builder: fn(Captures) -> Command) -> Self {
        Rule {
            regex : Regex::new(regex).unwrap(),
            builder: builder,
        }
    }

    // Apply the builder on regex captures
    fn apply(& self, input: &str) -> Option<Command> {
        // Capture groups using regex
        self.regex.captures(input).map(
            // And use them on our builder function
            // The extra parentheses are needed to call the builder function
            |captures| (self.builder)(captures)
        )
    }
}

// Only expose the top level parser
pub struct Parser {
    rules : [Rule; 4],
}

impl Parser {
    pub fn new() -> Self {
        // Build stored rules
        // linking a regex and a command generation function
        Parser {
            rules: [
                Rule::new(r"^LIST\n", rule_list),
                Rule::new(r"^GET (\w+)\n", rule_get),
                Rule::new(r"^PUT (\w+) (\w+)\n", rule_put),
                Rule::new(r"^QUIT\n", rule_quit),
            ],
        }
    }

    pub fn find_command(&mut self, input: &str) -> Option<Command> {
        // Remove end of line
        self.rules.into_iter().find(
            // Find first rule where the regex match
            |rule| rule.regex.is_match(input)
        ).map(
            // And transform it into a command
            |rule| rule.apply(input).unwrap()
        )
    }
}

// Where the magic happens !
fn rule_list(_ : Captures) -> Command {Command::List}
fn rule_get(captures : Captures) -> Command {
    // TODO: clean this shit
    let key = captures.get(1).unwrap().as_str();
    Command::Get(String::from(key))
}
fn rule_put(captures : Captures) -> Command {
    // TODO: clean this shit
    let key = captures.get(1).unwrap().as_str();
    let value = captures.get(2).unwrap().as_str();
    Command::Put(String::from(key), String::from(value))
}
fn rule_quit(_ : Captures) -> Command {Command::Quit}


#[test]
fn parser_get() {
    let mut p = Parser::new();
    assert_eq!(p.find_command("GET plop\n").expect("A nice get with newline"), Command::Get(String::from("plop")));
}

#[test]
fn parser_put() {
    let mut p = Parser::new();
    assert_eq!(p.find_command("PUT plop 1234a\n").expect("A nice put"), Command::Put(String::from("plop"), String::from("1234a")));
}

#[test]
fn parser_quit() {
    let mut p = Parser::new();
    assert_eq!(p.find_command("QUIT\n").expect("A nice quit"), Command::Quit);
}

#[test]
fn parser_list() {
    let mut p = Parser::new();
    assert_eq!(p.find_command("LIST\n").expect("A nice list"), Command::List);
}
