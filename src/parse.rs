use super::*;

pub fn parse_expression(tokens: &Vec<Token>) -> Result<Exp, String> {
    let mut parser = Parser::new(tokens);
    let exp = parser.parse()?;
    if parser.current_token().is_some() {
        return Err("Expected to find end of input".to_string());
    }
    Ok(exp)
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            '0'..='9' | '-' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    tokens.push(Token::RightArrow);
                    chars.next();
                } else {
                    let mut int_str = String::new();
                    int_str.push(ch);

                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_digit() {
                            int_str.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    match int_str.parse::<isize>() {
                        Ok(i) => tokens.push(Token::Int(i)),
                        Err(_) => {
                            return Err(format!(
                                "Invalid integer format: {}",
                                int_str
                            ));
                        }
                    }
                }
            }
            '+' => {
                chars.next();
                if let Some('+') = chars.peek() {
                    tokens.push(Token::Concat);
                    chars.next();
                } else {
                    tokens.push(Token::Plus);
                }
            }
            '<' => {
                tokens.push(Token::LessThan);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LeftParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RightParen);
                chars.next();
            }
            '{' => {
                tokens.push(Token::LeftBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::RightBrace);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            '=' => {
                tokens.push(Token::Equal);
                chars.next();
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        break;
                    }
                    s.push(ch);
                    chars.next();
                }
                if chars.next() != Some('"') {
                    return Err(format!("unterminated string"));
                }
                tokens.push(Token::Str(s));
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                // Parse identifiers and keywords
                let mut ident_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        ident_str.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                // Match keywords or push as identifier
                match ident_str.as_str() {
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "let" => tokens.push(Token::Let),
                    "true" => tokens.push(Token::Bool(true)),
                    "false" => tokens.push(Token::Bool(false)),
                    "fn" => tokens.push(Token::Fn),
                    "int" => tokens.push(Token::IntType),
                    "bool" => tokens.push(Token::BoolType),
                    "str" => tokens.push(Token::StrType),
                    _ => tokens.push(Token::Symbol(ident_str)),
                }
            }
            _ => {
                return Err(format!("Unexpected character: '{}'", ch));
            }
        }
    }

    Ok(tokens)
}

struct Parser<'a> {
    tokens: &'a Vec<Token>,
    position: usize,
}

// grammar:
// expression       -> term [ (+ | ++ | <) term ]*
// term             -> factor [ ( expression ) ]*
// factor           -> ( expression ) | conditional | let1 | lambda | int | bool | str | symbol
// conditional      -> if expression { expression } else { expression }
// let1             -> let symbol = expression { expression }
// lambda           -> fn ( symbol : typeexp ) { expression }
// typeexp          -> num | bool | str | (typeexp -> typeexp)

impl<'a> Parser<'a> {
    fn new(tokens: &'a Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn parse(&mut self) -> Result<Exp, String> {
        self.parse_expression()
    }

