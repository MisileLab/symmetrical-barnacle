use crate::ast::*;
use crate::lexer::{Lexer, Token};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl ParseError {
    fn new(msg: String) -> Self {
        ParseError { message: msg }
    }
}

type ParseResult<T> = Result<T, ParseError>;

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &Token::Eof
        }
    }

    fn peek(&self, offset: usize) -> &Token {
        let pos = self.pos + offset;
        if pos < self.tokens.len() {
            &self.tokens[pos]
        } else {
            &Token::Eof
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: Token) -> ParseResult<()> {
        if self.current() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(format!(
                "Expected {:?}, found {:?}",
                expected,
                self.current()
            )))
        }
    }

    pub fn parse_module(&mut self) -> ParseResult<Module> {
        let mut items = Vec::new();

        while self.current() != &Token::Eof {
            items.push(self.parse_item()?);
        }

        Ok(Module { items })
    }

    fn parse_item(&mut self) -> ParseResult<Item> {
        match self.current() {
            Token::Data => {
                self.advance();
                Ok(Item::DataDef(self.parse_data_def()?))
            }
            Token::Ident(_) => Ok(Item::FunctionDef(self.parse_function_def()?)),
            _ => Err(ParseError::new(format!(
                "Expected item, found {:?}",
                self.current()
            ))),
        }
    }

    fn parse_data_def(&mut self) -> ParseResult<DataDef> {
        let name = match self.current() {
            Token::Ident(s) => {
                let name = s.clone();
                self.advance();
                name
            }
            _ => {
                return Err(ParseError::new(format!(
                    "Expected data type name, found {:?}",
                    self.current()
                )))
            }
        };

        let mut type_params = Vec::new();
        while let Token::Ident(param) = self.current() {
            if param.chars().next().unwrap().is_lowercase() {
                type_params.push(param.clone());
                self.advance();
            } else {
                break;
            }
        }

        self.expect(Token::Eq)?;

        let mut constructors = Vec::new();
        loop {
            if self.current() == &Token::Pipe {
                self.advance();
            }

            let ctor_name = match self.current() {
                Token::Ident(s) => {
                    let name = s.clone();
                    self.advance();
                    name
                }
                _ => break,
            };

            let mut fields = Vec::new();
            while !matches!(
                self.current(),
                Token::Pipe | Token::Eof | Token::Ident(_)
            ) || (matches!(self.current(), Token::Ident(s) if s.chars().next().unwrap().is_lowercase()))
            {
                fields.push(self.parse_type()?);
            }

            constructors.push(Constructor {
                name: ctor_name,
                fields,
            });

            if self.current() != &Token::Pipe {
                break;
            }
        }

        Ok(DataDef {
            name,
            type_params,
            constructors,
        })
    }

    fn parse_function_def(&mut self) -> ParseResult<FunctionDef> {
        let name = match self.current() {
            Token::Ident(s) => {
                let name = s.clone();
                self.advance();
                name
            }
            _ => {
                return Err(ParseError::new(format!(
                    "Expected function name, found {:?}",
                    self.current()
                )))
            }
        };

        self.expect(Token::Colon)?;

        let type_sig = self.parse_type_sig()?;

        let mut params = Vec::new();
        if let Token::Ident(s) = self.current() {
            if s == &name {
                self.advance();
            }
        }

        while let Token::Ident(param) = self.current() {
            params.push(param.clone());
            self.advance();
            if self.current() == &Token::Eq {
                break;
            }
        }

        self.expect(Token::Eq)?;

        let body = self.parse_expr()?;

        Ok(FunctionDef {
            name,
            type_sig,
            params,
            body,
        })
    }

    fn parse_type_sig(&mut self) -> ParseResult<TypeSig> {
        let mut param_types = Vec::new();

        loop {
            let ty = self.parse_type()?;

            if self.current() == &Token::Arrow {
                self.advance();
                param_types.push(ty);
            } else {
                let effects = if self.current() == &Token::Bang {
                    self.parse_effects()?
                } else {
                    EffectSet::default()
                };

                return Ok(TypeSig {
                    param_types,
                    return_type: Box::new(ty),
                    effects,
                });
            }
        }
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        match self.current() {
            Token::Ident(s) if s == "i32" => {
                self.advance();
                Ok(Type::I32)
            }
            Token::Ident(s) if s == "Bool" => {
                self.advance();
                Ok(Type::Bool)
            }
            Token::Ident(s) if s == "Unit" => {
                self.advance();
                Ok(Type::Unit)
            }
            Token::Ident(s) if s == "String" => {
                self.advance();
                Ok(Type::String)
            }
            Token::Ident(s) if s == "Arena" => {
                self.advance();
                Ok(Type::Arena)
            }
            Token::Ident(s) if s == "Space" => {
                self.advance();
                Ok(Type::Space(SpaceKind::Cpu))
            }
            Token::Ident(s) if s == "Array" => {
                self.advance();
                let elem_type = self.parse_type()?;

                let space = if self.current() == &Token::LParen {
                    self.advance();
                    let space = match self.current() {
                        Token::Cpu => {
                            self.advance();
                            SpaceKind::Cpu
                        }
                        Token::Gpu => {
                            self.advance();
                            SpaceKind::Gpu
                        }
                        _ => SpaceKind::Cpu,
                    };
                    self.expect(Token::Comma)?;
                    let arena = match self.current() {
                        Token::Ident(s) => {
                            let name = s.clone();
                            self.advance();
                            name
                        }
                        _ => "default".to_string(),
                    };
                    self.expect(Token::RParen)?;
                    space
                } else {
                    SpaceKind::Cpu
                };

                Ok(Type::Array(
                    Box::new(elem_type),
                    space,
                    "default".to_string(),
                ))
            }
            Token::Ident(s) if s == "Task" => {
                self.advance();
                let inner = self.parse_type()?;
                Ok(Type::Task(Box::new(inner)))
            }
            Token::Ident(s) if s.chars().next().unwrap().is_lowercase() => {
                let name = s.clone();
                self.advance();
                Ok(Type::Var(name))
            }
            Token::Ident(s) => {
                let name = s.clone();
                self.advance();
                let mut type_args = Vec::new();
                while !matches!(
                    self.current(),
                    Token::Arrow | Token::Bang | Token::Comma | Token::RParen | Token::Eof
                ) {
                    type_args.push(self.parse_type()?);
                }
                Ok(Type::Custom(name, type_args))
            }
            Token::LParen => {
                self.advance();
                let mut types = vec![self.parse_type()?];
                while self.current() == &Token::Comma {
                    self.advance();
                    types.push(self.parse_type()?);
                }
                self.expect(Token::RParen)?;

                if self.current() == &Token::Arrow {
                    self.advance();
                    let return_type = self.parse_type()?;
                    let effects = if self.current() == &Token::Bang {
                        self.parse_effects()?
                    } else {
                        EffectSet::default()
                    };
                    Ok(Type::Function(types, Box::new(return_type), effects))
                } else if types.len() == 1 {
                    Ok(types.into_iter().next().unwrap())
                } else {
                    Err(ParseError::new("Invalid type".to_string()))
                }
            }
            _ => Err(ParseError::new(format!(
                "Expected type, found {:?}",
                self.current()
            ))),
        }
    }

    fn parse_effects(&mut self) -> ParseResult<EffectSet> {
        self.expect(Token::Bang)?;
        self.expect(Token::LBrace)?;

        let mut purity = Purity::Pure;
        let mut execution = Execution::Cpu;
        let mut allocation = Allocation::None;
        let mut concurrency = Concurrency::Single;
        let mut debug = false;

        loop {
            match self.current() {
                Token::Pure => {
                    purity = Purity::Pure;
                    self.advance();
                }
                Token::IO => {
                    purity = Purity::IO;
                    self.advance();
                }
                Token::State => {
                    purity = Purity::State;
                    self.advance();
                }
                Token::Debug => {
                    debug = true;
                    self.advance();
                }
                Token::Cpu => {
                    execution = Execution::Cpu;
                    self.advance();
                }
                Token::Gpu => {
                    execution = Execution::Gpu;
                    self.advance();
                }
                Token::Alloc => {
                    self.advance();
                    match self.current() {
                        Token::None_ => {
                            allocation = Allocation::None;
                            self.advance();
                        }
                        Token::Arena => {
                            allocation = Allocation::Arena;
                            self.advance();
                        }
                        Token::Heap => {
                            allocation = Allocation::Heap;
                            self.advance();
                        }
                        _ => {
                            return Err(ParseError::new(format!(
                                "Expected allocation type, found {:?}",
                                self.current()
                            )))
                        }
                    }
                }
                Token::Concurrent => {
                    concurrency = Concurrency::Concurrent;
                    self.advance();
                }
                Token::Single => {
                    concurrency = Concurrency::Single;
                    self.advance();
                }
                Token::Comma => {
                    self.advance();
                }
                Token::RBrace => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(ParseError::new(format!(
                        "Expected effect or }}, found {:?}",
                        self.current()
                    )))
                }
            }
        }

        Ok(EffectSet {
            purity,
            execution,
            allocation,
            concurrency,
            debug,
        })
    }

    fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_logical_and()?;

        while self.current() == &Token::Or {
            self.advance();
            let right = self.parse_logical_and()?;
            left = Expr::BinOp(BinOp::Or, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_logical_and(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_comparison()?;

        while self.current() == &Token::And {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinOp(BinOp::And, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_additive()?;

        loop {
            let op = match self.current() {
                Token::EqEq => BinOp::Eq,
                Token::Ne => BinOp::Ne,
                Token::Lt => BinOp::Lt,
                Token::Le => BinOp::Le,
                Token::Gt => BinOp::Gt,
                Token::Ge => BinOp::Ge,
                _ => break,
            };

            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinOp(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.current() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };

            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinOp(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.current() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };

            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> ParseResult<Expr> {
        match self.current() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnOp(UnOp::Neg, Box::new(expr)))
            }
            Token::Bang => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnOp(UnOp::Not, Box::new(expr)))
            }
            _ => self.parse_application(),
        }
    }

    fn parse_application(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_primary()?;

        // Parse function arguments - keep consuming primary expressions as arguments
        loop {
            // Stop at operators and keywords
            if matches!(
                self.current(),
                Token::In
                    | Token::Then
                    | Token::Else
                    | Token::Comma
                    | Token::RParen
                    | Token::RBrace
                    | Token::Pipe
                    | Token::Arrow
                    | Token::Eof
                    | Token::Plus
                    | Token::Minus
                    | Token::Star
                    | Token::Slash
                    | Token::Percent
                    | Token::EqEq
                    | Token::Ne
                    | Token::Lt
                    | Token::Le
                    | Token::Gt
                    | Token::Ge
                    | Token::And
                    | Token::Or
            ) {
                break;
            }

            // Stop if we see an identifier followed by : or = (start of function def)
            if matches!(self.current(), Token::Ident(_))
                && matches!(self.peek(1), Token::Colon | Token::Eq)
            {
                break;
            }

            // Parse the next argument
            let arg = self.parse_primary()?;

            // Convert to function call
            match expr {
                Expr::Call(func, mut args) => {
                    args.push(arg);
                    expr = Expr::Call(func, args);
                }
                _ => {
                    // First argument - convert Var to Call or create new Call
                    if let Expr::Var(name) = expr {
                        expr = Expr::Call(name, vec![arg]);
                    } else {
                        // For other expressions, we can't apply them
                        break;
                    }
                }
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> ParseResult<Expr> {
        match self.current() {
            Token::Int(n) => {
                let val = *n;
                self.advance();
                Ok(Expr::Literal(Literal::Int(val)))
            }
            Token::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            Token::Unit => {
                self.advance();
                Ok(Expr::Literal(Literal::Unit))
            }
            Token::String(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(val)))
            }
            Token::Let => {
                self.advance();
                let var = match self.current() {
                    Token::Ident(s) => {
                        let name = s.clone();
                        self.advance();
                        name
                    }
                    _ => {
                        return Err(ParseError::new(format!(
                            "Expected variable name, found {:?}",
                            self.current()
                        )))
                    }
                };
                self.expect(Token::Eq)?;
                let value = self.parse_expr()?;
                self.expect(Token::In)?;
                let body = self.parse_expr()?;
                Ok(Expr::Let(var, Box::new(value), Box::new(body)))
            }
            Token::If => {
                self.advance();
                let cond = self.parse_expr()?;
                self.expect(Token::Then)?;
                let then_expr = self.parse_expr()?;
                self.expect(Token::Else)?;
                let else_expr = self.parse_expr()?;
                Ok(Expr::If(
                    Box::new(cond),
                    Box::new(then_expr),
                    Box::new(else_expr),
                ))
            }
            Token::Match => {
                self.advance();
                let scrutinee = self.parse_expr()?;
                self.expect(Token::With)?;

                let mut arms = Vec::new();
                while self.current() == &Token::Pipe {
                    self.advance();
                    let pattern = self.parse_pattern()?;
                    self.expect(Token::Arrow)?;
                    let body = self.parse_expr()?;
                    arms.push(MatchArm { pattern, body });
                }

                Ok(Expr::Match(Box::new(scrutinee), arms))
            }
            Token::Unsafe => {
                self.advance();
                self.expect(Token::LBrace)?;
                let expr = self.parse_expr()?;
                self.expect(Token::RBrace)?;
                Ok(Expr::Unsafe(Box::new(expr)))
            }
            Token::ParFor => {
                self.advance();
                self.expect(Token::LParen)?;
                let start = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let end = self.parse_expr()?;
                self.expect(Token::RParen)?;
                let body = self.parse_expr()?;
                Ok(Expr::ParFor(Box::new(start), Box::new(end), Box::new(body)))
            }
            Token::ParMap => {
                self.advance();
                let func = self.parse_expr()?;
                let array = self.parse_expr()?;
                Ok(Expr::ParMap(Box::new(func), Box::new(array)))
            }
            Token::ParMapInplace => {
                self.advance();
                let func = self.parse_expr()?;
                let src = self.parse_expr()?;
                let dst = self.parse_expr()?;
                Ok(Expr::ParMapInplace(
                    Box::new(func),
                    Box::new(src),
                    Box::new(dst),
                ))
            }
            Token::Async => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::Async(Box::new(expr)))
            }
            Token::Await => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::Await(Box::new(expr)))
            }
            Token::NewArray => {
                self.advance();
                let space = match self.current() {
                    Token::Cpu => {
                        self.advance();
                        SpaceKind::Cpu
                    }
                    Token::Gpu => {
                        self.advance();
                        SpaceKind::Gpu
                    }
                    _ => {
                        return Err(ParseError::new(format!(
                            "Expected Cpu or Gpu, found {:?}",
                            self.current()
                        )))
                    }
                };
                let arena = match self.current() {
                    Token::Ident(s) => {
                        let name = s.clone();
                        self.advance();
                        name
                    }
                    _ => "default".to_string(),
                };
                let size = self.parse_expr()?;
                Ok(Expr::NewArray(space, arena, Box::new(size)))
            }
            Token::ArrayGet => {
                self.advance();
                let array = self.parse_expr()?;
                let index = self.parse_expr()?;
                Ok(Expr::ArrayGet(Box::new(array), Box::new(index)))
            }
            Token::ArraySet => {
                self.advance();
                let array = self.parse_expr()?;
                let index = self.parse_expr()?;
                let value = self.parse_expr()?;
                Ok(Expr::ArraySet(
                    Box::new(array),
                    Box::new(index),
                    Box::new(value),
                ))
            }
            Token::ArrayLen => {
                self.advance();
                let array = self.parse_expr()?;
                Ok(Expr::ArrayLen(Box::new(array)))
            }
            Token::GpuKernel => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::GpuKernel(Box::new(expr)))
            }
            Token::CpuToGpu => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::CpuToGpu(Box::new(expr)))
            }
            Token::GpuToCpu => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::GpuToCpu(Box::new(expr)))
            }
            Token::Log => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::Log(Box::new(expr)))
            }
            Token::Assert => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::Assert(Box::new(expr)))
            }
            Token::Print => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::Print(Box::new(expr)))
            }
            Token::RawThreadSpawn => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::RawThreadSpawn(Box::new(expr)))
            }
            Token::AtomicLoad => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::AtomicLoad(Box::new(expr)))
            }
            Token::AtomicStore => {
                self.advance();
                let addr = self.parse_expr()?;
                let value = self.parse_expr()?;
                Ok(Expr::AtomicStore(Box::new(addr), Box::new(value)))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Var(name))
            }
            _ => Err(ParseError::new(format!(
                "Expected expression, found {:?}",
                self.current()
            ))),
        }
    }

    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        match self.current() {
            Token::Ident(s) if s == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Token::Int(n) => {
                let val = *n;
                self.advance();
                Ok(Pattern::Literal(Literal::Int(val)))
            }
            Token::True => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(false)))
            }
            Token::Unit => {
                self.advance();
                Ok(Pattern::Literal(Literal::Unit))
            }
            Token::String(s) => {
                let val = s.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(val)))
            }
            Token::Ident(s) => {
                let name = s.clone();
                self.advance();

                if name.chars().next().unwrap().is_uppercase() {
                    let mut fields = Vec::new();
                    while !matches!(self.current(), Token::Arrow | Token::Pipe | Token::Eof) {
                        fields.push(self.parse_pattern()?);
                    }
                    Ok(Pattern::Constructor(name, fields))
                } else {
                    Ok(Pattern::Var(name))
                }
            }
            _ => Err(ParseError::new(format!(
                "Expected pattern, found {:?}",
                self.current()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let input = "inc: i32 -> i32\ninc x = x + 1";
        let mut parser = Parser::new(input);
        let module = parser.parse_module().unwrap();
        assert_eq!(module.items.len(), 1);
    }

    #[test]
    fn test_parse_effects() {
        let input = "foo: i32 -> i32 !{pure, cpu, alloc none}\nfoo x = x + 1";
        let mut parser = Parser::new(input);
        let module = parser.parse_module().unwrap();
        assert_eq!(module.items.len(), 1);
    }
}
