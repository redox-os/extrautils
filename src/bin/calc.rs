extern crate extra;
use extra::io::{fail, WriteExt};
use extra::option::OptionalExt;

use std::env::args;
use std::fmt;
use std::io::{self, Write};

#[derive(Debug, Clone)]
pub enum Token {
    Plus,
    Minus,
    Divide,
    Multiply,
    Exponent,
    OpenParen,
    CloseParen,
    Comma,
    Number(String),
    Identificator(String),
}

impl Token {
    pub fn to_str(&self) -> &'static str {
        match self {
            Token::Plus => "Plus",
            Token::Minus => "Minus",
            Token::Divide => "Divide",
            Token::Multiply => "Multiply",
            Token::Exponent => "Exponent",
            Token::OpenParen => "OpenParen",
            Token::CloseParen => "CloseParen",
            Token::Comma => "comma",
            Token::Number(_) => "Number",
            Token::Identificator(_) => "Identificator",
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidNumber(String),
    UnrecognizedToken(String),
    UnexpectedToken(String, &'static str),
    UnexpectedEndOfInput,
    UnexpectedNumberOfArgs(usize, usize),
    OtherError(String),
}

#[derive(Clone, Debug)]
pub struct IntermediateResult {
    value: f64,
    tokens_read: usize,
}

impl IntermediateResult {
    fn new(value: f64, tokens_read: usize) -> Self {
        IntermediateResult { value, tokens_read }
    }
}

pub trait OperatorFunctions {
    fn is_operator(self) -> bool;
    fn operator_type(self) -> Token;
}

impl OperatorFunctions for char {
    fn is_operator(self) -> bool {
        self == '+'
            || self == '-'
            || self == '*'
            || self == '/'
            || self == '^'
            || self == '('
            || self == ')'
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
            _ => fail("Invalid operator", &mut io::stderr()),
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::with_capacity(input.len());

    // TODO: Not this. Modify to use iterator
    let chars: Vec<char> = input.chars().collect();

    let input_length = chars.len();
    let mut current_pos = 0;
    while current_pos < input_length {
        let c = chars[current_pos];
        if c.is_digit(10) || c == '.' {
            let token_string = consume_number(&chars[current_pos..]);
            current_pos += token_string.len();
            tokens.push(Token::Number(token_string));
        } else if c.is_operator() {
            tokens.push(c.operator_type());
            current_pos += 1;
        } else if c.is_whitespace() {
            current_pos += 1;
        } else if c.is_alphabetic() {
            let token_string = consume_ident(&chars[current_pos..]);
            current_pos += token_string.len();
            tokens.push(Token::Identificator(token_string));
        } else if c == ',' {
            tokens.push(Token::Comma);
            current_pos += 1;
        } else {
            let token_string = consume_until_new_token(&chars[current_pos..]);
            return Err(ParseError::UnrecognizedToken(token_string));
        }
    }
    Ok(tokens)
}

fn consume_number(input: &[char]) -> String {
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

fn consume_ident(input: &[char]) -> String {
    let mut ident = String::with_capacity(input.len());
    for &c in input {
        if c.is_alphanumeric() {
            ident.push(c);
        } else {
            break;
        }
    }
    ident
}

fn consume_until_new_token(input: &[char]) -> String {
    input
        .iter()
        .take_while(|c| !(c.is_whitespace() || c.is_operator() || c.is_digit(10)))
        .copied()
        .collect()
}

// Addition and subtraction
pub fn e_expr(token_list: &[Token]) -> Result<IntermediateResult, ParseError> {
    let mut t1 = t_expr(token_list)?;
    let mut index = t1.tokens_read;

    while index < token_list.len() {
        match token_list[index] {
            Token::Plus => {
                let t2 = t_expr(&token_list[index + 1..])?;
                t1.value += t2.value;
                t1.tokens_read += t2.tokens_read + 1;
            }
            Token::Minus => {
                let t2 = t_expr(&token_list[index + 1..])?;
                t1.value -= t2.value;
                t1.tokens_read += t2.tokens_read + 1;
            }
            Token::Number(ref n) => return Err(ParseError::UnexpectedToken(n.clone(), "operator")),
            _ => break,
        };
        index = t1.tokens_read;
    }
    Ok(t1)
}

// Multiplication and division
pub fn t_expr(token_list: &[Token]) -> Result<IntermediateResult, ParseError> {
    let mut f1 = f_expr(token_list)?;
    let mut index = f1.tokens_read;

    while index < token_list.len() {
        match token_list[index] {
            Token::Multiply => {
                let f2 = f_expr(&token_list[index + 1..])?;
                f1.value *= f2.value;
                f1.tokens_read += f2.tokens_read + 1;
            }
            Token::Divide => {
                let f2 = f_expr(&token_list[index + 1..])?;
                if f2.value == 0.0 {
                    return Err(ParseError::OtherError("Divide by zero error".to_owned()));
                } else {
                    f1.value /= f2.value;
                    f1.tokens_read += f2.tokens_read + 1;
                }
            }
            Token::Number(ref n) => return Err(ParseError::UnexpectedToken(n.clone(), "operator")),
            _ => break,
        }
        index = f1.tokens_read;
    }
    Ok(f1)
}

// Exponentiation
pub fn f_expr(token_list: &[Token]) -> Result<IntermediateResult, ParseError> {
    let mut fn1 = i_expr(token_list)?;
    let mut index = fn1.tokens_read;
    let token_len = token_list.len();
    while index < token_len {
        match token_list[index] {
            Token::Exponent => {
                let f = f_expr(&token_list[index + 1..])?;
                fn1.value = fn1.value.powf(f.value);
                fn1.tokens_read += f.tokens_read + 1;
            }
            Token::Number(ref n) => return Err(ParseError::UnexpectedToken(n.clone(), "operator")),
            _ => break,
        }
        index = fn1.tokens_read;
    }
    Ok(fn1)
}

// Functions and variables(TODO)
pub fn i_expr(token_list: &[Token]) -> Result<IntermediateResult, ParseError> {
    if token_list.is_empty() {
        return Err(ParseError::UnexpectedEndOfInput);
    }
    match token_list[0] {
        Token::Identificator(ref ident) => {
            match token_list.get(1) {
                Some(Token::OpenParen) => {}
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        token_list[0].to_string(),
                        "parenthesis",
                    ));
                }
            }
            let mut i = 2;
            let mut close_parent = false;
            let mut args = Vec::new();
            while i < token_list.len() {
                if let Token::CloseParen = token_list[i] {
                    close_parent = true;
                    i += 1;
                    break;
                }
                if i != 2 {
                    if let Token::Comma = token_list[i] {
                        i += 1;
                    } else {
                        return Err(ParseError::UnexpectedToken(
                            token_list[i].to_string(),
                            "comma",
                        ));
                    }
                }
                let expr = e_expr(&token_list[i..])?;
                i += expr.tokens_read;
                args.push(expr.value);
            }
            if !close_parent {
                return Err(ParseError::OtherError(
                    "no matching close parenthesis found.".to_owned(),
                ));
            }
            macro_rules! functions_processor {
                ($($ident: expr ; $args_count: expr => $proc: expr),*) => ({
                    match &ident[..] {
                        $(
                            $ident => {
                                if args.len() != $args_count {
                                    return Err(ParseError::UnexpectedNumberOfArgs($args_count, args.len()));
                                }
                                $proc(args)
                            }
                        )*
                        f => {
                            return Err(ParseError::OtherError(format!("the function \"{}\" is unsupported.", f)));
                        }
                    }
                });
            }
            type A = Vec<f64>;
            let result = functions_processor!(
                    "abs";1 => |n:A| n[0].abs(),
                    "acos";1 => |n:A| n[0].acos(),
                    "acosh";1 => |n:A| n[0].acosh(),
                    "asin";1 => |n:A| n[0].asin(),
                    "asinh";1 => |n:A| n[0].asinh(),
                    "atan";1 => |n:A| n[0].atan(),
                    "atan2";2 => |n:A| f64::atan2(n[0], n[1]),
                    "atanh";1 => |n:A| n[0].atanh(),
                    "cbrt";1 => |n:A| n[0].cbrt(),
                    "ceil";1 => |n:A| n[0].ceil(),
                    "clamp";3 => |n:A| n[0].max(n[1]).min(n[2]),
                    "copysign";2 => |n:A| f64::copysign(n[0], n[1]),
                    "cos";1 => |n:A| n[0].cos(),
                    "cosh";1 => |n:A| n[0].cosh(),
                    "floor";1 => |n:A| n[0].floor(),
                    "fract";1 => |n:A| n[0].fract(),
                    "hypot";2 => |n:A| f64::hypot(n[0], n[1]),
                    "ln";1 => |n:A| n[0].ln(),
                    "ln_1p";1 => |n:A| n[0].ln_1p(),
                    "log";2 => |n:A| f64::log(n[0], n[1]),
                    "log10";1 => |n:A| n[0].log10(),
                    "log2";1 => |n:A| n[0].log2(),
                    "max";2 => |n:A| f64::max(n[0], n[1]),
                    "min";2 => |n:A| f64::min(n[0], n[1]),
                    "mul_add";3 => |n:A| f64::mul_add(n[0], n[1], n[2]),
                    "recip";1 => |n:A| n[0].recip(),
                    "round";1 => |n:A| n[0].round(),
                    "signum";1 => |n:A| n[0].signum(),
                    "sin";1 => |n:A| n[0].sin(),
                    "sinh";1 => |n:A| n[0].sinh(),
                    "sqrt";1 => |n:A| n[0].sqrt(),
                    "tan";1 => |n:A| n[0].tan(),
                    "tanh";1 => |n:A| n[0].tanh(),
                    "trunc";1 => |n:A| n[0].trunc());
            Ok(IntermediateResult::new(result, i))
        }
        _ => g_expr(token_list),
    }
}

// Numbers and parenthesized expressions
pub fn g_expr(token_list: &[Token]) -> Result<IntermediateResult, ParseError> {
    if !token_list.is_empty() {
        match token_list[0] {
            Token::Number(ref n) => n
                .parse::<f64>()
                .map_err(|_| ParseError::InvalidNumber(n.clone()))
                .map(|num| IntermediateResult::new(num, 1)),
            Token::Minus => {
                if token_list.len() > 1 {
                    if let Token::Number(ref n) = token_list[1] {
                        n.parse::<f64>()
                            .map_err(|_| ParseError::InvalidNumber(n.clone()))
                            .map(|num| IntermediateResult::new(-1.0 * num, 2))
                    } else {
                        Err(ParseError::UnexpectedToken(
                            token_list[1].to_string(),
                            "number",
                        ))
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
                                Token::CloseParen => {
                                    Ok(IntermediateResult::new(ir.value, close_paren + 1))
                                }
                                _ => Err(ParseError::UnexpectedToken(
                                    token_list[close_paren].to_string(),
                                    ")",
                                )),
                            }
                        } else {
                            Err(ParseError::OtherError(
                                "no matching close parenthesis found.".to_owned(),
                            ))
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            _ => Err(ParseError::UnexpectedToken(
                token_list[0].to_string(),
                "number",
            )),
        }
    } else {
        Err(ParseError::UnexpectedEndOfInput)
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<String, ParseError> {
    e_expr(&tokens).map(|answer| answer.value.to_string())
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
        assert_eq!(
            tokenize("12+(2+(3+5))+4+(((6)))").and_then(parse).unwrap(),
            "32"
        );
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

    #[test]
    fn functions() {
        assert_eq!(tokenize("signum(-42)").and_then(parse).unwrap(), "-1");
        assert_eq!(
            tokenize("copysign(-123, 3)").and_then(parse).unwrap(),
            "123"
        );
        assert_eq!(
            tokenize("min(5, max(3, 9))+trunc(9.1)")
                .and_then(parse)
                .unwrap(),
            "14"
        );
    }
}

fn eval(input: &str) -> String {
    match tokenize(input).and_then(parse) {
        Ok(s) => s,
        Err(e) => match e {
            ParseError::InvalidNumber(s) => ["Error: Invalid number: ", s.as_str()].concat(),
            ParseError::UnrecognizedToken(s) => {
                ["Error: Unrecognized token: ", s.as_str()].concat()
            }
            ParseError::UnexpectedToken(found, expected) => [
                "Error: Unexpected token: expected [",
                expected,
                "] but found '",
                found.as_str(),
                "'",
            ]
            .concat(),
            ParseError::UnexpectedEndOfInput => "Error: Unexpected end of input.".to_owned(),
            ParseError::UnexpectedNumberOfArgs(expected, passed) => {
                format!("Error: Expected {} arguments, found {}", expected, passed)
            }
            ParseError::OtherError(s) => s,
        },
    }
}

fn main() {
    let args = args();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    if args.len() > 1 {
        let input: Vec<String> = args.skip(1).collect();
        stdout
            .writeln(eval(&input.join("")).as_bytes())
            .try(&mut stderr);
    } else {
        loop {
            print!("[]> ");
            stdout.flush().try(&mut stderr);
            let mut input = String::new();
            io::stdin().read_line(&mut input).try(&mut stderr);
            if input.is_empty() {
                break;
            } else {
                match input.trim() {
                    "" => (),
                    "exit" => break,
                    s => {
                        stdout.writeln(eval(s).as_bytes()).try(&mut stderr);
                        stdout.flush().try(&mut stderr);
                    }
                }
            }
        }
    }
}
