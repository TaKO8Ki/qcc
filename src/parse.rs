use crate::{
    Function, Member, Node, NodeKind, Scope, Token, TokenKind, Tokens, Type, TypeKind, Var,
    VarScope,
};
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
    fn new(kind: NodeKind, token: &Token) -> Self {
        Node {
            kind,
            lhs: None,
            rhs: None,
            ty: None,
            token: token.clone(),
        }
    }

    fn new_binary(kind: NodeKind, lhs: Node, rhs: Node, token: &Token) -> Self {
        Node {
            kind,
            lhs: Some(Box::new(lhs)),
            rhs: Some(Box::new(rhs)),
            ty: None,
            token: token.clone(),
        }
    }

    fn new_unary(kind: NodeKind, lhs: Node, token: &Token) -> Self {
        Node {
            kind,
            lhs: Some(Box::new(lhs)),
            rhs: None,
            ty: None,
            token: token.clone(),
        }
    }

    fn new_node_num(val: u16, token: &Token) -> Self {
        Node {
            kind: NodeKind::Num(val),
            lhs: None,
            rhs: None,
            ty: None,
            token: token.clone(),
        }
    }

    fn new_node_var(var: Var, ty: Type, token: &Token) -> Self {
        Node {
            kind: NodeKind::Var(var),
            lhs: None,
            rhs: None,
            ty: Some(ty),
            token: token.clone(),
        }
    }

    fn new_block(body: Vec<Node>, token: &Token) -> Self {
        Node {
            kind: NodeKind::Block {
                body: Box::new(body),
            },
            lhs: None,
            rhs: None,
            ty: None,
            token: token.clone(),
        }
    }

    fn new_add(lhs: Node, rhs: Node, token: &Token) -> Self {
        let mut lhs = lhs;
        let mut rhs = rhs;

        lhs.add_type();
        rhs.add_type();

        if let Some(lhs_ty) = &lhs.ty {
            let lhs_ty = lhs_ty.clone();
            if let Some(rhs_ty) = &rhs.ty {
                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    return Node::new_binary(NodeKind::Add, lhs, rhs, token);
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
                        Self::new_node_num(lhs_ty.base().unwrap().size().unwrap(), token),
                        token,
                    ),
                    token,
                );
            }
        }

        unreachable!("invalid operands")
    }

    fn new_sub(lhs: Node, rhs: Node, token: &Token) -> Self {
        let mut lhs = lhs;
        let mut rhs = rhs;

        lhs.add_type();
        rhs.add_type();

        if let Some(lhs_ty) = &lhs.ty {
            if let Some(rhs_ty) = &rhs.ty {
                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    return Node::new_binary(NodeKind::Sub, lhs, rhs, token);
                }

                if rhs_ty.is_integer() {
                    match &lhs_ty.kind {
                        TypeKind::Ptr { base, .. } | TypeKind::Array { base, .. } => {
                            let mut rhs = Node::new_binary(
                                NodeKind::Mul,
                                rhs,
                                Self::new_node_num(base.size().unwrap(), token),
                                token,
                            );
                            rhs.add_type();
                            let ty = lhs.ty.clone();
                            let mut node = Node::new_binary(NodeKind::Sub, lhs, rhs, token);
                            node.ty = ty;
                            return node;
                        }
                        _ => (),
                    }
                }

                if rhs_ty.is_pointer() {
                    match &lhs_ty.clone().kind {
                        TypeKind::Ptr { base, .. } | TypeKind::Array { base, .. } => {
                            let mut node = Node::new_binary(NodeKind::Sub, lhs, rhs, token);
                            node.ty = Some(Type::type_int());
                            return Node::new_binary(
                                NodeKind::Div,
                                node,
                                Self::new_node_num(base.size().unwrap(), token),
                                token,
                            );
                        }
                        _ => (),
                    }
                }
            }
        }

        panic!("invalid operands: lhs={:?}, rhs={:?}", lhs, rhs);
    }
}

