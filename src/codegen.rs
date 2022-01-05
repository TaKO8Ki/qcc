use crate::{Node, NodeKind};

impl Node {
    pub fn codegen(asm: &mut Vec<String>, code: Vec<Node>) {
        asm.push(String::from(".intel_syntax noprefix"));
        asm.push(String::from(".globl main"));
        asm.push(String::from("main:"));

        asm.push(String::from("  push rbp"));
        asm.push(String::from("  mov rbp, rsp"));
        asm.push(String::from("  sub rsp, 208"));

        for node in code {
            node.gen(asm);
            asm.push(String::from("  pop rax"));
        }

        asm.push(String::from("  mov rsp, rbp"));
        asm.push(String::from("  pop rbp"));
        asm.push(String::from("  ret"));
    }

    fn gen_lval(&self, asm: &mut Vec<String>) {
        if let NodeKind::LVar(offset) = self.kind {
            asm.push(String::from("  mov rax, rbp"));
            asm.push(format!("  sub rax, {}", offset));
            asm.push(String::from("  push rax"));
        }
    }

    pub fn gen(&self, asm: &mut Vec<String>) {
        match self.kind {
            NodeKind::Num(val) => {
                asm.push(format!("  push {}", val));
                return;
            }
            NodeKind::LVar(_) => {
                self.gen_lval(asm);
                asm.push(String::from("  pop rax"));
                asm.push(String::from("  mov rax, [rax]"));
                asm.push(String::from("  push rax"));
                return;
            }
            NodeKind::Assign => {
                if let Some(node) = self.lhs.as_ref() {
                    node.gen_lval(asm);
                }
                if let Some(node) = self.rhs.as_ref() {
                    node.gen(asm);
                }

                asm.push(String::from("  pop rdi"));
                asm.push(String::from("  pop rax"));
                asm.push(String::from("  mov [rax], rdi"));
                asm.push(String::from("  push rdi"));
                return;
            }
            NodeKind::Return => {
                if let Some(node) = self.lhs.as_ref() {
                    node.gen(asm);
                }
                asm.push(String::from("  pop rax"));
                asm.push(String::from("  mov rsp, rbp"));
                asm.push(String::from("  pop rbp"));
                asm.push(String::from("  ret"));
                return;
            }
            _ => (),
        }

        if let Some(node) = self.lhs.as_ref() {
            node.gen(asm);
        }
        if let Some(node) = self.rhs.as_ref() {
            node.gen(asm);
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
