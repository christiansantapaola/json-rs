use std::{alloc::System, collections::HashMap, fmt::Error, ptr::null, slice::SliceIndex};

// TODO
// HANDLING ERROR

#[derive(Debug)]
enum Value {
    String {value: String},
    Number {value: f64},
    Array {value: Vec<Value>},
    Object {value: JSON},
    Boolean {value: bool},
    Null
}

#[derive(Debug)]
struct JSON {
    json: HashMap<String, Value>
}

#[derive(Debug)]
enum Token {
    CurlyParOpen,
    CurlyParClose,
    SquareParOpen,
    SquareParClose,
    Whitespace,
    Comma,
    Colons,
    Quote
}

impl Token {
    fn get_token(&self) -> char {
        match *self {
            Self::CurlyParOpen => '{',
            Self::CurlyParClose => '}',
            Self::SquareParOpen => '[',
            Self::SquareParClose => ']',
            Self::Whitespace => ' ',
            Self::Comma => ',',
            Self::Colons => ':',
            Self::Quote => '"',
        }
    }
    
}

#[derive(Debug)]
struct EntryIter {
    content: String,
    index: usize,
    content_len: usize
}

impl EntryIter {
    pub fn iter(content: &str) -> EntryIter {
        EntryIter{content: String::from(content), index: 0, content_len: content.char_indices().count()}
    }
    
}

fn count_parens(string: &str) -> i64 {
    let mut parens = 0;
    for c in string.chars().into_iter() {
        if c == Token::SquareParOpen.get_token() {
            parens += 1;
        }
        if c == Token::SquareParClose.get_token() {
            parens -= 1;
        }
    }
    return parens;
}

fn find_not_between(string: &str, pat: char, open: char, close: char) -> Option<usize> {
    let mut state: i64 = 0;
    let mut index = 0;
    for c in string.chars().into_iter() {
        if c == open {
            state += 1;
        }
        if c == close {
            state -= 1;
        }
        if c == pat && state == 0 {
            return Some(index);
        }
        index += 1;
    }
    return None;

}


impl Iterator for EntryIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.content_len {
            return None;
        }   
        let content_left: String = self.content.char_indices().filter(|&(i, _)| i >= self.index && i < self.content_len).map(|(_, c)| c).collect();
        match find_not_between(&content_left, Token::Comma.get_token(), Token::SquareParOpen.get_token(), Token::SquareParClose.get_token()) {
            Some(comma_index) => {
                let res = self.content.char_indices().filter(|&(i, _)| i >= self.index && i < self.index + comma_index).map(|(_, c)| c).collect();
                self.index += comma_index + 1;
                println!("{}", res);
                Some(res)
            },
            None => {
                let content_left_len = content_left.chars().count();
                self.index += content_left_len;
                Some(content_left)
            }
        }
    }
}

// 

impl JSON {
    fn parse(content: String) -> JSON {
        assert!(content.len() > 0);
        let content_trim = content.trim();
        assert!(content_trim.len() > 0);
        assert_eq!(content_trim.chars().nth(0).unwrap(), Token::CurlyParOpen.get_token());
        assert_eq!(content_trim.chars().nth(content.len() - 1).unwrap(), Token::CurlyParClose.get_token());
        let content_entriens: String = content.char_indices().filter(|&(i, _)| i > 0 && i < content.len() - 1).map(|(_, c)| c).collect();
        let mut entries: Vec<String> = Vec::new();
        let iter = EntryIter::iter(&content_entriens);
        for entry in iter {
            entries.push(entry);
        }
        let mut map = HashMap::<String, Value>::new();
        for entry in entries {
            let (key, value) = JSON::parse_entry(entry.to_string());
            map.insert(key, value);
        }
        JSON { json: map }
    }

    fn parse_key(key: String) -> String {
        let len = key.trim().chars().count();
        assert!(len > 0);
        assert!(key.trim().chars().nth(0).unwrap() == Token::Quote.get_token());
        assert!(key.trim().chars().nth(len - 1).unwrap() == Token::Quote.get_token());

        key.trim().char_indices().filter(|&(i, c)| !(i == 0 && c == Token::Quote.get_token())).filter(|&(i, c)| !(i == len - 1 && c == Token::Quote.get_token())).map(|(_, c)| c).collect()
    }

    fn parse_entry(content: String) -> (String, Value) {
        let colon_index = find_not_between(&content, Token::Colons.get_token(), Token::Quote.get_token(), Token::Quote.get_token()).unwrap();
        let key: String = content.char_indices().filter(|&(i, _)| i < colon_index).map(|(_, c)|c).collect();
        let value: String = content.char_indices().filter(|&(i, _)| i > colon_index).map(|(_, c)|c).collect();
        (JSON::parse_key(key), JSON::parse_value(value))
    }

    fn parse_value(content: String) -> Value {
        let val = content.trim();
        match val {
            "null" => Value::Null,
            "true" => Value::Boolean { value: true },
            "false" => Value::Boolean { value: false },
            _ => {
                if val.len() < 1 {
                    panic!("empty value");
                }
                let first_char = val.chars().nth(0).unwrap();
                match first_char {
                    '"' => JSON::parse_string(&val.to_string()),
                    '[' => JSON::parse_array(&val.to_string()),
                    '{' => Value::Object { value: JSON::parse(val.to_string()) },
                    _ => JSON::parse_number(&val.to_string())
                }

            }
        }
    }

    fn parse_number(content: &String) -> Value {
        let number = content.parse::<f64>();
        match  number {
            Ok(num) => Value::Number { value: num },
            Err(err) => panic!("{} {}",content, err)
        }
    }

    fn parse_string(content: &String) -> Value {
        assert!(content.len() > 0);
        assert_eq!(content.chars().nth(0).unwrap(), Token::Quote.get_token());
        assert_eq!(content.chars().last().unwrap(), Token::Quote.get_token());
        Value::String { value: content.trim().get(1..(content.len()-1)).unwrap().to_string() }
    }

    fn parse_array(content: &String) -> Value {
        Value::Array { value: content.get(1..(content.len()-1)).unwrap().split(Token::Comma.get_token()).map(String::from).map(JSON::parse_value).collect() }
    }

}


fn main() {
    // println!("{:#?}", JSON::parse_entry(String::from("\"key\": \"value\"")));
    // println!("{:#?}", JSON::parse_entry(String::from("\"key\": \"123\"")));
    // println!("{:#?}", JSON::parse_entry(String::from("\"key\": 123")));
    // println!("{:#?}", JSON::parse_entry(String::from("\"key\": -123")));
    // println!("{:#?}", JSON::parse_entry(String::from("\"key\": -0.123")));
    // println!("{:#?}", JSON::parse_entry(String::from("\"key\": true")));
    // println!("{:#?}", JSON::parse_entry(String::from("\"key\": false")));
    // println!("{:#?}", JSON::parse_entry(String::from("\"key\": null")));
    // println!("{:#?}", JSON::parse_entry(String::from("\"key\": [1,2,3]")));
    // println!("{:#?}", JSON::parse_entry(String::from("\"key\": [1, 2, \"3\"]")));
    // println!("{:#?}", JSON::parse(String::from("{\"key\": \"value\", \"key1\": 123, \"key2\": {\"key3\": [1,2,3]}}")));
    println!("{:#?}", JSON::parse(String::from("{\"key\": \"value\",\n \"key1\": 123,\n \"key2\": {\n\"key3\": [1,2,3]}}")));

}
