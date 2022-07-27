use acvm::FieldElement;
use noirc_frontend::util::vecmap;

use crate::errors::RuntimeError;

use super::{
    block::BlockId,
    context::SsaContext,
    node::{
        Binary, BinaryOp, Instruction, Mark, Node, NodeEval, NodeId, ObjectType, Opcode, Operation,
    },
};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
};

// Performs constant folding, arithmetic simplifications and move to standard form
// Modifies ins.mark with whether the instruction should be deleted, replaced, or neither
pub fn simplify(ctx: &mut SsaContext, ins: &mut Instruction) -> Result<(), RuntimeError> {
    if ins.is_deleted() {
        return Ok(());
    }
    //1. constant folding
    let new_id = ins.evaluate(ctx)?.to_index(ctx);

    if new_id != ins.id {
        use Mark::*;
        ins.mark = if new_id == NodeId::dummy() { Deleted } else { ReplaceWith(new_id) };
        return Ok(());
    }

    //2. standard form
    ins.standard_form();
    if let Operation::Cast(value_id) = ins.operation {
        if let Some(value) = ctx.try_get_node(value_id) {
            if value.get_type() == ins.res_type {
                ins.mark = Mark::ReplaceWith(value_id);
                return Ok(());
            }
        }
    }

    //3. left-overs (it requires &mut ctx)
    if ins.is_deleted() {
        return Ok(());
    }

    if let Operation::Binary(binary) = &mut ins.operation {
        if let NodeEval::ConstField(r_const, r_type) = NodeEval::from_id(ctx, binary.rhs) {
            if binary.operator == BinaryOp::Div {
                binary.rhs = ctx.get_or_create_const(r_const.inverse(), r_type);
                binary.operator = BinaryOp::Mul;
            }
        }
    }

    if let Operation::Intrinsic(opcode, args) = &ins.operation {
        let args = args
            .iter()
            .map(|arg| NodeEval::from_id(ctx, *arg).into_const_value().map(|f| f.to_u128()));

        if let Some(args) = args.collect() {
            ins.mark = Mark::ReplaceWith(evaluate_intrinsic(ctx, *opcode, args));
        }
    }

    Ok(())
}

fn evaluate_intrinsic(ctx: &mut SsaContext, op: acvm::acir::OPCODE, args: Vec<u128>) -> NodeId {
    match op {
        acvm::acir::OPCODE::ToBits => {
            let bit_count = args[1] as u32;
            let bits = vecmap(0..bit_count, |i| {
                if args[0] & (1 << i) != 0 {
                    FieldElement::one()
                } else {
                    FieldElement::zero()
                }
            });

            ctx.new_array(ObjectType::Unsigned(1), bits)
        }
        other => todo!("Unimplemented opcode: {:?}", other),
    }
}
////////////////////CSE////////////////////////////////////////

pub fn find_similar_instruction(
    igen: &SsaContext,
    operation: &Operation,
    prev_ins: &VecDeque<NodeId>,
) -> Option<NodeId> {
    for iter in prev_ins {
        if let Some(ins) = igen.try_get_instruction(*iter) {
            if &ins.operation == operation && !ins.is_deleted() {
                return Some(*iter);
            }
        }
    }
    None
}

pub fn find_similar_cast(
    igen: &SsaContext,
    operator: &Operation,
    res_type: ObjectType,
    prev_ins: &VecDeque<NodeId>,
) -> Option<NodeId> {
    for iter in prev_ins {
        if let Some(ins) = igen.try_get_instruction(*iter) {
            if &ins.operation == operator && ins.res_type == res_type && !ins.is_deleted() {
                return Some(*iter);
            }
        }
    }
    None
}

#[derive(Debug)]
pub enum CseAction {
    ReplaceWith(NodeId),
    Remove(NodeId),
    Keep,
}

