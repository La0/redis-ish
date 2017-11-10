use std::str;
use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum Command {
    List,
    Get(String),
    Put(String, String),
    Quit,
}

struct Rule {
    regex: Regex,
    builder : fn(Vec<String>) -> Command,
}

impl Rule {
    fn new(regex : &str, builder: fn(Vec<String>) -> Command) -> Self {
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
            |captures| (self.builder)(captures.iter().map(
                |cap| match cap {
                    Some(m) => String::from(m.as_str()),
                    None => String::new(),
                }
            ).collect())
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
fn rule_list(_ : Vec<String>) -> Command {Command::List}
fn rule_get(items : Vec<String>) -> Command {
    Command::Get(items[1].clone())
}
fn rule_put(items: Vec<String>) -> Command {
    Command::Put(items[1].clone(), items[2].clone())
}
fn rule_quit(_ : Vec<String>) -> Command {Command::Quit}


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
