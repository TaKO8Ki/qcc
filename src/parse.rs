use crate::{Node, NodeKind, Token, TokenKind, Tokens};

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
}

impl Tokens {
    pub fn new(tokens: Vec<Token>) -> Self {
        Tokens { tokens, index: 0 }
    }

    fn next(&mut self) -> Option<&Token> {
        self.index += 1;
        self.tokens.get(self.index - 1)
    }

    fn token(&self) -> &Token {
        &self.tokens[self.index]
    }

    pub fn expr(&mut self) -> Node {
        self.equality()
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

        Node::new_node_num(self.expect_number().unwrap())
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

    fn expect(&mut self, op: &str) {
        let token = self.token();
        if token.kind != TokenKind::Reserved || token.str.to_string() != op {
            panic!("expected: {}, actual: {}", op, token.str);
        }
        self.next();
    }

    fn consume(&mut self, op: &str) -> bool {
        let token = self.token();
        if token.kind != TokenKind::Reserved || token.str.to_string() != op {
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
