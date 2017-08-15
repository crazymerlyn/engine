use node::{self, Node};
use css::{SimpleSelector, self};

pub struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    pub fn new(input: String) -> Parser {
        Parser {
            pos: 0,
            input: input,
        }
    }

    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: vec![],
        };

        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    // universal selector
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }
        selector
    }

    fn parse_rules(&mut self) -> Vec<css::Rule> {
        let mut rules = vec![];
        loop {
            self.consume_whitespace();
            if self.eof() { break }
            rules.push(self.parse_rule());
        }
        rules
    }

    fn parse_rule(&mut self) -> css::Rule {
        css::Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    fn parse_selectors(&mut self) -> Vec<css::Selector> {
        let mut selectors = vec![];
        loop {
            selectors.push(css::Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => break,
                c => panic!("Unexpected character {} in selector list", c),
            }
        }
        selectors.sort_by(|a,b| b.specificity().cmp(&a.specificity()));
        selectors
    }

    fn parse_declarations(&mut self) -> Vec<css::Declaration> {
        assert!(self.consume_char() == '{');
        let mut declarations = vec![];
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }
        declarations
    }

    fn parse_declaration(&mut self) -> css::Declaration {
        let property_name = self.parse_identifier();
        self.consume_whitespace();
        assert!(self.consume_char() == ':');
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        assert!(self.consume_char() == ';');

        css::Declaration {
            name: property_name,
            value: value,
        }
    }

    fn parse_value(&mut self) -> css::Value {
        match self.next_char() {
            c if c.is_digit(10) => self.parse_length(),
            '#' => self.parse_color(),
            _ => css::Value::Keyword(self.parse_identifier()),
        }
    }

    fn parse_length(&mut self) -> css::Value {
        css::Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_float(&mut self) -> f32 {
        let s = self.consume_while(|c| c.is_digit(10) || c == '.');
        s.parse().unwrap()
    }

    fn parse_unit(&mut self) -> css::Unit {
        match &*self.parse_identifier().to_lowercase() {
            "px" => css::Unit::Px,
            _ => panic!("Unrecognized unit"),
        }
    }

    fn parse_color(&mut self) -> css::Value {
        assert!(self.consume_char() == '#');
        css::Value::Color(css::Color {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255,
        })
    }

    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos..self.pos+2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    fn parse_node(&mut self) -> Node {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text(),
        }
    }

    fn parse_element(&mut self) -> Node {
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attrs();
        assert!(self.consume_char() == '>');
        
        let children = self.parse_nodes();

        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name);
        assert!(self.consume_char() == '>');

        return node::elem(tag_name, attrs, children);
    }

    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        assert!(self.consume_char() == '=');
        let value = self.parse_attr_value();
        return (name, value);
    }

    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert!(self.consume_char() == open_quote);
        return value;
    }

    fn parse_attrs(&mut self) -> node::AttrMap {
        let mut attrs = node::AttrMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attrs.insert(name, value);
        }
        return attrs;
    }

    fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        return nodes;
    }

    fn parse_text(&mut self) -> Node {
        node::text(self.consume_while(|c| c != '<'))
    }

    fn parse_tag_name(&mut self) -> String {
        self.consume_while(char::is_alphanumeric)
    }

    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        return cur_char;
    }

    fn consume_while<F>(&mut self, test: F) -> String 
            where F: Fn(char) -> bool {
        let mut res = String::new();
        while !self.eof() && test(self.next_char()) {
            res.push(self.consume_char());
        }
        res
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

fn valid_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}

pub fn parse(input: String) -> Node {
    let mut nodes = Parser::new(input).parse_nodes();
    if nodes.len() == 1 {
        nodes.swap_remove(0)
    } else {
        node::elem("html".to_string(), node::AttrMap::new(), nodes)
    }
}

