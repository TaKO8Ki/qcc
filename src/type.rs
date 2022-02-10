use crate::{Node, NodeKind, Type, TypeKind};

impl Type {
    pub fn type_int() -> Self {
        Self {
            kind: TypeKind::Int { size: 8 },
            name: None,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.kind, TypeKind::Int { .. })
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self.kind, TypeKind::Ptr { .. })
    }

    pub fn base(&self) -> Option<Type> {
        match &self.kind {
            TypeKind::Ptr { base, .. } | TypeKind::Array { base, .. } => Some(*base.clone()),
            _ => None,
        }
    }

    pub fn size(&self) -> Option<u16> {
        match &self.kind {
            TypeKind::Int { size, .. }
            | TypeKind::Ptr { size, .. }
            | TypeKind::Array { size, .. } => Some(size.clone()),
            _ => None,
        }
    }

    pub fn pointer_to(self) -> Self {
        Self {
            name: None,
            kind: TypeKind::Ptr {
                size: 8,
                base: Box::new(self),
            },
        }
    }

    pub fn func_type(&self, params: Vec<Type>) -> Self {
        Self {
            name: None,
            kind: TypeKind::Func {
                params: Box::new(params),
                return_ty: Some(Box::new(self.clone())),
            },
        }
    }

    pub fn array_of(self, len: u16) -> Self {
        match self.size() {
            Some(size) => Self {
                name: None,
                kind: TypeKind::Array {
                    base: Box::new(self),
                    size: size * len,
                    len,
                },
            },
            None => unreachable!("size does not exist"),
        }
    }
}

impl Node {
    pub fn add_type(&mut self) {
        if self.ty.is_some() {
            return;
        }

        if let Some(lhs) = self.lhs.as_mut() {
            lhs.add_type();
        }
        if let Some(rhs) = self.rhs.as_mut() {
            rhs.add_type();
        }

        if let Some(body) = self.body.as_mut() {
            for node in body.iter_mut() {
                node.add_type();
            }
        }

        if let Some(body) = self.body.as_mut() {
            for node in body.iter_mut() {
                node.add_type();
            }
        }

        if let NodeKind::FuncCall { args, .. } = &mut self.kind {
            for arg in args.iter_mut() {
                arg.add_type();
            }
        }

        match self.kind {
            NodeKind::Add | NodeKind::Sub | NodeKind::Mul | NodeKind::Div => {
                self.ty = self.lhs.as_ref().map(|lhs| lhs.ty.clone()).flatten()
            }
            NodeKind::Assign => {
                if let Some(lhs) = &self.lhs {
                    if let Some(ty) = &lhs.ty {
                        if let TypeKind::Array { .. } = ty.kind {
                            panic!("not an lvalue");
                        }
                    }
                }
                self.ty = self.lhs.as_ref().map(|lhs| lhs.ty.clone()).flatten()
            }
            NodeKind::Eq
            | NodeKind::Ne
            | NodeKind::Lt
            | NodeKind::Le
            | NodeKind::LVar(_)
            | NodeKind::Num(_)
            | NodeKind::FuncCall { .. } => self.ty = Some(Type::type_int()),
            NodeKind::Addr => {
                self.ty = if let Some(TypeKind::Array { base, .. }) = self
                    .lhs
                    .as_ref()
                    .map(|lhs| lhs.ty.as_ref())
                    .flatten()
                    .map(|ty| ty.clone().kind)
                {
                    Some(base.pointer_to())
                } else {
                    self.lhs
                        .as_ref()
                        .map(|lhs| lhs.ty.clone().map(|ty| ty.pointer_to()))
                        .flatten()
                };
            }
            NodeKind::Deref => {
                if let Some(base) = self
                    .lhs
                    .as_ref()
                    .map(|lhs| lhs.ty.clone().map(|ty| ty.base()))
                    .flatten()
                    .unwrap_or_default()
                {
                    self.ty = Some(base)
                } else {
                    self.ty = Some(Type::type_int())
                }
            }
            _ => {}
        }

        log::debug!("type={:?}", self.ty);
    }
}