impl Tokens {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut scope = LinkedList::new();
        scope.push_front(Scope::default());
        Tokens {
            locals: LinkedList::new(),
            globals: LinkedList::new(),
            scope,
            tokens,
            index: 0,
            functions: LinkedList::new(),
            string_literal_id: 0,
        }
    }

    fn next(&mut self) -> Option<&Token> {
        self.index += 1;
        self.tokens.get(self.index - 1)
    }

    fn next_token(&self) -> Option<&Token> {
        self.tokens.get(self.index + 1)
    }

    fn find_var(&self) -> Option<&Var> {
        for scope in self.scope.iter().rev() {
            for var in scope.vars.iter() {
                if var.name.len() == self.token().str.len() && var.name == self.token().str {
                    return Some(&var.var);
                }
            }
        }
        None
    }

    fn enter_scope(&mut self) {
        self.scope.push_front(Scope::default());
    }

    fn leave_scope(&mut self) {
        self.scope.pop_front();
    }

    fn token(&self) -> &Token {
        &self.tokens[self.index]
    }

    pub fn expr(&mut self) -> Node {
        let mut node = self.assign();

        let token = self.token().clone();
        while self.consume(',') {
            node = Node::new_unary(NodeKind::ExprStmt, node, &token);
            node = Node::new_binary(NodeKind::Comma, node, self.assign(), &token);
        }
        node
    }

    fn assign(&mut self) -> Node {
        let mut node = self.equality();
        if self.consume("=") {
            node = Node::new_binary(NodeKind::Assign, node, self.assign(), self.token());
        }
        node
    }

    fn global_variable(&mut self) {
        let ty = self.declspec();
        let mut first = true;

        while !self.consume(';') {
            if !first {
                self.expect(',');
            }
            first = false;

            let ty = self.declarator(ty.clone());
            let gvar = self.add_gvar(
                ty.clone().name.unwrap().get_ident().unwrap(),
                ty.clone(),
                None,
            );
            Node::new_node_var(gvar, ty, self.token());
        }
    }

    pub fn is_function(&mut self) -> bool {
        if self.equal(';') {
            return false;
        }
        let mut tokens = Self {
            tokens: self.tokens.clone(),
            locals: LinkedList::new(),
            globals: LinkedList::new(),
            scope: LinkedList::new(),
            index: self.index,
            functions: LinkedList::new(),
            string_literal_id: 0,
        };
        let ty = tokens.declspec();
        let ty = tokens.declarator(ty);
        matches!(ty.kind, TypeKind::Func { .. })
    }

    pub fn program(&mut self) {
        loop {
            log::debug!("program token={:?}", self.token());
            if let TokenKind::Eof = self.token().kind {
                break;
            }

            if self.is_function() {
                let function = self.function();
                self.functions.push_back(function);
                continue;
            }

            self.global_variable();
        }
        log::debug!("functions={:?}", self.functions);
    }

    fn push_scope(&mut self, name: String, var: Var) -> Option<&VarScope> {
        let sc = VarScope { name, var };
        if let Some(scope) = self.scope.front_mut() {
            scope.vars.push_front(sc);
            return scope.vars.front();
        }
        None
    }

    fn add_lvar(&mut self, name: String, ty: Type) -> Var {
        let lvar = Var {
            id: self.locals.front().map_or(0, |lvar| lvar.id + 1),
            name: name.clone(),
            offset: 0,
            ty,
            is_local: true,
            init_data: None,
        };
        self.locals.push_front(lvar.clone());
        self.push_scope(name, lvar.clone());
        lvar
    }

    fn add_gvar(&mut self, name: String, ty: Type, init_data: Option<String>) -> Var {
        let gvar = Var {
            id: 0,
            name: name.clone(),
            offset: 0,
            ty,
            is_local: false,
            init_data,
        };
        self.globals.push_front(gvar.clone());
        self.push_scope(name, gvar.clone());
        gvar
    }

    fn new_string_literal(&mut self, ty: Type, init_data: String) -> Var {
        let name = format!(".L..{}", self.string_literal_id);
        self.string_literal_id += 1;
        self.add_gvar(name, ty, Some(init_data))
    }

    fn get_number(&self) -> u16 {
        if let TokenKind::Num(val) = self.token().kind {
            return val;
        }
        unreachable!("expected a number: {:?}", self.token());
    }

    fn declspec(&mut self) -> Type {
        if self.consume("char") {
            return Type::type_char();
        }

        if self.consume("int") {
            return Type::type_int();
        }

        if self.consume("struct") {
            return self.struct_decl();
        }

        self.error_token("typename expected");
        unreachable!()
    }

    fn struct_members(&mut self) -> Type {
        let mut members = Vec::new();
        let token = self.token().clone();

        let mut offset = 0;
        while !self.consume('}') {
            let basety = self.declspec();

            let mut i = 0;
            log::debug!("struct members={:?} i={}", basety, i);
            while !self.consume(';') {
                log::debug!("i={}", i);
                if i != 0 {
                    self.expect(',');
                }
                i += 1;

                let ty = self.declarator(basety.clone());
                members.push(Member {
                    ty: ty.clone(),
                    name: ty.name.clone(),
                    offset,
                });
                offset += ty.size().unwrap();
            }
        }

        Type::type_struct(members, offset, token)
    }

    fn struct_decl(&mut self) -> Type {
        self.expect('{');
        self.struct_members()
    }

    fn struct_ref(&self, lhs: &mut Node) -> Node {
        lhs.add_type();
        match &lhs.ty {
            Some(ty) if matches!(ty.kind, TypeKind::Struct { .. }) => (),
            _ => self.error_token("not a struct`"),
        }

        Node::new_unary(
            NodeKind::Member(self.get_struct_member(lhs.ty.clone().unwrap())),
            lhs.clone(),
            self.token(),
        )
    }

    fn get_struct_member(&self, ty: Type) -> Member {
        if let TypeKind::Struct { members, .. } = ty.kind {
            log::debug!(
                "struct members={:?} token name={} token loc={}",
                members,
                self.token().str,
                self.token().loc
            );
            let name = self.token().clone();
            let member = members
                .iter()
                .find(|member| {
                    member.name.as_ref().map_or(false, |member_name| {
                        member_name.str == name.str && member_name.loc != name.loc
                    })
                })
                .unwrap();
            return member.clone();
        }
        self.error_token("no such member");
        unreachable!()
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
            let lhs = Node::new_node_var(lvar, ty, self.token());

            if !self.consume('=') {
                continue;
            }

            let rhs = self.assign();
            let node = Node::new_binary(NodeKind::Assign, lhs, rhs, self.token());
            body.push(Node::new_unary(NodeKind::ExprStmt, node, self.token()));
        }

        log::debug!("body={:?}", body);
        let node = Node::new_block(body, self.token());
        log::debug!("declaration last token={:?}", self.token());
        node
    }

    fn stmt(&mut self) -> Node {
        if self.consume("if") {
            self.expect('(');
            let cond = self.expr();
            self.expect(')');
            let then = self.stmt();
            let mut node = Node::new(
                NodeKind::If {
                    cond: Box::new(cond),
                    then: Box::new(then),
                    els: None,
                },
                self.token(),
            );
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
            return Node::new(
                NodeKind::While {
                    cond: Box::new(cond),
                    then: Box::new(then),
                },
                self.token(),
            );
        };

        if self.consume("for") {
            self.expect('(');
            let init = self.expr_stmt();
            self.expect(';');
            let mut cond = None;
            let mut inc = None;

            if !self.consume(';') {
                cond = Some(self.expr());
                self.expect(';');
            }

            if !self.consume(')') {
                inc = Some(self.expr_stmt());
                self.expect(')');
            }

            let then = self.stmt();
            return Node::new(
                NodeKind::For {
                    init: Box::new(init),
                    cond: cond.map(|c| Box::new(c)),
                    inc: inc.map(|i| Box::new(i)),
                    then: Box::new(then),
                },
                self.token(),
            );
        };

        if self.consume("return") {
            let node = Node::new_unary(NodeKind::Return, self.expr(), self.token());
            self.expect(';');
            return node;
        };

        if self.consume("{") {
            return self.compound_stmt();
        }

        let node = self.expr_stmt();
        self.expect(';');
        node
    }

    fn compound_stmt(&mut self) -> Node {
        let mut body = Vec::new();
        self.enter_scope();
        while !self.consume("}") {
            let mut node = if self.is_type_name() {
                log::debug!(
                    "declaration, token={:?}, index={}",
                    self.token(),
                    self.index
                );
                self.declaration()
            } else {
                self.stmt()
            };
            node.add_type();
            body.push(node);
        }
        self.leave_scope();
        Node::new_block(body, self.token())
    }

    fn expr_stmt(&mut self) -> Node {
        if self.consume(';') {
            return Node::new_block(Vec::new(), self.token());
        }

        let node = Node::new_unary(NodeKind::ExprStmt, self.expr(), self.token());
        node
    }

    fn add(&mut self) -> Node {
        let mut node = self.mul();

        loop {
            if self.consume('+') {
                node = Node::new_add(node, self.mul(), self.token());
            } else if self.consume('-') {
                node = Node::new_sub(node, self.mul(), self.token())
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node {
        let mut node = self.unary();

        loop {
            if self.consume("*") {
                node = Node::new_binary(NodeKind::Mul, node, self.unary(), self.token());
            } else if self.consume("/") {
                node = Node::new_binary(NodeKind::Div, node, self.unary(), self.token());
            } else {
                return node;
            }
        }
    }

    /// unary = ("+" | "-" | "*" | "&") unary
    ///       | postfix
    fn unary(&mut self) -> Node {
        if self.consume('+') {
            return self.unary();
        } else if self.consume('-') {
            return Node::new_binary(
                NodeKind::Sub,
                Node::new_node_num(0, self.token()),
                self.unary(),
                self.token(),
            );
        } else if self.consume('&') {
            return Node::new_unary(NodeKind::Addr, self.unary(), self.token());
        } else if self.consume('*') {
            return Node::new_unary(NodeKind::Deref, self.unary(), self.token());
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Node {
        let mut node = self.primary();

        loop {
            if self.consume('[') {
                let idx = self.expr();
                self.expect(']');
                node = Node::new_unary(
                    NodeKind::Deref,
                    Node::new_add(node, idx, self.token()),
                    self.token(),
                );
                continue;
            }

            if self.consume('.') {
                node = self.struct_ref(&mut node);
                self.next();
                continue;
            }

            return node;
        }
    }

    fn primary(&mut self) -> Node {
        if self.consume('(') {
            if self.consume('{') {
                let mut body = self.compound_stmt().body().unwrap();
                if let Some(last_node) = body.pop() {
                    body.push(*last_node.lhs.unwrap());
                }
                let node = Node::new(
                    NodeKind::StmtExpr {
                        body: Box::new(body),
                    },
                    self.token(),
                );
                self.expect(')');
                return node;
            }
            let node = self.expr();
            self.expect(')');
            return node;
        }

        if self.consume("sizeof") {
            let mut node = self.unary();
            node.add_type();
            return Node::new_node_num(node.ty.unwrap().size().unwrap(), self.token());
        }

        if let TokenKind::Ident = self.token().kind {
            if self.next_equal("(") {
                return self.funcall();
            }

            let var = self.find_var();
            let node = match var {
                Some(var) => Node::new_node_var(var.clone(), var.ty.clone(), self.token()),
                None => panic!(
                    "undefined variable: {:?}, locals={:?}, global={:?}, scope={:?}",
                    self.token(),
                    self.locals,
                    self.globals,
                    self.scope
                ),
            };

            self.next();
            return node;
        }

        if let TokenKind::Str { ty, str } = self.token().clone().kind {
            let var = self.new_string_literal(*ty, str);
            log::debug!("string literal: {:?}", var);
            self.next();
            return Node::new_node_var(var.clone(), var.ty, self.token());
        }

        if let TokenKind::Num(val) = self.token().kind {
            let node = Node::new_node_num(val, self.token());
            self.next();
            return node;
        }

        panic!("primary: unexpected token {:?}", self.token());
    }

    fn function(&mut self) -> Function {
        let ty = self.declspec();
        let ty = self.declarator(ty);
        self.locals = LinkedList::new();
        self.enter_scope();

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
            let function = Function {
                name,
                body: self.compound_stmt(),
                params: func_params,
                locals: self.locals.clone(),
                stack_size: None,
            };
            self.leave_scope();
            return function;
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
        Node::new(
            NodeKind::FuncCall {
                name: start.str,
                args,
            },
            self.token(),
        )
    }

    fn equality(&mut self) -> Node {
        let mut node = self.relational();

        loop {
            if self.consume("==") {
                node = Node::new_binary(NodeKind::Eq, node, self.relational(), self.token());
            } else if self.consume("!=") {
                node = Node::new_binary(NodeKind::Ne, node, self.relational(), self.token());
            } else {
                return node;
            }
        }
    }

    fn relational(&mut self) -> Node {
        let mut node = self.add();

        loop {
            if self.consume("<") {
                node = Node::new_binary(NodeKind::Lt, node, self.add(), self.token());
            } else if self.consume("<=") {
                node = Node::new_binary(NodeKind::Le, node, self.add(), self.token());
            } else if self.consume(">") {
                node = Node::new_binary(NodeKind::Lt, self.add(), node, self.token());
            } else if self.consume(">=") {
                node = Node::new_binary(NodeKind::Le, self.add(), node, self.token());
            } else {
                return node;
            }
        }
    }

    fn expect(&mut self, op: impl Into<String>) {
        let token = self.token();
        let op = op.into();
        if matches!(token.kind, TokenKind::Keyword) && matches!(token.kind, TokenKind::Punct)
            || token.str.to_string() != op
        {
            self.error_token(format!("expected: `{}`, actual: `{}`", op, token.str))
        }
        self.next();
    }

    fn consume(&mut self, op: impl Into<String>) -> bool {
        let token = self.token();
        let op = op.into();
        if matches!(token.kind, TokenKind::Keyword) && matches!(token.kind, TokenKind::Punct)
            || token.str.to_string() != op
        {
            return false;
        }
        self.next();
        true
    }

    fn equal(&self, op: impl Into<String>) -> bool {
        let token = self.token();
        let op = op.into();
        if !matches!(token.kind, TokenKind::Keyword) && !matches!(token.kind, TokenKind::Punct)
            || token.str.to_string() != op
        {
            return false;
        }
        true
    }

    fn next_equal(&self, op: impl Into<String>) -> bool {
        if let Some(token) = self.next_token() {
            let op = op.into();
            if !matches!(token.kind, TokenKind::Keyword) && !matches!(token.kind, TokenKind::Punct)
                || token.str.to_string() != op
            {
                return false;
            }
        }
        true
    }

    fn is_type_name(&self) -> bool {
        self.equal("int") || self.equal("char") || self.equal("struct")
    }

    fn error_token(&self, msg: impl Into<String>) {
        panic!(
            "{}\n{}^ {} {}:{}",
            self.tokens
                .iter()
                .map(|token| token.str.clone())
                .collect::<String>(),
            (1..self.index).map(|_| " ").collect::<String>(),
            msg.into(),
            self.token().line_number,
            self.token().loc,
        )
    }
}
