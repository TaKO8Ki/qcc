use crate::{Function, LVar, Node, NodeKind, Token, TokenKind, Tokens, Type, TypeKind};
use std::collections::LinkedList;

impl Token {
    fn get_ident(&self) -> Option<String> {
        match self.kind {
            TokenKind::Ident => Some(self.str.clone()),
            _ => None,
        }
    }
}

impl Node {
    fn new(kind: NodeKind) -> Self {
        Node {
            kind,
            lhs: None,
            rhs: None,
            body: None,
            ty: None,
        }
    }

    fn new_binary(kind: NodeKind, lhs: Node, rhs: Node) -> Self {
        Node {
            kind,
            lhs: Some(Box::new(lhs)),
            rhs: Some(Box::new(rhs)),
            body: None,
            ty: None,
        }
    }

    fn new_unary(kind: NodeKind, lhs: Node) -> Self {
        Node {
            kind,
            lhs: Some(Box::new(lhs)),
            rhs: None,
            body: None,
            ty: None,
        }
    }

    fn new_node_num(val: u16) -> Self {
        Node {
            kind: NodeKind::Num(val),
            lhs: None,
            rhs: None,
            body: None,
            ty: None,
        }
    }

    fn new_lvar(lvar: LVar, ty: Type) -> Self {
        Node {
            kind: NodeKind::LVar(lvar),
            lhs: None,
            rhs: None,
            body: None,
            ty: Some(ty),
        }
    }

    fn new_block(body: Option<Vec<Node>>) -> Self {
        Node {
            kind: NodeKind::Block,
            lhs: None,
            rhs: None,
            body: body.map(|body| Box::new(body)),
            ty: None,
        }
    }

    fn new_add(lhs: Node, rhs: Node) -> Self {
        let mut lhs = lhs;
        let mut rhs = rhs;

        lhs.add_type();
        rhs.add_type();

        if let Some(lhs_ty) = &lhs.ty {
            let lhs_ty = lhs_ty.clone();
            if let Some(rhs_ty) = &rhs.ty {
                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    return Node::new_binary(NodeKind::Add, lhs, rhs);
                }

                if lhs_ty.is_pointer() && rhs_ty.is_pointer() {
                    panic!("invalid operands")
                }

                if !lhs_ty.is_pointer() && rhs_ty.is_pointer() {
                    let tmp = lhs;
                    lhs = rhs;
                    rhs = tmp;
                }

                return Node::new_binary(
                    NodeKind::Add,
                    lhs,
                    Node::new_binary(
                        NodeKind::Mul,
                        rhs,
                        Self::new_node_num(lhs_ty.base().unwrap().size().unwrap()),
                    ),
                );
            }
        }

        unreachable!("invalid operands")
    }

    fn new_sub(lhs: Node, rhs: Node) -> Self {
        let mut lhs = lhs;
        let mut rhs = rhs;

        lhs.add_type();
        rhs.add_type();

        if let Some(lhs_ty) = &lhs.ty {
            if let Some(rhs_ty) = &rhs.ty {
                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    return Node::new_binary(NodeKind::Sub, lhs, rhs);
                }

                if rhs_ty.is_integer() {
                    if let TypeKind::Ptr { base, .. } = &lhs_ty.kind {
                        let mut rhs = Node::new_binary(
                            NodeKind::Mul,
                            rhs,
                            Self::new_node_num(base.size().unwrap()),
                        );
                        rhs.add_type();
                        let ty = lhs.ty.clone();
                        let mut node = Node::new_binary(NodeKind::Sub, lhs, rhs);
                        node.ty = ty;
                        return node;
                    }
                }

                if rhs_ty.is_pointer() {
                    if let TypeKind::Ptr { base, .. } = &lhs_ty.clone().kind {
                        let mut node = Node::new_binary(NodeKind::Sub, lhs, rhs);
                        node.ty = Some(Type::type_int());
                        return Node::new_binary(
                            NodeKind::Div,
                            node,
                            Self::new_node_num(base.size().unwrap()),
                        );
                    }
                }
            }
        }

        panic!("invalid operands: lhs={:?}, rhs={:?}", lhs, rhs);
    }
}

impl Tokens {
    pub fn new(tokens: Vec<Token>) -> Self {
        Tokens {
            locals: LinkedList::new(),
            tokens,
            index: 0,
            functions: LinkedList::new(),
        }
    }

    fn next(&mut self) -> Option<&Token> {
        self.index += 1;
        self.tokens.get(self.index - 1)
    }

