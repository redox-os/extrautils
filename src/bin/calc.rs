#![deny(warnings)]
extern crate coreutils;
use coreutils::extra::OptionalExt;
use std::env::args;
use std::io;
use std::io::Write;
//use std::rand::FloatMath::*;

#[derive(Debug, Clone)]
pub enum Token {
    Plus,
    Minus,
    Divide,
    Multiply,
    Exponent,
    OpenParen,
    CloseParen,
    Number(String),
}

impl Token {
    pub fn to_str(&self) -> &'static str {
        match self {
            &Token::Plus       => "Plus",
            &Token::Minus      => "Minus",
            &Token::Divide     => "Divide",
            &Token::Multiply   => "Multiply",
            &Token::Exponent   => "Exponent",
            &Token::OpenParen  => "OpenParen",
            &Token::CloseParen => "CloseParen",
            &Token::Number(_)  => "Number",
        }
    }

    pub fn to_string(&self) -> String {
        self.to_str().to_owned()
    }
}

#[derive(Debug,  Clone)]
pub enum ParseError {
    InvalidNumber(String),
    UnrecognizedToken(String),
    UnexpectedToken(String, &'static str),
    UnexpectedEndOfInput,
    OtherError(String),
}

#[derive(Clone,Debug)]
pub struct IntermediateResult {
    value: f64,
    tokens_read: usize,
}

impl IntermediateResult {
    fn new(value: f64, tokens_read: usize) -> Self {
        IntermediateResult {
            value: value,
            tokens_read: tokens_read,
        }
    }
}

pub trait OperatorFunctions {
    fn is_operator(self) -> bool;
    fn operator_type(self) -> Token;
}

impl OperatorFunctions for char {
    fn is_operator(self) -> bool {
        self == '+' ||
        self == '-' ||
        self == '*' ||
        self == '/' ||
        self == '^' ||
        self == '(' ||
        self == ')'
    }

