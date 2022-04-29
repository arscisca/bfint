use std::error::Error;
use std::io::{BufRead, BufReader, Read};
use std::fmt::Formatter;


#[derive(Clone, PartialEq, Eq)]
pub struct Token {
    kind: TokenKind,
    row: usize,
    col: usize,
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Plus,
    Minus,
    LeftBrace,
    RightBrace,
    Dot,
    Comma,
    LeftBracket,
    RightBracket,
}


pub struct Tokenizer<R: Read> {
    reader: BufReader<R>,
    chars: Vec<char>,
    current_line_n: usize,
    current_char_n: usize,
}


/* Token **************************************************************************************************************/
impl Token {
    pub fn from_char(c: char, row: usize, col: usize) -> Result<Token, Box<dyn Error>> {
        Ok(Token {kind: TokenKind::from_char(c)?, row, col})
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }
}


impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}({}:{})", self.kind, self.row, self.col)
    }
}


/* Tokenizer **********************************************************************************************************/
impl<R: Read> Tokenizer<R> {
    pub fn read(source: R) -> Tokenizer<R> {
        let reader = BufReader::new(source);
        Tokenizer {
            reader,
            chars: Vec::new(),
            current_line_n: 0,
            current_char_n: 0,
        }
    }

    fn read_next_line(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => {
                // No more lines: iteration ends
                Ok(false)
            }
            Ok(..) => {
                // Read n characters: update chars iterator and read next character
                self.chars = line.chars().collect();
                self.current_char_n = 0;
                self.current_line_n += 1;
                Ok(true)
            },
            Err(e) => {
                Err(e.into())
            }
        }
    }
}


impl<R: Read> Iterator for Tokenizer<R> {
    type Item = Result<Token, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let char_n = self.current_char_n;
        self.current_char_n += 1;
        if char_n < self.chars.len() {
            let c = self.chars[char_n];
            // Ignore whitespace
            if c.is_whitespace() {
                return self.next();
            }
            // Ignore comments
            if c == '#' {
                return match self.read_next_line() {
                    Ok(true) => self.next(),
                    Ok(false) => None,
                    Err(e) => Some(Err(e.into())),
                }
            }
            // Generate token
            Some(Token::from_char(c, self.current_line_n, self.current_char_n))
        } else {
            // End of line, try to read next
            match self.read_next_line() {
                Ok(true) => self.next(),
                Ok(false) => None,
                Err(e) => Some(Err(e.into())),
            }
        }
    }
}

/* TokenKind **********************************************************************************************************/
impl TokenKind {
    pub fn from_char(c: char) -> Result<TokenKind, Box<dyn Error>> {
        match c {
            '+' => Ok(TokenKind::Plus),
            '-' => Ok(TokenKind::Minus),
            '<' => Ok(TokenKind::LeftBrace),
            '>' => Ok(TokenKind::RightBrace),
            '.' => Ok(TokenKind::Dot),
            ',' => Ok(TokenKind::Comma),
            '[' => Ok(TokenKind::LeftBracket),
            ']' => Ok(TokenKind::RightBracket),
            _ => Err(format!("Invalid character: '{}'", c).into())
        }
    }

    pub fn to_char(&self) -> char {
        match *self {
            TokenKind::Plus => '+',
            TokenKind::Minus => '-',
            TokenKind::LeftBrace => '<',
            TokenKind::RightBrace => '>',
            TokenKind::Dot => '.',
            TokenKind::Comma => ',',
            TokenKind::LeftBracket => '[',
            TokenKind::RightBracket => ']',
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    mod tokenizer {
        use std::fs::File;
        use super::*;

        #[test]
        fn read_from_string() {
            let data = String::from("+-[]<>");
            for token in Tokenizer::read(data.as_bytes()) {
                assert!(token.is_ok(), "Could not read token");
            }
        }

        #[test]
        fn read_from_file() {
            let f = File::open("test/helloworld.bf").expect("Could not open file");
            for token in Tokenizer::read(f) {
                assert!(token.is_ok(), "Could not read token")
            }
        }

        #[test]
        fn token_conversion() {
            use TokenKind::*;
            let chars = String::from("+-[]<>.,");
            let exp_tokens = [
                Token {row: 1, col: 1, kind: Plus},
                Token {row: 1, col: 2, kind: Minus},
                Token {row: 1, col: 3, kind: LeftBracket},
                Token {row: 1, col: 4, kind: RightBracket},
                Token {row: 1, col: 5, kind: LeftBrace},
                Token {row: 1, col: 6, kind: RightBrace},
                Token {row: 1, col: 7, kind: Dot},
                Token {row: 1, col: 8, kind: Comma},
            ];
            assert_eq!(chars.len(), exp_tokens.len(), "Ill formed test: arrays don't match!");
            // Iterate tokens
            for (i, token) in Tokenizer::read(chars.as_bytes()).enumerate() {
                let token = token.expect("Could not convert token");
                assert_eq!(token, exp_tokens[i]);
            }
        }
    }
}
