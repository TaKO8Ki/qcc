use crate::{Function, Node, NodeKind, Tokens, TypeKind};

const ARG_REG8: &[&str] = &["dil", "sil", "dl", "cl", "r8b", "r9b"];
const ARG_REG64: &[&str] = &["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

impl Tokens {
    pub(crate) fn codegen(&self, asm: &mut Vec<String>) {
        self.emit_data(asm);
        let mut count = 0;
        for func in &self.functions {
            asm.push(String::from(".intel_syntax noprefix"));
            asm.push(format!(".globl {}", func.name));
            asm.push(String::from(".text"));
            asm.push(format!("{}:", func.name));

            asm.push(String::from("  push rbp"));
            asm.push(String::from("  mov rbp, rsp"));
            asm.push(String::from("  sub rsp, 208"));

            func.gen_param(asm);

            func.body.gen_stmt(asm, &mut count);
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
            } else {
                asm.push(format!("  .zero {}", global.ty.size().unwrap()));
            }
        }
    }
}

impl Function {
    fn gen_param(&self, asm: &mut Vec<String>) {
        for (i, var) in self.params.iter().enumerate() {
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
}

impl Node {
    fn load(&self, asm: &mut Vec<String>) {
        if let Some(ty) = &self.ty {
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

    fn store(&self, asm: &mut Vec<String>) {
        if let Some(ty) = &self.ty {
            if matches!(ty.size(), Some(size) if size == 1) {
                asm.push(String::from("  mov [rax], dil"));
                return;
            }
        }

        asm.push(String::from("  mov [rax], rdi"));
    }

    fn gen_lval(&self, asm: &mut Vec<String>, count: &mut usize) {
        match &self.kind {
            NodeKind::Var(var) => {
                if var.is_local {
                    asm.push(String::from("  mov rax, rbp"));
                    asm.push(format!("  sub rax, {}", var.offset));
                } else {
                    asm.push(format!("  lea rax, {}[rip]", var.name));
                }
                asm.push(String::from("  push rax"));
            }
            NodeKind::Deref => {
                if let Some(node) = self.lhs.as_ref() {
                    node.gen_expr(asm, count);
                }
            }
            _ => unreachable!("not lval"),
        }
    }

    pub fn gen_stmt(&self, asm: &mut Vec<String>, count: &mut usize) {
        match &self.kind {
            NodeKind::Return => {
                if let Some(node) = self.lhs.as_ref() {
                    node.gen_expr(asm, count);
                }
                asm.push(String::from("  pop rax"));
                asm.push(String::from("  mov rsp, rbp"));
                asm.push(String::from("  pop rbp"));
                asm.push(String::from("  ret"));
                return;
            }
            NodeKind::Block { body } => {
                for node in body.iter() {
                    node.gen_stmt(asm, count);
                }
                return;
            }
            NodeKind::ExprStmt => {
                if let Some(node) = self.lhs.as_ref() {
                    node.gen_expr(asm, count);
                }
                return;
            }
            NodeKind::If { cond, then, els } => {
                *count += 1;
                let c = count.clone();
                cond.gen_expr(asm, count);
                asm.push(String::from("  pop rax"));
                asm.push(String::from("  cmp rax, 0"));
                asm.push(format!("  je .L.else{}", c));
                then.gen_stmt(asm, count);
                asm.push(format!("  jmp .L.end{}", c));
                asm.push(format!(".L.else{}:", c));
                if let Some(els) = els {
                    els.gen_stmt(asm, count);
                }
                asm.push(format!(".L.end{}:", c));
                return;
            }
            NodeKind::While { cond, then } => {
                *count += 1;
                let c = count.clone();
                asm.push(format!(".L.begin{}:", c));
                cond.gen_expr(asm, count);
                asm.push(String::from("  pop rax"));
                asm.push(String::from("  cmp rax, 0"));
                asm.push(format!("  je .L.end{}", c));
                then.gen_stmt(asm, count);
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
                init.gen_expr(asm, count);
                asm.push(format!(".L.begin{}:", c));
                if let Some(cond) = cond {
                    cond.gen_expr(asm, count);
                    asm.push(String::from("  pop rax"));
                    asm.push(String::from("  cmp rax, 0"));
                    asm.push(format!("  je .L.end{}", c));
                }
                then.gen_stmt(asm, count);
                if let Some(inc) = inc {
                    inc.gen_expr(asm, count);
                }
                asm.push(format!("  jmp .L.begin{}", c));
                asm.push(format!(".L.end{}:", c));
                return;
            }
            _ => self.gen_expr(asm, count),
        }
    }

    pub fn gen_expr(&self, asm: &mut Vec<String>, count: &mut usize) {
        match &self.kind {
            NodeKind::Num(val) => {
                asm.push(format!("  push {}", val));
                return;
            }
            NodeKind::Var { .. } => {
                self.gen_lval(asm, count);
                asm.push(String::from("  pop rax"));
                self.load(asm);
                asm.push(String::from("  push rax"));
                return;
            }
            NodeKind::Assign => {
                if let Some(node) = self.lhs.as_ref() {
                    node.gen_lval(asm, count);
                }
                if let Some(node) = self.rhs.as_ref() {
                    node.gen_expr(asm, count);
                }

                asm.push(String::from("  pop rdi"));
                asm.push(String::from("  pop rax"));
                self.store(asm);
                asm.push(String::from("  push rdi"));
                return;
            }
            NodeKind::Addr => {
                if let Some(node) = self.lhs.as_ref() {
                    node.gen_lval(asm, count);
                }
                return;
            }
            NodeKind::Deref => {
                if let Some(node) = self.lhs.as_ref() {
                    node.gen_expr(asm, count);
                }
                asm.push(String::from("  pop rax"));
                self.load(asm);
                asm.push(String::from("  push rax"));
                return;
            }
            NodeKind::StmtExpr { body } => {
                for node in body.iter() {
                    node.gen_stmt(asm, count);
                }
                return;
            }
            NodeKind::FuncCall { name, args } => {
                let mut nargs = 0;
                for arg in args {
                    arg.gen_expr(asm, count);
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

        if let Some(node) = self.lhs.as_ref() {
            node.gen_expr(asm, count);
        }
        if let Some(node) = self.rhs.as_ref() {
            node.gen_expr(asm, count);
        }
        asm.push(String::from("  pop rdi"));
        asm.push(String::from("  pop rax"));

        match self.kind {
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
