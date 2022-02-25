use crate::{Function, Node, NodeKind, Tokens, TypeKind, Var};

const ARG_REG8: &[&str] = &["dil", "sil", "dl", "cl", "r8b", "r9b"];
const ARG_REG64: &[&str] = &["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

impl Tokens {
    pub(crate) fn codegen(&mut self, asm: &mut Vec<String>) {
        for func in &mut self.functions {
            func.stack_size = Some(func.assign_lvar_offset());
        }
        self.emit_data(asm);
        let mut count = 0;
        for func in &self.functions {
            asm.push(String::from(".intel_syntax noprefix"));
            asm.push(format!(".globl {}", func.name));
            asm.push(String::from(".text"));
            asm.push(format!("{}:", func.name));

            asm.push(String::from("  push rbp"));
            asm.push(String::from("  mov rbp, rsp"));
            log::debug!("stack size={:?}", func.stack_size);
            asm.push(format!("  sub rsp, {}", func.stack_size.unwrap()));

            func.gen_param(asm);

            func.gen_stmt(&func.body, asm, &mut count);
            asm.push(String::from("  pop rax"));

            asm.push(String::from("  mov rsp, rbp"));
            asm.push(String::from("  pop rbp"));
            asm.push(String::from("  ret"));
        }
    }

    fn emit_data(&self, asm: &mut Vec<String>) {
        for global in &self.globals {
            asm.push(String::from(".data"));
            asm.push(format!(".globl {}", global.name));
            asm.push(format!("{}:", global.name));

            if let Some(data) = global.init_data.as_ref() {
                for ch in data.chars() {
                    asm.push(format!("  .byte {}", ch as u8));
                }
                asm.push(String::from("  .byte 0"));
            } else {
                asm.push(format!("  .zero {}", global.ty.size().unwrap()));
            }
        }
    }
}

impl Function {
    fn gen_param(&self, asm: &mut Vec<String>) {
        for (i, var) in self.params.iter().enumerate() {
            let var = self.find_lvar(&var).unwrap();
            asm.push(String::from("  mov rax, rbp"));
            asm.push(format!("  sub rax, {}", var.offset));
            asm.push(String::from("  push rax"));
            asm.push(format!("  push {}", ARG_REG64[i]));
            asm.push(String::from("  pop rdi"));
            asm.push(String::from("  pop rax"));
            if matches!(var.ty.size(), Some(size) if size == 1) {
                asm.push(String::from("  mov [rax], dil"));
            } else {
                asm.push(String::from("  mov [rax], rdi"));
            }
            asm.push(String::from("  push rdi"));
        }
    }

    fn assign_lvar_offset(&mut self) -> usize {
        let mut offset = 0;
        log::debug!("locals={:?}", self.locals);
        for lvar in &mut self.locals.iter_mut() {
            offset += lvar.ty.size().unwrap() as usize;
            lvar.offset = offset;
        }
        (offset + 16 - 1) / 16 * 16
    }

    fn find_lvar(&self, var: &Var) -> Option<&Var> {
        self.locals
            .iter()
            .find(|lvar| lvar.name == var.name && lvar.id == var.id)
    }

    fn load(&self, node: &Node, asm: &mut Vec<String>) {
        if let Some(ty) = &node.ty {
            if let TypeKind::Array { .. } = ty.kind {
                return;
            }
            if matches!(ty.size(), Some(size) if size == 1) {
                asm.push(String::from("  movzx rax, BYTE PTR [rax]"));
                return;
            }
        }

        asm.push(String::from("  mov rax, [rax]"))
    }

    fn store(&self, node: &Node, asm: &mut Vec<String>) {
        if let Some(ty) = &node.ty {
            if matches!(ty.size(), Some(size) if size == 1) {
                asm.push(String::from("  mov [rax], dil"));
                return;
            }
        }

        asm.push(String::from("  mov [rax], rdi"));
    }

    fn gen_lval(&self, node: &Node, asm: &mut Vec<String>, count: &mut usize) {
        match &node.kind {
            NodeKind::Var(var) => {
                if var.is_local {
                    asm.push(String::from("  mov rax, rbp"));
                    asm.push(format!(
                        "  sub rax, {}",
                        self.find_lvar(&var).unwrap().offset
                    ));
                } else {
                    asm.push(format!("  lea rax, {}[rip]", var.name));
                }
                asm.push(String::from("  push rax"));
            }
            NodeKind::Deref => {
                if let Some(node) = node.lhs.as_ref() {
                    self.gen_expr(&node, asm, count);
                }
            }
            _ => unreachable!("not lval"),
        }
    }

    pub fn gen_stmt(&self, node: &Node, asm: &mut Vec<String>, count: &mut usize) {
        match &node.kind {
            NodeKind::Return => {
                if let Some(node) = node.lhs.as_ref() {
                    self.gen_expr(node, asm, count);
                }
                asm.push(String::from("  pop rax"));
                asm.push(String::from("  mov rsp, rbp"));
                asm.push(String::from("  pop rbp"));
                asm.push(String::from("  ret"));
                return;
            }
            NodeKind::Block { body } => {
                for node in body.iter() {
                    self.gen_stmt(node, asm, count);
                }
                return;
            }
            NodeKind::ExprStmt => {
                if let Some(node) = node.lhs.as_ref() {
                    self.gen_expr(&node, asm, count);
                    asm.push(String::from("  add rsp, 8"));
                }
                return;
            }
            NodeKind::If { cond, then, els } => {
                *count += 1;
                let c = count.clone();
                self.gen_expr(&cond, asm, count);
                asm.push(String::from("  pop rax"));
                asm.push(String::from("  cmp rax, 0"));
                asm.push(format!("  je .L.else{}", c));
                self.gen_stmt(&then, asm, count);
                asm.push(format!("  jmp .L.end{}", c));
                asm.push(format!(".L.else{}:", c));
                if let Some(els) = els {
                    self.gen_stmt(&els, asm, count);
                }
                asm.push(format!(".L.end{}:", c));
                return;
            }
            NodeKind::While { cond, then } => {
                *count += 1;
                let c = count.clone();
                asm.push(format!(".L.begin{}:", c));
                self.gen_expr(&cond, asm, count);
                asm.push(String::from("  pop rax"));
                asm.push(String::from("  cmp rax, 0"));
                asm.push(format!("  je .L.end{}", c));
                self.gen_stmt(&then, asm, count);
                asm.push(format!("  jmp .L.begin{}", c));
                asm.push(format!(".L.end{}:", c));
                return;
            }
            NodeKind::For {
                init,
                cond,
                inc,
                then,
            } => {
                *count += 1;
                let c = count.clone();
                self.gen_stmt(&init, asm, count);
                asm.push(format!(".L.begin{}:", c));
                if let Some(cond) = cond {
                    self.gen_expr(&cond, asm, count);
                    asm.push(String::from("  pop rax"));
                    asm.push(String::from("  cmp rax, 0"));
                    asm.push(format!("  je .L.end{}", c));
                }
                self.gen_stmt(&then, asm, count);
                if let Some(inc) = inc {
                    self.gen_expr(&inc, asm, count);
                }
                asm.push(format!("  jmp .L.begin{}", c));
                asm.push(format!(".L.end{}:", c));
                return;
            }
            _ => (),
        }
    }

    pub fn gen_expr(&self, node: &Node, asm: &mut Vec<String>, count: &mut usize) {
        asm.push(format!("  .loc 1 {}", node.token.line_number));
        match &node.kind {
            NodeKind::Num(val) => {
                asm.push(format!("  push {}", val));
                return;
            }
            NodeKind::Var { .. } => {
                self.gen_lval(&node, asm, count);
                asm.push(String::from("  pop rax"));
                self.load(&node, asm);
                asm.push(String::from("  push rax"));
                return;
            }
            NodeKind::Assign => {
                if let Some(node) = node.lhs.as_ref() {
                    self.gen_lval(&node, asm, count);
                }
                if let Some(node) = node.rhs.as_ref() {
                    self.gen_expr(&node, asm, count);
                }

                asm.push(String::from("  pop rdi"));
                asm.push(String::from("  pop rax"));
                self.store(&node, asm);
                asm.push(String::from("  push rdi"));
                return;
            }
            NodeKind::Addr => {
                if let Some(node) = node.lhs.as_ref() {
                    self.gen_lval(&node, asm, count);
                }
                return;
            }
            NodeKind::Deref => {
                if let Some(node) = node.lhs.as_ref() {
                    self.gen_expr(&node, asm, count);
                }
                asm.push(String::from("  pop rax"));
                self.load(&node, asm);
                asm.push(String::from("  push rax"));
                return;
            }
            NodeKind::StmtExpr { body } => {
                let mut body = body.clone();
                let last = body.pop().unwrap();
                for node in body.iter() {
                    self.gen_stmt(&node, asm, count);
                }
                self.gen_expr(&last, asm, count);
                return;
            }
            NodeKind::FuncCall { name, args } => {
                let mut nargs = 0;
                for arg in args {
                    self.gen_expr(&arg, asm, count);
                    nargs += 1;
                }

                for i in (0..nargs).rev() {
                    asm.push(format!("  pop {}", ARG_REG64[i]));
                }

                asm.push(String::from("  mov rax, 0"));
                asm.push(format!("  call {}", name));
                asm.push(String::from("  push rax"));
                return;
            }
            _ => (),
        }

        if let Some(node) = node.lhs.as_ref() {
            self.gen_expr(&node, asm, count);
        }
        if let Some(node) = node.rhs.as_ref() {
            self.gen_expr(&node, asm, count);
        }
        asm.push(String::from("  pop rdi"));
        asm.push(String::from("  pop rax"));

        match node.kind {
            NodeKind::Add => {
                asm.push(String::from("  add rax, rdi"));
            }
            NodeKind::Sub => {
                asm.push(String::from("  sub rax, rdi"));
            }
            NodeKind::Mul => {
                asm.push(String::from("  imul rax, rdi"));
            }
            NodeKind::Div => {
                asm.push(String::from("  cqo"));
                asm.push(String::from("  idiv rdi"));
            }
            NodeKind::Eq => {
                asm.push(String::from("  cmp rax, rdi"));
                asm.push(String::from("  sete al"));
                asm.push(String::from("  movzb rax, al"));
            }
            NodeKind::Ne => {
                asm.push(String::from("  cmp rax, rdi"));
                asm.push(String::from("  setne al"));
                asm.push(String::from("  movzb rax, al"));
            }
            NodeKind::Lt => {
                asm.push(String::from("  cmp rax, rdi"));
                asm.push(String::from("  setl al"));
                asm.push(String::from("  movzb rax, al"));
            }
            NodeKind::Le => {
                asm.push(String::from("  cmp rax, rdi"));
                asm.push(String::from("  setle al"));
                asm.push(String::from("  movzb rax, al"));
            }
            _ => {}
        }

        asm.push(String::from("  push rax"));
    }
}
