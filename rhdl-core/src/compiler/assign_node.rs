use crate::{
    ast::{
        ast_impl,
        visit_mut::{self, VisitorMut},
    },
    error::RHDLError,
    kernel::Kernel,
};

// Recursively traverse the AST, and assign
// NodeIds to all of the nodes in the tree.

// NodeId generator for AST nodes.
#[derive(Default)]
pub struct NodeIdGenerator {
    id: u32,
}

impl NodeIdGenerator {
    fn next(&mut self) -> ast_impl::NodeId {
        let id = self.id;
        self.id += 1;
        ast_impl::NodeId::new(id)
    }

    fn id(&mut self, id: &mut ast_impl::NodeId) {
        if id.is_invalid() {
            *id = self.next();
        }
    }
}

type Result<T> = std::result::Result<T, RHDLError>;

impl VisitorMut for NodeIdGenerator {
    fn visit_mut_stmt(&mut self, node: &mut ast_impl::Stmt) -> Result<()> {
        self.id(&mut node.id);
        visit_mut::visit_mut_stmt(self, node)
    }
    fn visit_mut_block(&mut self, node: &mut ast_impl::Block) -> Result<()> {
        self.id(&mut node.id);
        visit_mut::visit_mut_block(self, node)
    }
    fn visit_mut_local(&mut self, node: &mut ast_impl::Local) -> Result<()> {
        self.id(&mut node.id);
        visit_mut::visit_mut_local(self, node)
    }
    fn visit_mut_pat(&mut self, node: &mut ast_impl::Pat) -> Result<()> {
        self.id(&mut node.id);
        visit_mut::visit_mut_pat(self, node)
    }
    fn visit_mut_expr(&mut self, node: &mut ast_impl::Expr) -> Result<()> {
        self.id(&mut node.id);
        visit_mut::visit_mut_expr(self, node)
    }
    fn visit_mut_kernel_fn(&mut self, node: &mut ast_impl::KernelFn) -> Result<()> {
        self.id(&mut node.id);
        visit_mut::visit_mut_kernel_fn(self, node)
    }
    fn visit_mut_match_arm(&mut self, node: &mut ast_impl::Arm) -> Result<()> {
        self.id(&mut node.id);
        visit_mut::visit_mut_match_arm(self, node)
    }
}

pub fn assign_node_ids(root: &mut Kernel) -> Result<()> {
    let mut generator = NodeIdGenerator::default();
    generator.visit_mut_kernel_fn(root.inner_mut())
}
