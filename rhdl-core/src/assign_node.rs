use crate::{
    ast,
    visit_mut::{self, VisitorMut},
};
use anyhow::Result;

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

impl VisitorMut for NodeIdGenerator {
    fn visit_mut_stmt(&mut self, node: &mut ast::Stmt) -> Result<()> {
        self.id(&mut node.id);
        visit_mut::visit_mut_stmt(self, node)
    }
    fn visit_mut_block(&mut self, node: &mut ast::Block) -> Result<()> {
        self.id(&mut node.id);
        visit_mut::visit_mut_block(self, node)
    }
    fn visit_mut_local(&mut self, node: &mut ast::Local) -> Result<()> {
        self.id(&mut node.id);
        visit_mut::visit_mut_local(self, node)
    }
    fn visit_mut_pat(&mut self, node: &mut ast::Pat) -> Result<()> {
        self.id(&mut node.id);
        visit_mut::visit_mut_pat(self, node)
    }
    fn visit_mut_expr(&mut self, node: &mut ast::Expr) -> Result<()> {
        self.id(&mut node.id);
        visit_mut::visit_mut_expr(self, node)
    }
}

pub fn assign_node_ids(root: &mut Box<ast::Block>) -> Result<()> {
    let mut generator = NodeIdGenerator::new();
    generator.visit_mut_block(root)
}
