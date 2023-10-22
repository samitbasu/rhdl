use crate::ast;

// Recursively traverse the AST, and assign
// NodeIds to all of the nodes in the tree.

// NodeId generator for AST nodes.
pub struct NodeIdGenerator {
    id: u32,
}

impl NodeIdGenerator {
    pub fn new() -> Self {
        NodeIdGenerator { id: 0 }
    }

    pub fn next(&mut self) -> ast::NodeId {
        let id = self.id;
        self.id += 1;
        ast::NodeId::new(id)
    }
}

fn assign_node_ids(root: &mut Box<ast::Block>) {
    let mut generator = NodeIdGenerator::new();
    assign_node_ids_block(root, &mut generator);
}

fn assign_node_ids_block(block: &mut Box<ast::Block>, generator: &mut NodeIdGenerator) {
    block.id = Some(generator.next());
    for stmt in &mut block.stmts {
        assign_node_ids_stmt(stmt, generator);
    }
}

fn assign_node_ids_stmt(stmt: &mut Box<ast::Stmt>, generator: &mut NodeIdGenerator) {
    stmt.id = Some(generator.next());
    match &mut stmt.kind {
        ast::StmtKind::Local(local) => {
            assign_node_ids_local(local, generator);
        }
        ast::StmtKind::Expr(expr) => {
            assign_node_ids_expr(expr, generator);
        }
        ast::StmtKind::Semi(expr) => {
            assign_node_ids_expr(expr, generator);
        }
    }
}

fn assign_node_ids_local(local: &mut Box<ast::Local>, generator: &mut NodeIdGenerator) {
    local.id = Some(generator.next());
    assign_node_ids_pat(&mut local.pat, generator);
    if let Some(init) = &mut local.init {
        assign_node_ids_expr(init, generator);
    }
}

fn assign_node_ids_pat(pat: &mut Box<ast::Pat>, generator: &mut NodeIdGenerator) {
    pat.id = Some(generator.next());
    match &mut pat.kind {
        ast::PatKind::Ident { .. } => {}
        ast::PatKind::Struct { fields, .. } => {
            for field in fields {
                assign_node_ids_pat_field(field, generator);
            }
        }
        ast::PatKind::Tuple { elems, .. } => {
            for elem in elems {
                assign_node_ids_pat(elem, generator);
            }
        }
        ast::PatKind::Range { from, to, .. } => {
            assign_node_ids_expr(from, generator);
            assign_node_ids_expr(to, generator);
        }
        ast::PatKind::Lit { .. } => {}
        ast::PatKind::Wild => {}
    }
}

fn assign_node_ids_pat_field(field: &mut ast::PatField, generator: &mut NodeIdGenerator) {
    field.id = Some(generator.next());
    assign_node_ids_pat(&mut field.pat, generator);
}

fn assign_node_ids_expr(expr: &mut Box<ast::Expr>, generator: &mut NodeIdGenerator) {
    expr.id = Some(generator.next());
    match &mut expr.kind {
        ast::ExprKind::Path { .. } => {}
        ast::ExprKind::Lit { .. } => {}
        ast::ExprKind::Struct { fields, .. } => {
            for field in fields {
                assign_node_ids_expr_field(field, generator);
            }
        }
        ast::ExprKind::Tuple { elems, .. } => {
            for elem in elems {
                assign_node_ids_expr(elem, generator);
            }
        }
        ast::ExprKind::Unary { expr, .. } => {
            assign_node_ids_expr(expr, generator);
        }
        ast::ExprKind::Binary { lhs, rhs, .. } => {
            assign_node_ids_expr(lhs, generator);
            assign_node_ids_expr(rhs, generator);
        }
        ast::ExprKind::Index { expr, index, .. } => {
            assign_node_ids_expr(expr, generator);
            assign_node_ids_expr(index, generator);
        }
        ast::ExprKind::Field { expr, .. } => {
            assign_node_ids_expr(expr, generator);
        }
        ast::ExprKind::Range { from, to, .. } => {
            assign_node_ids_expr(from, generator);
            assign_node_ids_expr(to, generator);
        }
        ast::ExprKind::Cast { expr, .. } => {
            assign_node_ids_expr(expr, generator);
        }
        ast::ExprKind::Call { func, args, .. } => {
            assign_node_ids_expr(func, generator);
            for arg in args {
                assign_node_ids_expr(arg, generator);
            }
        }
        ast::ExprKind::MethodCall { receiver, args, .. } => {
            assign_node_ids_expr(receiver, generator);
            for arg in args {
                assign_node_ids_expr(arg, generator);
            }
        }
        ast::ExprKind::Closure { body, .. } => {
            assign_node_ids_block(body, generator);
        }
        ast::ExprKind::If { cond, then, els } => {
            assign_node_ids_expr(cond, generator);
            assign_node_ids_block(then, generator);
            if let Some(els) = els {
                assign_node_ids_expr(els, generator);
            }
        }
        ast::ExprKind::While { cond, body } =>