    fn parse_expression(&mut self) -> Result<Exp, String> {
        let mut left = self.parse_term()?;

        loop {
            match self.current_token() {
                Some(Token::Plus) => {
                    self.expect_token(&Token::Plus)?;
                    let right = self.parse_term()?;
                    left = Exp::Plus {
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }

                Some(Token::Concat) => {
                    self.expect_token(&Token::Concat)?;
                    let right = self.parse_term()?;
                    left = Exp::Concat {
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }

                Some(Token::LessThan) => {
                    self.expect_token(&Token::LessThan)?;
                    let right = self.parse_term()?;
                    left = Exp::LessThan {
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }

                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Exp, String> {
        let mut term = self.parse_factor()?;

        while let Some(&Token::LeftParen) = self.current_token() {
            let fun = Box::new(term);
            self.expect_token(&Token::LeftParen)?;
            let arg = Box::new(self.parse_expression()?);
            self.expect_token(&Token::RightParen)?;
            term = Exp::App { fun, arg };
        }

        Ok(term)
    }

    fn parse_factor(&mut self) -> Result<Exp, String> {
        match self.current_token() {
            Some(Token::LeftParen) => {
                // ( expr )
                self.expect_token(&Token::LeftParen)?;
                let expr = self.parse_expression()?;
                self.expect_token(&Token::RightParen)?;
                Ok(expr)
            }

            Some(Token::If) => self.parse_conditional(),

            Some(Token::Let) => self.parse_let1(),

            Some(Token::Fn) => self.parse_lambda(),

            Some(&Token::Int(n)) => {
                self.advance();

                Ok(Exp::Int(n))
            }

            Some(&Token::Bool(b)) => {
                self.advance();

                Ok(Exp::Bool(b))
            }

            Some(Token::Str(s)) => {
                let ss = s.clone();
                self.advance();

                Ok(Exp::Str(ss))
            }

            Some(Token::Symbol(s)) => {
                let var = Exp::Var(s.clone());
                self.advance();

                Ok(var)
            }

            _ => Err("Expected a factor".to_string()),
        }
    }

    fn parse_conditional(&mut self) -> Result<Exp, String> {
        // if cnd { thn } else { els }
        self.expect_token(&Token::If)?;
        let tst = Box::new(self.parse_expression()?);
        self.expect_token(&Token::LeftBrace)?;
        let thn = Box::new(self.parse_expression()?);
        self.expect_token(&Token::RightBrace)?;
        self.expect_token(&Token::Else)?;
        self.expect_token(&Token::LeftBrace)?;
        let els = Box::new(self.parse_expression()?);
        self.expect_token(&Token::RightBrace)?;
        Ok(Exp::Cnd { tst, thn, els })
    }

    fn parse_let1(&mut self) -> Result<Exp, String> {
        // let symbol = exp { exp }
        self.expect_token(&Token::Let)?;
        let Some(Token::Symbol(s)) = self.current_token() else {
            return Err("Expected an indentifier".to_string());
        };
        let var = s.clone();
        self.advance();
        self.expect_token(&Token::Equal)?;
        let value = Box::new(self.parse_expression()?);
        self.expect_token(&Token::LeftBrace)?;
        let body = Box::new(self.parse_expression()?);
        self.expect_token(&Token::RightBrace)?;
        Ok(Exp::Let1 { var, value, body })
    }

    fn parse_lambda(&mut self) -> Result<Exp, String> {
        // fn ( symbol : typeexp ) { exp }
        self.expect_token(&Token::Fn)?;
        self.expect_token(&Token::LeftParen)?;
        let Some(Token::Symbol(s)) = self.current_token() else {
            return Err("Expected an indentifier".to_string());
        };
        let var = s.clone();
        self.advance();
        self.expect_token(&Token::Colon)?;
        let param_type = self.parse_typeexp()?;
        self.expect_token(&Token::RightParen)?;
        self.expect_token(&Token::LeftBrace)?;
        let body = Box::new(self.parse_expression()?);
        self.expect_token(&Token::RightBrace)?;
        Ok(Exp::Lam { var, var_type: param_type, body })
    }

    fn parse_typeexp(&mut self) -> Result<Type, String> {
        // num | bool | str | (typeexp -> typeexp)
        match self.current_token() {
            Some(Token::IntType) => {
                self.advance();
                Ok(Type::Int)
            }

            Some(Token::BoolType) => {
                self.advance();
                Ok(Type::Bool)
            }

            Some(Token::StrType) => {
                self.advance();
                Ok(Type::Str)
            }

            Some(Token::LeftParen) => {
                self.expect_token(&Token::LeftParen)?;
                let param = Box::new(self.parse_typeexp()?);
                self.expect_token(&Token::RightArrow)?;
                let result = Box::new(self.parse_typeexp()?);
                self.expect_token(&Token::RightParen)?;
                Ok(Type::Fun { param, result })
            }

            _ => Err(format!("Expected a type")),
        }
    }

    fn expect_token(&mut self, expected: &Token) -> Result<(), String> {
        if self.current_token() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected '{:?}' token", expected))
        }
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) {
        self.position += 1;
    }
}
