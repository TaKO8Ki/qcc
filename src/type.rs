use crate::{Node, NodeKind, Type, TypeKind};

impl Type {
    pub fn is_integer(&self) -> bool {
        matches!(self.kind, TypeKind::Int)
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self.kind, TypeKind::Ptr(_))
    }

    pub fn base(&self) -> Option<Type> {
        match &self.kind {
            TypeKind::Ptr(base) => Some(*base.clone()),
            _ => None,
        }
    }

    pub fn pointer_to(self) -> Self {
        Self {
            name: None,
            kind: TypeKind::Ptr(Box::new(self)),
        }
    }

    pub fn func_type(&self) -> Self {
        Self {
            name: None,
            kind: TypeKind::Func(Some(Box::new(self.clone()))),
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

        match self.kind {
            NodeKind::Add | NodeKind::Sub | NodeKind::Mul | NodeKind::Div | NodeKind::Assign => {
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
                self.ty = self
                    .lhs
                    .as_ref()
                    .map(|lhs| lhs.ty.clone().map(|ty| ty.pointer_to()))
                    .flatten()
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