    fn next_token(&self) -> Option<&Token> {
        self.tokens.get(self.index + 1)
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
            node = Node::new_binary(NodeKind::Assign, node, self.assign());
        }
        node
    }

    pub fn program(&mut self) {
        loop {
            log::debug!("program token={:?}", self.token());
            if self.token().kind == TokenKind::Eof {
                break;
            }

            let function = self.function();
            self.functions.push_back(function);
        }
        log::debug!("functions={:?}", self.functions);
    }

    fn add_lvar(&mut self, name: String, ty: Type) -> LVar {
        let lvar = LVar {
            name,
            offset: self.locals.front().map_or(0, |lvar| lvar.offset) + ty.size().unwrap() as usize,
            ty,
        };
        self.locals.push_front(lvar.clone());
        lvar
    }

    fn get_number(&self) -> u16 {
        if let TokenKind::Num(val) = self.token().kind {
            return val;
        }
        unreachable!("expected a number: {:?}", self.token());
    }

    fn declspec(&mut self) -> Type {
        self.expect("int");
        Type::type_int()
    }

    fn func_params(&mut self, ty: Type) -> Type {
        let mut params = Vec::new();

        while !self.consume(')') {
            log::debug!("type_suffix token={:?}", self.token());
            if params.len() > 0 {
                self.expect(",");
            }
            let basety = self.declspec();
            let ty = self.declarator(basety);
            params.push(ty);
        }

        ty.func_type(params)
    }

    fn type_suffix(&mut self, ty: Type) -> Type {
        if self.consume("(") {
            return self.func_params(ty);
        }

        if self.consume('[') {
            let sz = self.get_number();
            self.next();
            self.expect(']');
            return self.type_suffix(ty.clone()).array_of(sz);
        }
        ty
    }

    fn declarator(&mut self, ty: Type) -> Type {
        let mut ty = ty;
        while self.consume('*') {
            ty = ty.pointer_to();
        }

        if !matches!(self.token().kind, TokenKind::Ident) {
            panic!("expected a variable name, got {:?}", self.token());
        }

        let func_name = self.token().clone();
        self.next();
        log::debug!("declarator token={:?}", self.token());
        let mut ty = self.type_suffix(ty);
        ty.name = Some(func_name);
        ty
    }

    fn declaration(&mut self) -> Node {
        let basety = self.declspec();
        let mut body = Vec::new();

        let mut i = 0;
        while !self.consume(';') {
            if i > 0 {
                self.expect(',');
            }
            i += 1;

            let ty = self.declarator(basety.clone());
            let lvar = self.add_lvar(ty.clone().name.unwrap().get_ident().unwrap(), ty.clone());
            let lhs = Node::new_lvar(lvar, ty);

            if !self.consume('=') {
                continue;
            }

            let rhs = self.assign();
            let node = Node::new_binary(NodeKind::Assign, lhs, rhs);
            body.push(Node::new_unary(NodeKind::ExprStmt, node));
        }

        log::debug!("body={:?}", body);
        let node = Node::new_block(Some(body));
        log::debug!("declaration last token={:?}", self.token());
        node
    }

    fn stmt(&mut self) -> Node {
        if self.consume("if") {
            self.expect('(');
            let cond = self.expr();
            self.expect(')');
            let then = self.stmt();
            let mut node = Node::new(NodeKind::If {
                cond: Box::new(cond),
                then: Box::new(then),
                els: None,
            });
            if self.consume("else") {
                let els = self.stmt();
                if let NodeKind::If { cond, then, .. } = node.kind {
                    node.kind = NodeKind::If {
                        cond: cond,
                        then: then,
                        els: Some(Box::new(els)),
                    };
                }
            }
            return node;
        };

        if self.consume("while") {
            self.expect('(');
            let cond = self.expr();
            self.expect(')');
            let then = self.stmt();
            return Node::new(NodeKind::While {
                cond: Box::new(cond),
                then: Box::new(then),
            });
        };

        if self.consume("for") {
            self.expect('(');
            let init = self.stmt();
            let mut cond = None;
            let mut inc = None;

            if !self.consume(';') {
                cond = Some(self.expr());
                self.expect(';');
            }

            if !self.consume(')') {
                inc = Some(self.expr());
                self.expect(')');
            }

            let then = self.stmt();
            return Node::new(NodeKind::For {
                init: Box::new(init),
                cond: cond.map(|c| Box::new(c)),
                inc: inc.map(|i| Box::new(i)),
                then: Box::new(then),
            });
        };

        if self.consume("return") {
            let node = Node::new_unary(NodeKind::Return, self.expr());
            self.expect(';');
            return node;
        };

        if self.consume("{") {
            return self.compound_stmt();
        }

        self.expr_stmt()
    }

    fn compound_stmt(&mut self) -> Node {
        let mut body = Vec::new();
        while !self.consume("}") {
            body.push(if self.equal("int") {
                log::debug!(
                    "declaration, token={:?}, index={}",
                    self.token(),
                    self.index
                );
                self.declaration()
            } else {
                let mut node = self.stmt();
                node.add_type();
                node
            });
        }
        Node::new_block(Some(body))
    }

    fn expr_stmt(&mut self) -> Node {
        if self.consume(';') {
            return Node::new_block(None);
        }

        let node = self.expr();
        self.expect(';');
        node
    }

    fn add(&mut self) -> Node {
        let mut node = self.mul();

        loop {
            if self.consume("+") {
                node = Node::new_add(node, self.mul());
            } else if self.consume("-") {
                node = Node::new_sub(node, self.mul())
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node {
        let mut node = self.unary();

        loop {
            if self.consume("*") {
                node = Node::new_binary(NodeKind::Mul, node, self.unary());
            } else if self.consume("/") {
                node = Node::new_binary(NodeKind::Div, node, self.unary());
            } else {
                return node;
            }
        }
    }

    /// unary = ("+" | "-" | "*" | "&") unary
    ///       | postfix
    fn unary(&mut self) -> Node {
        if self.consume("+") {
            return self.primary();
        } else if self.consume("-") {
            return Node::new_binary(NodeKind::Sub, Node::new_node_num(0), self.primary());
        } else if self.consume("&") {
            return Node::new_unary(NodeKind::Addr, self.unary());
        } else if self.consume("*") {
            return Node::new_unary(NodeKind::Deref, self.unary());
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Node {
        let mut node = self.primary();
        while self.consume('[') {
            let idx = self.expr();
            self.expect(']');
            node = Node::new_unary(NodeKind::Deref, Node::new_add(node, idx))
        }
        node
    }

    fn primary(&mut self) -> Node {
        if self.consume('(') {
            let node = self.expr();
            self.expect(')');
            return node;
        }

        if self.consume("sizeof") {
            let mut node = self.unary();
            node.add_type();
            return Node::new_node_num(node.ty.unwrap().size().unwrap());
        }

        if let TokenKind::Ident = self.token().kind {
            if self.next_equal("(") {
                return self.funcall();
            }

            let lvar = self.find_lvar();
            let node = match lvar {
                Some(lvar) => Node::new_lvar(lvar.clone(), lvar.ty.clone()),
                None => panic!(
                    "undefined variable: {:?}, locals={:?}",
                    self.token(),
                    self.locals
                ),
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

    fn function(&mut self) -> Function {
        let ty = self.declspec();
        let ty = self.declarator(ty);
        self.locals = LinkedList::new();

        if let TypeKind::Func { params, .. } = ty.clone().kind {
            let mut func_params = LinkedList::new();

            log::debug!("function params={:?}", params);
            for param in params.iter() {
                let lvar = self.add_lvar(
                    param.clone().name.unwrap().get_ident().unwrap(),
                    param.clone(),
                );
                func_params.push_back(lvar.clone())
            }
            log::debug!("function token={:?}", self.token());

            let name = ty.clone().name.unwrap().get_ident().unwrap();
            log::debug!("function name={:?}", name);

            self.expect('{');
            return Function {
                name,
                body: self.compound_stmt(),
                params: func_params,
                locals: self.locals.clone(),
            };
        }
        unreachable!("ty is not function")
    }

    fn funcall(&mut self) -> Node {
        let start = self.token().clone();
        self.next();
        self.next();
        let mut args = Vec::new();
        while !self.consume(')') {
            if args.len() > 0 {
                log::debug!("args len={}", args.len());
                self.expect(',');
            }
            args.push(self.assign());
        }
        log::debug!("tokentokentoken={:?}", self.token());
        Node::new(NodeKind::FuncCall {
            name: start.str,
            args,
        })
    }

    fn equality(&mut self) -> Node {
        let mut node = self.relational();

        loop {
            if self.consume("==") {
                node = Node::new_binary(NodeKind::Eq, node, self.relational());
            } else if self.consume("!=") {
                node = Node::new_binary(NodeKind::Ne, node, self.relational());
            } else {
                return node;
            }
        }
    }

    fn relational(&mut self) -> Node {
        let mut node = self.add();

        loop {
            if self.consume("<") {
                node = Node::new_binary(NodeKind::Lt, node, self.add());
            } else if self.consume("<=") {
                node = Node::new_binary(NodeKind::Le, node, self.add());
            } else if self.consume(">") {
                node = Node::new_binary(NodeKind::Lt, self.add(), node);
            } else if self.consume(">=") {
                node = Node::new_binary(NodeKind::Le, self.add(), node);
            } else {
                return node;
            }
        }
    }

    fn expect(&mut self, op: impl Into<String>) {
        let token = self.token();
        let op = op.into();
        if (token.kind != TokenKind::Keyword && token.kind != TokenKind::Punct)
            || token.str.to_string() != op
        {
            panic!("expected: `{}`, actual: `{}`", op, token.str);
        }
        self.next();
    }

    fn consume(&mut self, op: impl Into<String>) -> bool {
        let token = self.token();
        let op = op.into();
        if (token.kind != TokenKind::Keyword && token.kind != TokenKind::Punct)
            || token.str.to_string() != op
        {
            return false;
        }
        self.next();
        true
    }

    fn equal(&mut self, op: impl Into<String>) -> bool {
        let token = self.token();
        let op = op.into();
        if (token.kind != TokenKind::Keyword && token.kind != TokenKind::Punct)
            || token.str.to_string() != op
        {
            return false;
        }
        true
    }

    fn next_equal(&mut self, op: impl Into<String>) -> bool {
        if let Some(token) = self.next_token() {
            let op = op.into();
            if (token.kind != TokenKind::Keyword && token.kind != TokenKind::Punct)
                || token.str.to_string() != op
            {
                return false;
            }
        }
        true
    }
}
