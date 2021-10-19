#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    IntegerConstant(i64),
    FloatConstant(f64),
}

#[derive(Debug, PartialEq)]
pub enum ReaderError {
    EoF,
    ExpectedADigit(char),
    UnexpectedCharacter(char),
}

pub struct Reader {
    input: String,
    position: usize,
}

impl Reader {
    pub fn new(input: String) -> Self {
        Self { input, position: 0 }
    }

    fn peek(&self, amount: usize) -> Result<char, ReaderError> {
        let position = self.position + amount;
        self.input.chars().nth(position).ok_or(ReaderError::EoF)
    }

    fn current(&self) -> Result<char, ReaderError> {
        self.peek(0)
    }

    fn next(&mut self) -> Result<char, ReaderError> {
        self.position += 1;
        self.current()
    }

    fn next_or_eof(&mut self) -> Result<bool, ReaderError> {
        match self.next() {
            Err(ReaderError::EoF) => Ok(true),
            Err(err) => Err(err),
            Ok(_) => Ok(false),
        }
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    fn is_separator(&self, input: &char) -> bool {
        [' ', '\t', '\n', '\r', '\t'].contains(input)
    }

    fn read_number(&mut self) -> Result<Token, ReaderError> {
        let sign = if self.current()? == '-' {
            self.next()?;
            -1
        } else {
            1
        };

        let mut whole = 0;

        let mut fractional: Option<i64> = None;
        let mut fractional_multiplier = 1;

        while !self.is_separator(&self.current()?) && self.current()? != '.' {
            let digit = self.current()? as i64 - '0' as i64;

            if digit < 0 || digit > 9 {
                return Err(ReaderError::ExpectedADigit(self.current()?));
            }

            whole *= 10;
            whole += digit;

            if self.next_or_eof()? {
                return Ok(Token::IntegerConstant(sign * whole));
            }
        }

        if self.current()? == '.' {
            self.next()?;

            fractional = Some(0);

            while !self.is_separator(&self.current()?) {
                let digit = self.current()? as i64 - '0' as i64;

                if digit < 0 || digit > 9 {
                    return Err(ReaderError::ExpectedADigit(self.current()?));
                }

                fractional = Some(fractional.unwrap() * 10 + digit);
                fractional_multiplier *= 10;

                if self.next_or_eof()? {
                    break;
                }
            }
        }

        if let Some(fractional) = fractional {
            let fractional = fractional as f64 / fractional_multiplier as f64;

            Ok(Token::FloatConstant(
                sign as f64 * (whole as f64 + fractional),
            ))
        } else {
            Ok(Token::IntegerConstant(sign * whole))
        }
    }

    fn read_identifier(&mut self) -> Result<Token, ReaderError> {
        let mut identifier = String::new();

        loop {
            let mut is_legal = false;

            let legal_ranges = [
                ('a'..='z'),
                ('A'..='Z'),
                ('0'..='9'),
                ('!'..='!'),
                ('#'..='/'),
                (':'..=':'),
                ('<'..='@'),
                ('\\'..='\\'),
                ('^'..='`'),
                ('|'..='|'),
                ('~'..='~'),
            ];

            for range in legal_ranges {
                if range.contains(&self.current()?) {
                    is_legal = true;
                    break;
                }
            }

            if !is_legal {
                return Err(ReaderError::UnexpectedCharacter(self.current()?));
            }

            identifier.push(self.current()?);

            if self.next_or_eof()? || self.is_separator(&self.current()?) {
                return Ok(Token::Identifier(identifier));
            }
        }
    }

    fn skip_separators(&mut self) -> Result<(), ReaderError> {
        while self.is_separator(&self.current()?) {
            if self.next_or_eof()? {
                break;
            }
        }

        Ok(())
    }

    pub fn tokenise(&mut self) -> Result<Vec<Token>, ReaderError> {
        let mut tokens = Vec::new();

        while !self.is_eof() {
            self.skip_separators()?;

            if ('0'..='9').contains(&self.current()?)
                || self.current()? == '-' && ('0'..='9').contains(&self.peek(1)?)
            {
                tokens.push(self.read_number()?);
            } else {
                tokens.push(self.read_identifier()?);
            }

            if !self.is_eof() {
                self.skip_separators()?;
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader() {
        let tokens = Reader::new("+ + 0.5 -1.0 -1".into()).tokenise().unwrap();

        assert_eq!(tokens.len(), 5);

        assert_eq!(tokens[0], Token::Identifier("+".into()));
        assert_eq!(tokens[1], Token::Identifier("+".into()));

        assert_eq!(tokens[4], Token::IntegerConstant(-1));

        match tokens[2] {
            Token::FloatConstant(value) => assert!((value - 0.5).abs() <= 1e-3),
            _ => unreachable!(),
        }

        match tokens[3] {
            Token::FloatConstant(value) => assert!((value + 1.0).abs() <= 1e-3),
            _ => unreachable!(),
        }

        let tokens = Reader::new("set a noop set b a".into()).tokenise().unwrap();

        assert_eq!(tokens.len(), 6);

        for (index, expected_value) in ["set", "a", "noop", "set", "b", "a"].iter().enumerate() {
            assert_eq!(tokens[index], Token::Identifier(expected_value.to_string()));
        }
    }
}