fn find_similar_mem_instruction(
    ctx: &SsaContext,
    op: &Operation,
    prev_ins: &VecDeque<NodeId>,
) -> CseAction {
    match op {
        Operation::Load { index, .. } => {
            for iter in prev_ins.iter() {
                if let Some(ins_iter) = ctx.try_get_instruction(*iter) {
                    match &ins_iter.operation {
                        Operation::Load { index: index2, .. } => {
                            if !ctx.maybe_distinct(*index2, *index) {
                                return CseAction::ReplaceWith(*iter);
                            }
                        }
                        Operation::Store { index: index2, value, .. } => {
                            if !ctx.maybe_distinct(*index2, *index) {
                                return CseAction::ReplaceWith(*value);
                            }
                            if ctx.maybe_equal(*index2, *index) {
                                return CseAction::Keep;
                            }
                        }
                        _ => unreachable!("invalid operator in the memory anchor list"),
                    }
                }
            }
        }
        Operation::Store { index, .. } => {
            for node_id in prev_ins.iter() {
                if let Some(ins_iter) = ctx.try_get_instruction(*node_id) {
                    match ins_iter.operation {
                        Operation::Load { index: index2, .. } => {
                            if ctx.maybe_equal(index2, *index) {
                                return CseAction::Keep;
                            }
                        }
                        Operation::Store { index: index2, .. } => {
                            if !ctx.maybe_distinct(index2, *index) {
                                return CseAction::Remove(*node_id);
                            }
                            if ctx.maybe_equal(index2, *index) {
                                return CseAction::Keep;
                            }
                        }
                        _ => unreachable!("invalid operator in the memory anchor list"),
                    }
                }
            }
        }
        _ => unreachable!("invalid non memory operator"),
    }

    CseAction::Keep
}

pub fn propagate(ctx: &SsaContext, id: NodeId, modified: &mut bool) -> NodeId {
    if let Some(obj) = ctx.try_get_instruction(id) {
        if let Mark::ReplaceWith(replacement) = obj.mark {
            *modified = true;
            return replacement;
        } else if let Operation::Binary(Binary { operator: BinaryOp::Assign, rhs, .. }) =
            &obj.operation
        {
            *modified = true;
            return *rhs;
        }
    }
    id
}

//common subexpression elimination, starting from the root
pub fn cse(igen: &mut SsaContext, first_block: BlockId) -> Result<Option<NodeId>, RuntimeError> {
    let mut anchor = Anchor::default();
    let mut modified = false;
    cse_tree(igen, first_block, &mut anchor, &mut modified)
}

//Perform CSE for the provided block and then process its children following the dominator tree, passing around the anchor list.
fn cse_tree(
    igen: &mut SsaContext,
    block_id: BlockId,
    anchor: &mut Anchor,
    modified: &mut bool,
) -> Result<Option<NodeId>, RuntimeError> {
    let mut instructions = Vec::new();
    let mut res = cse_block_with_anchor(igen, block_id, &mut instructions, anchor, modified)?;
    for b in igen[block_id].dominated.clone() {
        let sub_res = cse_tree(igen, b, &mut anchor.clone(), modified)?;
        if sub_res.is_some() {
            res = sub_res;
        }
    }
    Ok(res)
}

//perform common subexpression elimination until there is no more change
pub fn full_cse(
    igen: &mut SsaContext,
    first_block: BlockId,
) -> Result<Option<NodeId>, RuntimeError> {
    let mut modified = true;
    let mut result = None;
    while modified {
        modified = false;
        let mut anchor = Anchor::default();
        result = cse_tree(igen, first_block, &mut anchor, &mut modified)?;
    }
    Ok(result)
}

/// A list of instructions with the same Operation variant, ordered by the order
/// they appear in their respective blocks.
#[derive(Default, Clone)]
struct Anchor {
    map: HashMap<Opcode, VecDeque<NodeId>>,
}

impl Anchor {
    fn push_front(&mut self, op: Opcode, id: NodeId) {
        self.map.entry(op).or_insert_with(VecDeque::new).push_front(id);
    }

    fn get_all(&self, opcode: Opcode) -> Cow<VecDeque<NodeId>> {
        match self.map.get(&opcode) {
            Some(vec) => Cow::Borrowed(vec),
            None => Cow::Owned(VecDeque::new()),
        }
    }
}

pub fn cse_block(
    ctx: &mut SsaContext,
    block_id: BlockId,
    instructions: &mut Vec<NodeId>,
    modified: &mut bool,
) -> Result<Option<NodeId>, RuntimeError> {
    cse_block_with_anchor(ctx, block_id, instructions, &mut Anchor::default(), modified)
}