    fn operator_type(self) -> Token {
        match self {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '/' => Token::Divide,
            '*' => Token::Multiply,
            '^' => Token::Exponent,
            '(' => Token::OpenParen,
            ')' => Token::CloseParen,
            _   => coreutils::extra::fail("Invalid operator", &mut io::stderr())
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    // CONSIDER: any smarter way to guess an initial capacity?
    let mut tokens: Vec<Token> = Vec::with_capacity(25);

    // TODO: Not this. Modify to use iterator
    let chars: Vec<char> = input.chars().collect();

    let input_length = chars.len();
    let mut current_pos = 0;
    while current_pos < input_length {
        let c : char = chars[current_pos];
        if c.is_digit(10) {
            let token_string = consume_number(&chars[current_pos..]);
            current_pos += token_string.len();
            tokens.push(Token::Number(token_string));
        } else if c.is_operator() {
            tokens.push(c.operator_type());
            current_pos += 1;
        } else if c.is_whitespace() {
            current_pos += 1;
        } else {
            let token_string = consume_until_new_token(&chars[current_pos..]);
            return Err(ParseError::UnrecognizedToken(token_string));
        }
    }
    Ok(tokens)
}

fn consume_number(input: &[char]) -> String {
    // CONSIDER: input.len seems a bit generous
    let mut number = String::with_capacity(input.len());
    let mut has_decimal_point = false;
    for &c in input {
        if c == '.' {
            if has_decimal_point {
                break;
            } else {
                number.push(c);
                has_decimal_point = true;
            }
        } else if c.is_digit(10) {
            number.push(c);
        } else {
            break;
        }
    }
    number
}

fn consume_until_new_token(input: &[char]) -> String {
    input.iter()
         .take_while(|c| !(c.is_whitespace() || c.is_operator() || c.is_digit(10)))
         .map(|&c| c)
         .collect()
}

// Addition and subtraction
pub fn e_expr(token_list: &[Token]) -> Result<IntermediateResult, ParseError> {
    let mut t1 = try!(t_expr(token_list));
    let mut index = t1.tokens_read;

    while index < token_list.len() {
        match token_list[index] {
            Token::Plus => {
                let t2 = try!(t_expr(&token_list[index+1..]));
                t1.value += t2.value;
                t1.tokens_read += t2.tokens_read + 1;
            }
            Token::Minus => {
                let t2 = try!(t_expr(&token_list[index+1..]));
                t1.value -= t2.value;
                t1.tokens_read += t2.tokens_read + 1;
            }
            Token::Number(ref n) => return Err(ParseError::UnexpectedToken(n.clone(),"operator")),
            _ => break,
        };
        index = t1.tokens_read;
    }
    Ok(t1)
}

// Multiplication and division
pub fn t_expr(token_list: &[Token]) -> Result<IntermediateResult, ParseError> {
    let mut f1 = try!(f_expr(token_list));
    let mut index = f1.tokens_read;

    while index < token_list.len() {
        match token_list[index] {
            Token::Multiply => {
                let f2 = try!(f_expr(&token_list[index+1..]));
                f1.value *= f2.value;
                f1.tokens_read += f2.tokens_read + 1;
            }
            Token::Divide => {
                let f2 = try!(f_expr(&token_list[index+1..]));
                if f2.value == 0.0 {
                    return Err(ParseError::OtherError("Divide by zero error".to_owned()));
                } else {
                    f1.value /= f2.value;
                    f1.tokens_read += f2.tokens_read + 1;
                }
            }
            Token::Number(ref n) => return Err(ParseError::UnexpectedToken(n.clone(),"operator")),
            _ => break,
        }
        index = f1.tokens_read;
    }
    Ok(f1)
}

// Exponentiation
pub fn f_expr(token_list: &[Token]) -> Result<IntermediateResult, ParseError> {
    let mut g1 = try!(g_expr(token_list));
    let mut index = g1.tokens_read;
    let token_len = token_list.len();
    while index < token_len {
        match token_list[index] {
            Token::Exponent => {
                let f = try!(f_expr(&token_list[index+1..]));
                g1.value = g1.value.powf(f.value);
                g1.tokens_read += f.tokens_read + 1;
            }
            Token::Number(ref n) => return Err(ParseError::UnexpectedToken(n.clone(),"operator")),
            _ => break,
        }
        index = g1.tokens_read;
    }
    Ok(g1)
}

// Numbers and parenthesized expressions
pub fn g_expr(token_list: &[Token]) -> Result<IntermediateResult, ParseError> {
    if !token_list.is_empty() {
        match token_list[0] {
            Token::Number(ref n) => {
                n.parse::<f64>()
                 .map_err(|_| ParseError::InvalidNumber(n.clone()))
                 .and_then(|num| Ok(IntermediateResult::new(num, 1)))
            }
            Token::Minus => {
                if token_list.len() > 1 {
                    if let Token::Number(ref n) = token_list[1] {
                        n.parse::<f64>()
                         .map_err(|_| ParseError::InvalidNumber(n.clone()))
                         .and_then(|num| Ok(IntermediateResult::new(-1.0 * num, 2)))
                    } else {
                        Err(ParseError::UnexpectedToken(token_list[1].to_string(), "number"))
                    }
                } else {
                    Err(ParseError::UnexpectedEndOfInput)
                }
            }
            Token::OpenParen => {
                let expr = e_expr(&token_list[1..]);
                match expr {
                    Ok(ir) => {
                        let close_paren = ir.tokens_read + 1;
                        if close_paren < token_list.len() {
                            match token_list[close_paren] {
                                Token::CloseParen => Ok(IntermediateResult::new(ir.value, close_paren+1)),
                                _ => Err(ParseError::UnexpectedToken(token_list[close_paren].to_string(), ")")),
                            }
                        } else {
                            Err(ParseError::OtherError("no matching close parenthesis found.".to_owned()))
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            _ => Err(ParseError::UnexpectedToken(token_list[0].to_string(), "number"))
        }
    } else {
        Err(ParseError::UnexpectedEndOfInput)
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<String, ParseError> {
    e_expr(&tokens[..]).map(|answer| answer.value.to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_addition() {
        assert_eq!(tokenize("12+3").and_then(parse).unwrap(), "15");
    }

    #[test]
    fn addition() {
        assert_eq!(tokenize("12+3+5").and_then(parse).unwrap(), "20");
    }

    #[test]
    fn simple_subtraction() {
        assert_eq!(tokenize("12-3").and_then(parse).unwrap(), "9");
    }

    #[test]
    fn subtraction() {
        assert_eq!(tokenize("12-3-4").and_then(parse).unwrap(), "5");
    }

    #[test]
    fn mixed_addition_and_subtraction() {
        assert_eq!(tokenize("12+3-4+8-2-3").and_then(parse).unwrap(), "14");
    }

    #[test]
    fn simple_parentheses() {
        assert_eq!(tokenize("((3))").and_then(parse).unwrap(), "3");
        assert_eq!(tokenize("(12+(2+3))").and_then(parse).unwrap(), "17");
        assert_eq!(tokenize("12+(2+3)").and_then(parse).unwrap(), "17");
    }

    #[test]
    fn parentheses() {
        assert_eq!(tokenize("12+(2+(3+5))+4+(((6)))").and_then(parse).unwrap(), "32");
    }

    #[test]
    fn multiplication() {
        assert_eq!(tokenize("3*3").and_then(parse).unwrap(), "9");
        assert_eq!(tokenize("3*5").and_then(parse).unwrap(), "15");
        assert_eq!(tokenize("0*5").and_then(parse).unwrap(), "0");
        assert_eq!(tokenize("5*4*3*2*1").and_then(parse).unwrap(), "120");
        assert_eq!(tokenize("(5*4)*3*(2*1)").and_then(parse).unwrap(), "120");
    }

    #[test]
    fn division() {
        assert_eq!(tokenize("12/4").and_then(parse).unwrap(), "3");
        assert_eq!(tokenize("12/3").and_then(parse).unwrap(), "4");
        assert_eq!(tokenize("5/2").and_then(parse).unwrap(), "2.5");
        assert_eq!(tokenize("120/5/4/3/2").and_then(parse).unwrap(), "1");
        assert_eq!(tokenize("(120/5)/4/(3/2)").and_then(parse).unwrap(), "4");
    }

    #[test]
    fn exponentiation() {
        assert_eq!(tokenize("3^2").and_then(parse).unwrap(), "9");
        assert_eq!(tokenize("2^3^2").and_then(parse).unwrap(), "512");
        assert_eq!(tokenize("2^(2+1)^2").and_then(parse).unwrap(), "512");
    }
}

fn eval(input: &str) -> String {
    match tokenize(input).and_then(parse) {
        Ok(s) => s,
        Err(e) => match e {
            ParseError::InvalidNumber(s) => ["Invalid number: ", s.as_str() ].join(""),
            ParseError::UnrecognizedToken(s) => ["Unrecognized token: ", s.as_str()].join(""),
            ParseError::UnexpectedToken(found, expected) => ["Unexpected token: expected [", expected, "] but found '", found.as_str(), "'"].join(""),
            ParseError::UnexpectedEndOfInput => "Error: Unexpected end of input.".to_owned(),
            ParseError::OtherError(s) => s,
        }
    }
}

fn main() {
    let args = args();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    if args.len() > 1 {
        let input: Vec<String> = args.skip(1).collect();
        println!("{}", eval(&input.join("")));
    } else {
        let prompt = "[]> ".as_bytes();
        loop {
            stdout.write(prompt).try(&mut stderr);
            stdout.flush().try(&mut stderr);
            let mut input = String::new();
            io::stdin().read_line(&mut input)
                       .try(&mut stderr);
            match input.trim() {
                "exit" => break,
                s => {
                    stdout.write(eval(s).as_bytes()).try(&mut stderr);
                    stdout.write("\n".as_bytes()).try(&mut stderr);
                    stdout.flush().try(&mut stderr);
                },
            }
        }
    }
}
