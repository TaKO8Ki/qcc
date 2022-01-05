use crate::{LVar, Node, NodeKind, Token, TokenKind, Tokens};
use std::collections::LinkedList;

impl Node {
    fn new(kind: NodeKind, lhs: Node, rhs: Node) -> Self {
        Node {
            kind,
            lhs: Some(Box::new(lhs)),
            rhs: Some(Box::new(rhs)),
        }
    }

    fn new_node_num(val: u16) -> Self {
        Node {
            kind: NodeKind::Num(val),
            lhs: None,
            rhs: None,
        }
    }

    fn new_node_var(offset: usize) -> Self {
        Node {
            kind: NodeKind::LVar(offset),
            lhs: None,
            rhs: None,
        }
    }
}

impl Tokens {
    pub fn new(tokens: Vec<Token>) -> Self {
        Tokens {
            locals: LinkedList::new(),
            tokens,
            index: 0,
            code: Vec::new(),
        }
    }

    fn next(&mut self) -> Option<&Token> {
        self.index += 1;
        self.tokens.get(self.index - 1)
    }

    fn find_lvar(&self) -> Option<&LVar> {
        for lvar in self.locals.iter() {
            if lvar.name.len() == self.token().str.len() && lvar.name == self.token().str {
                return Some(lvar);
            }
        }
        None
    }

    fn token(&self) -> &Token {
        &self.tokens[self.index]
    }

    pub fn expr(&mut self) -> Node {
        self.assign()
    }

    fn assign(&mut self) -> Node {
        let mut node = self.equality();
        if self.consume("=") {
            node = Node::new(NodeKind::Assign, node, self.assign());
        }
        node
    }

    pub fn program(&mut self) {
        loop {
            log::debug!("program token={:?}", self.token());
            if self.token().kind == TokenKind::Eof {
                break;
            }

            let stmt = self.stmt();
            self.code.push(stmt);
        }
    }

    fn stmt(&mut self) -> Node {
        let node = if self.consume("return") {
            Node {
                kind: NodeKind::Return,
                lhs: Some(Box::new(self.expr())),
                rhs: None,
            }
        } else {
            self.expr()
        };
        log::debug!("node: {:?}", node);
        self.expect(';');
        node
    }

    fn add(&mut self) -> Node {
        let mut node = self.mul();

        loop {
            if self.consume("+") {
                node = Node::new(NodeKind::Add, node, self.mul());
            } else if self.consume("-") {
                node = Node::new(NodeKind::Sub, node, self.mul());
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node {
        let mut node = self.unary();

        loop {
            if self.consume("*") {
                node = Node::new(NodeKind::Mul, node, self.unary());
            } else if self.consume("/") {
                node = Node::new(NodeKind::Div, node, self.unary());
            } else {
                return node;
            }
        }
    }

    fn unary(&mut self) -> Node {
        if self.consume("+") {
            return self.primary();
        } else if self.consume("-") {
            return Node::new(NodeKind::Sub, Node::new_node_num(0), self.primary());
        }
        self.primary()
    }

    fn primary(&mut self) -> Node {
        if self.consume("(") {
            let node = self.expr();
            self.expect(")");
            return node;
        }

        if let TokenKind::Ident = self.token().kind {
            let lvar = self.find_lvar();
            let node = match lvar {
                Some(lvar) => Node::new_node_var(lvar.offset),
                None => {
                    let lvar = LVar {
                        name: self.token().str.clone(),
                        offset: self.locals.front().map_or(0, |lvar| lvar.offset) + 8,
                    };
                    let node = Node::new_node_var(lvar.offset);
                    self.locals.push_front(lvar);
                    node
                }
            };

            self.next();
            return node;
        }

        if let TokenKind::Num(val) = self.token().kind {
            let node = Node::new_node_num(val);
            self.next();
            return node;
        }

        panic!("primary: unexpected token {:?}", self.token());
    }

    fn equality(&mut self) -> Node {
        let mut node = self.relational();

        loop {
            if self.consume("==") {
                node = Node::new(NodeKind::Eq, node, self.relational());
            } else if self.consume("!=") {
                node = Node::new(NodeKind::Ne, node, self.relational());
            } else {
                return node;
            }
        }
    }

    fn relational(&mut self) -> Node {
        let mut node = self.add();

        loop {
            if self.consume("<") {
                node = Node::new(NodeKind::Lt, node, self.add());
            } else if self.consume("<=") {
                node = Node::new(NodeKind::Le, node, self.add());
            } else if self.consume(">") {
                node = Node::new(NodeKind::Lt, self.add(), node);
            } else if self.consume(">=") {
                node = Node::new(NodeKind::Le, self.add(), node);
            } else {
                return node;
            }
        }
    }

    fn expect(&mut self, op: impl Into<String>) {
        let token = self.token();
        let op = op.into();
        if token.kind != TokenKind::Reserved || token.str.to_string() != op {
            panic!("expected: {}, actual: {}", op, token.str);
        }
        self.next();
    }

    fn consume(&mut self, op: impl Into<String>) -> bool {
        let token = self.token();
        let op = op.into();
        if (token.kind != TokenKind::Reserved && token.kind != TokenKind::Return)
            || token.str.to_string() != op
        {
            return false;
        }
        self.next();
        true
    }

    fn expect_number(&mut self) -> Result<u16, String> {
        let token = self.token();
        if let TokenKind::Num(val) = token.kind {
            self.next();
            Ok(val)
        } else {
            Err(format!("{} is not number", token.str))
        }
    }
}
