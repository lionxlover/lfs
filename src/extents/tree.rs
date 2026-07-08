use std::io::Result;
use crate::transaction::transaction::TxContext;
use crate::btree::tree::BTree;
use bytemuck::{Pod, Zeroable};

pub const EXTENT_TREE_NODE_TYPE: u32 = 3;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ExtentTreeValue {
    pub physical_start: u64,
    pub length: u64,
}
unsafe impl Zeroable for ExtentTreeValue {}
unsafe impl Pod for ExtentTreeValue {}

pub struct ExtentTree {
    pub btree: BTree<u64, ExtentTreeValue>,
}

impl ExtentTree {
    pub fn new(root_block: u64) -> Self {
        Self {
            btree: BTree::new(root_block, EXTENT_TREE_NODE_TYPE),
        }
    }

    pub fn init_empty(ctx: &mut TxContext, root_block: u64) -> Result<()> {
        BTree::<u64, ExtentTreeValue>::init_empty(ctx, root_block, EXTENT_TREE_NODE_TYPE)
    }

    pub fn lookup(&self, ctx: &mut TxContext, logical_start: u64) -> Result<Option<ExtentTreeValue>> {
        // Range search for extents could be more optimal since extents cover a range of logical blocks.
        // For Phase 4, we assume exact match lookup for simplicity.
        self.btree.lookup(ctx, &logical_start)
    }

    pub fn insert<F>(&mut self, ctx: &mut TxContext, logical_start: u64, physical_start: u64, length: u64, allocate_block: F) -> Result<()>
    where
        F: FnMut(&mut TxContext) -> Result<u64>,
    {
        let value = ExtentTreeValue {
            physical_start,
            length,
        };
        self.btree.insert(ctx, logical_start, value, allocate_block)
    }
}