//Performs common subexpression elimination and copy propagation on a block
fn cse_block_with_anchor(
    ctx: &mut SsaContext,
    block_id: BlockId,
    instructions: &mut Vec<NodeId>,
    anchor: &mut Anchor,
    modified: &mut bool,
) -> Result<Option<NodeId>, RuntimeError> {
    let mut new_list = Vec::new();
    let bb = &ctx[block_id];
    let is_join = bb.predecessor.len() > 1;
    if instructions.is_empty() {
        instructions.append(&mut bb.instructions.clone());
    }

    for ins_id in instructions {
        if let Some(ins) = ctx.try_get_instruction(*ins_id) {
            if ins.is_deleted() {
                continue;
            }

            let operator = ins.operation.map_id(|id| propagate(ctx, id, modified));
            let mut new_mark = Mark::None;

            let check_similar = |opcode| {
                let variants = anchor.get_all(opcode);
                if let Some(similar) = find_similar_instruction(ctx, &operator, &variants) {
                    debug_assert!(similar != ins.id);
                    *modified = true;
                    new_mark = Mark::ReplaceWith(similar);
                } else {
                    new_list.push(*ins_id);
                    anchor.push_front(ins.operation.opcode(), *ins_id);
                }
            };

            match &operator {
                Operation::Binary(binary) => {
                    let variants = anchor.get_all(binary.opcode());
                    if let Some(similar) = find_similar_instruction(ctx, &operator, &variants) {
                        debug_assert!(similar != ins.id);
                        *modified = true;
                        new_mark = Mark::ReplaceWith(similar);
                    } else if binary.operator == BinaryOp::Assign {
                        *modified = true;
                        new_mark = Mark::ReplaceWith(binary.rhs);
                    } else {
                        new_list.push(*ins_id);
                        anchor.push_front(ins.operation.opcode(), *ins_id);
                    }
                }
                Operation::Load { .. } => check_similar(Opcode::Load),
                Operation::Store { .. } => check_similar(Opcode::Store),
                Operation::Phi { block_args, .. } => {
                    // propagate phi arguments
                    if let Some(first) = Instruction::simplify_phi(ins.id, block_args) {
                        if first == ins.id {
                            new_list.push(*ins_id);
                        } else {
                            *modified = true;
                            new_mark = Mark::ReplaceWith(first);
                        }
                    } else {
                        new_mark = Mark::Deleted;
                    }
                }
                Operation::Cast(_) => {
                    //Similar cast must have same type
                    if let Some(similar) = find_similar_cast(
                        ctx,
                        &operator,
                        ins.res_type,
                        &anchor.get_all(Opcode::Cast),
                    ) {
                        new_mark = Mark::ReplaceWith(similar);
                        *modified = true;
                    } else {
                        new_list.push(*ins_id);
                        anchor.push_front(operator.opcode(), *ins_id);
                    }
                }
                Operation::Call(func, arguments) => {
                    new_list.push(*ins_id);
                }
                Operation::Return(..) => new_list.push(*ins_id),
                Operation::Intrinsic(_, args) => {
                    if let Some(similar) =
                        find_similar_instruction(ctx, &operator, &anchor.get_all(operator.opcode()))
                    {
                        *modified = true;
                        new_mark = Mark::ReplaceWith(similar);
                    } else {
                        new_list.push(*ins_id);
                        anchor.push_front(operator.opcode(), *ins_id);
                    }
                }
                _ => {
                    //TODO: checks we do not need to propagate res arguments
                    new_list.push(*ins_id);
                }
            }

            let update = ctx.get_mut_instruction(*ins_id);

            update.operation = operator;
            update.mark = new_mark;
            if new_mark == Mark::Deleted {
                update.operation = Operation::Nop;
            }

            let mut update2 = update.clone();
            simplify(ctx, &mut update2)?;
            let update3 = ctx.get_mut_instruction(*ins_id);
            *update3 = update2;
        }
    }

    let last = new_list.iter().copied().rev().find(|id| is_some(ctx, *id));
    ctx[block_id].instructions = new_list;
    Ok(last)
}

pub fn is_some(ctx: &SsaContext, id: NodeId) -> bool {
    if id == NodeId::dummy() {
        return false;
    }
    if let Some(ins) = ctx.try_get_instruction(id) {
        if ins.operation != Operation::Nop {
            return true;
        }
    } else if ctx.try_get_node(id).is_some() {
        return true;
    }
    false
}
