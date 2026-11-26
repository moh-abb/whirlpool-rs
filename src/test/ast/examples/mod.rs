use crate::ast::expr::Expr;
use crate::ast::expr::ExprNode;
use crate::ast::expr::ExprUnit;
use crate::ast::expr::builtin::BuiltinCall;
use crate::ast::expr::ident::Identifier;
use crate::ast::note::NoteUnit;
use crate::ast::time::CycleTime;
use crate::mem::Multiple;
use crate::mem::linked::LinkedNode;

pub mod pattern;

impl ExprNode {
    pub fn new_builtin_call(builtin_call: BuiltinCall) -> Self {
        let name = format!("{}", builtin_call)
            .bytes()
            .collect::<Identifier>();

        Self(LinkedNode::new(Expr::Call {
            is_receiver: false,
            name,
            args: Multiple::new(),
        }))
    }

    pub fn new_builtin_note_unit(note_unit: NoteUnit) -> Self {
        let name = format!("{}", note_unit)
            .bytes()
            .collect::<Identifier>();

        Self(LinkedNode::new(Expr::Unit(ExprUnit::Ident(name))))
    }

    pub fn new_builtin_number(number: CycleTime) -> Self {
        Self(LinkedNode::new(Expr::Unit(ExprUnit::Number(number))))
    }

    pub fn new_builtin_silence() -> Self {
        Self(LinkedNode::new(Expr::Unit(ExprUnit::Ident(
            Identifier::from_iter(b"~".iter().cloned()),
        ))))
    }
}
