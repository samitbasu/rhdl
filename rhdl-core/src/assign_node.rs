use crate::{
    ast,
    visit::{walk_block, Visitor},
};

// Recursively traverse the AST, and assign
// NodeIds to all of the nodes in the tree.

// NodeId generator for AST nodes.
pub struct NodeIdGenerator {
    id: u32,
}

impl NodeIdGenerator {
    fn new() -> Self {
        NodeIdGenerator { id: 0 }
    }

    fn next(&mut self) -> ast::NodeId {
        let id = self.id;
        self.id += 1;
        ast::NodeId::new(id)
    }

    fn id(&mut self, id: &mut Option<ast::NodeId>) {
        if id.is_none() {
            *id = Some(self.next());
        }
    }
}

impl Visitor for NodeIdGenerator {
    fn visit_stmt(&mut self, stmt: &mut ast::Stmt) {
        self.id(&mut stmt.id);
    }
    fn visit_block(&mut self, block: &mut ast::Block) {
        self.id(&mut block.id);
    }
    fn visit_local(&mut self, local: &mut ast::Local) {
        self.id(&mut local.id);
    }
    fn visit_pat(&mut self, pat: &mut ast::Pat) {
        self.id(&mut pat.id);
    }
    fn visit_path(&mut self, path: &mut ast::Path) {
        self.id(&mut path.id);
    }
    fn visit_expr(&mut self, expr: &mut ast::Expr) {
        self.id(&mut expr.id);
    }
}

pub fn assign_node_ids(root: &mut Box<ast::Block>) {
    let mut generator = NodeIdGenerator::new();
    walk_block(&mut generator, root);
}
