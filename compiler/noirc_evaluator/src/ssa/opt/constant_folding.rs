//! The goal of the constant folding optimization pass is to propagate any constants forwards into
//! later [`Instruction`]s to maximize the impact of [compile-time simplifications][Instruction::simplify()].
//!
//! The pass works as follows:
//! - Re-insert each instruction in order to apply the instruction simplification performed
//!   by the [`DataFlowGraph`] automatically as new instructions are pushed.
//! - Check whether any input values have been constrained to be equal to a value of a simpler form
//!   by a [constrain instruction][Instruction::Constrain]. If so, replace the input value with the simpler form.
//! - Check whether the instruction [can_be_replaced][Instruction::can_be_replaced()]
//!   by duplicate instruction earlier in the same block.
//!
//! These operations are done in parallel so that they can each benefit from each other
//! without the need for multiple passes.
//!
//! Other passes perform a certain amount of constant folding automatically as they insert instructions
//! into the [`DataFlowGraph`] but this pass can become needed if [`DataFlowGraph::set_value`] or
//! [`DataFlowGraph::set_value_from_id`] are used on a value which enables instructions dependent on the value to
//! now be simplified.
//!
//! This is the only pass which removes duplicated pure [`Instruction`]s however and so is needed when
//! different blocks are merged, i.e. after the [`flatten_cfg`][super::flatten_cfg] pass.
use std::collections::HashSet;

use acvm::{acir::AcirField, FieldElement};
use iter_extended::vecmap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::{DataFlowGraph, InsertInstructionResult},
        function::Function,
        instruction::{Instruction, InstructionId},
        types::Type,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};
use fxhash::FxHashMap as HashMap;

impl Ssa {
    /// Performs constant folding on each instruction.
    ///
    /// See [`constant_folding`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn fold_constants(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            constant_fold(function, false);
        }
        self
    }

    /// Performs constant folding on each instruction.
    ///
    /// Also uses constraint information to inform more optimizations.
    ///
    /// See [`constant_folding`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn fold_constants_using_constraints(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            constant_fold(function, true);
        }
        self
    }
}

/// The structure of this pass is simple:
/// Go through each block and re-insert all instructions.
fn constant_fold(function: &mut Function, use_constraint_info: bool) {
    let mut context = Context { use_constraint_info, ..Default::default() };
    context.block_queue.push(function.entry_block());

    while let Some(block) = context.block_queue.pop() {
        if context.visited_blocks.contains(&block) {
            continue;
        }

        context.visited_blocks.insert(block);
        context.fold_constants_in_block(function, block);
    }
}

#[derive(Default)]
struct Context {
    use_constraint_info: bool,
    /// Maps pre-folded ValueIds to the new ValueIds obtained by re-inserting the instruction.
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: Vec<BasicBlockId>,
}

/// HashMap from (Instruction, side_effects_enabled_var) to the results of the instruction.
/// Stored as a two-level map to avoid cloning Instructions during the `.get` call.
type InstructionResultCache = HashMap<Instruction, HashMap<Option<ValueId>, Vec<ValueId>>>;

impl Context {
    fn fold_constants_in_block(&mut self, function: &mut Function, block: BasicBlockId) {
        let instructions = function.dfg[block].take_instructions();

        // Cache of instructions without any side-effects along with their outputs.
        let mut cached_instruction_results = HashMap::default();

        // Contains sets of values which are constrained to be equivalent to each other.
        //
        // The mapping's structure is `side_effects_enabled_var => (constrained_value => simplified_value)`.
        //
        // We partition the maps of constrained values according to the side-effects flag at the point
        // at which the values are constrained. This prevents constraints which are only sometimes enforced
        // being used to modify the rest of the program.
        let mut constraint_simplification_mappings: HashMap<ValueId, HashMap<ValueId, ValueId>> =
            HashMap::default();
        let mut side_effects_enabled_var =
            function.dfg.make_constant(FieldElement::one(), Type::bool());

        for instruction_id in instructions {
            self.fold_constants_into_instruction(
                &mut function.dfg,
                block,
                instruction_id,
                &mut cached_instruction_results,
                &mut constraint_simplification_mappings,
                &mut side_effects_enabled_var,
            );
        }
        self.block_queue.extend(function.dfg[block].successors());
    }

    fn fold_constants_into_instruction(
        &self,
        dfg: &mut DataFlowGraph,
        block: BasicBlockId,
        id: InstructionId,
        instruction_result_cache: &mut InstructionResultCache,
        constraint_simplification_mappings: &mut HashMap<ValueId, HashMap<ValueId, ValueId>>,
        side_effects_enabled_var: &mut ValueId,
    ) {
        let constraint_simplification_mapping =
            constraint_simplification_mappings.entry(*side_effects_enabled_var).or_default();
        let instruction = Self::resolve_instruction(id, dfg, constraint_simplification_mapping);
        let old_results = dfg.instruction_results(id).to_vec();

        // If a copy of this instruction exists earlier in the block, then reuse the previous results.
        if let Some(cached_results) =
            Self::get_cached(dfg, instruction_result_cache, &instruction, *side_effects_enabled_var)
        {
            Self::replace_result_ids(dfg, &old_results, cached_results);
            return;
        }

        // Otherwise, try inserting the instruction again to apply any optimizations using the newly resolved inputs.
        let new_results = Self::push_instruction(id, instruction.clone(), &old_results, block, dfg);

        Self::replace_result_ids(dfg, &old_results, &new_results);

        self.cache_instruction(
            instruction.clone(),
            new_results,
            dfg,
            instruction_result_cache,
            constraint_simplification_mapping,
            *side_effects_enabled_var,
        );

        // If we just inserted an `Instruction::EnableSideEffects`, we need to update `side_effects_enabled_var`
        // so that we use the correct set of constrained values in future.
        if let Instruction::EnableSideEffects { condition } = instruction {
            *side_effects_enabled_var = condition;
        };
    }

    /// Fetches an [`Instruction`] by its [`InstructionId`] and fully resolves its inputs.
    fn resolve_instruction(
        instruction_id: InstructionId,
        dfg: &DataFlowGraph,
        constraint_simplification_mapping: &HashMap<ValueId, ValueId>,
    ) -> Instruction {
        let instruction = dfg[instruction_id].clone();

        // Alternate between resolving `value_id` in the `dfg` and checking to see if the resolved value
        // has been constrained to be equal to some simpler value in the current block.
        //
        // This allows us to reach a stable final `ValueId` for each instruction input as we add more
        // constraints to the cache.
        fn resolve_cache(
            dfg: &DataFlowGraph,
            cache: &HashMap<ValueId, ValueId>,
            value_id: ValueId,
        ) -> ValueId {
            let resolved_id = dfg.resolve(value_id);
            match cache.get(&resolved_id) {
                Some(cached_value) => resolve_cache(dfg, cache, *cached_value),
                None => resolved_id,
            }
        }

        // Resolve any inputs to ensure that we're comparing like-for-like instructions.
        instruction
            .map_values(|value_id| resolve_cache(dfg, constraint_simplification_mapping, value_id))
    }

    /// Pushes a new [`Instruction`] into the [`DataFlowGraph`] which applies any optimizations
    /// based on newly resolved values for its inputs.
    ///
    /// This may result in the [`Instruction`] being optimized away or replaced with a constant value.
    fn push_instruction(
        id: InstructionId,
        instruction: Instruction,
        old_results: &[ValueId],
        block: BasicBlockId,
        dfg: &mut DataFlowGraph,
    ) -> Vec<ValueId> {
        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(old_results, |result| dfg.type_of_value(*result)));

        let call_stack = dfg.get_call_stack(id);
        let new_results =
            match dfg.insert_instruction_and_results(instruction, block, ctrl_typevars, call_stack)
            {
                InsertInstructionResult::SimplifiedTo(new_result) => vec![new_result],
                InsertInstructionResult::SimplifiedToMultiple(new_results) => new_results,
                InsertInstructionResult::Results(_, new_results) => new_results.to_vec(),
                InsertInstructionResult::InstructionRemoved => vec![],
            };
        // Optimizations while inserting the instruction should not change the number of results.
        assert_eq!(old_results.len(), new_results.len());

        new_results
    }

    fn cache_instruction(
        &self,
        instruction: Instruction,
        instruction_results: Vec<ValueId>,
        dfg: &DataFlowGraph,
        instruction_result_cache: &mut InstructionResultCache,
        constraint_simplification_mapping: &mut HashMap<ValueId, ValueId>,
        side_effects_enabled_var: ValueId,
    ) {
        if self.use_constraint_info {
            // If the instruction was a constraint, then create a link between the two `ValueId`s
            // to map from the more complex to the simpler value.
            if let Instruction::Constrain(lhs, rhs, _) = instruction {
                // These `ValueId`s should be fully resolved now.
                match (&dfg[lhs], &dfg[rhs]) {
                    // Ignore trivial constraints
                    (Value::NumericConstant { .. }, Value::NumericConstant { .. }) => (),

                    // Prefer replacing with constants where possible.
                    (Value::NumericConstant { .. }, _) => {
                        constraint_simplification_mapping.insert(rhs, lhs);
                    }
                    (_, Value::NumericConstant { .. }) => {
                        constraint_simplification_mapping.insert(lhs, rhs);
                    }
                    // Otherwise prefer block parameters over instruction results.
                    // This is as block parameters are more likely to be a single witness rather than a full expression.
                    (Value::Param { .. }, Value::Instruction { .. }) => {
                        constraint_simplification_mapping.insert(rhs, lhs);
                    }
                    (Value::Instruction { .. }, Value::Param { .. }) => {
                        constraint_simplification_mapping.insert(lhs, rhs);
                    }
                    (_, _) => (),
                }
            }
        }

        // If the instruction doesn't have side-effects and if it won't interact with enable_side_effects during acir_gen,
        // we cache the results so we can reuse them if the same instruction appears again later in the block.
        if instruction.can_be_deduplicated(dfg, self.use_constraint_info) {
            let use_predicate =
                self.use_constraint_info && instruction.requires_acir_gen_predicate(dfg);
            let predicate = use_predicate.then_some(side_effects_enabled_var);

            instruction_result_cache
                .entry(instruction)
                .or_default()
                .insert(predicate, instruction_results);
        }
    }

    /// Replaces a set of [`ValueId`]s inside the [`DataFlowGraph`] with another.
    fn replace_result_ids(
        dfg: &mut DataFlowGraph,
        old_results: &[ValueId],
        new_results: &[ValueId],
    ) {
        for (old_result, new_result) in old_results.iter().zip(new_results) {
            dfg.set_value_from_id(*old_result, *new_result);
        }
    }

    fn get_cached<'a>(
        dfg: &DataFlowGraph,
        instruction_result_cache: &'a mut InstructionResultCache,
        instruction: &Instruction,
        side_effects_enabled_var: ValueId,
    ) -> Option<&'a Vec<ValueId>> {
        let results_for_instruction = instruction_result_cache.get(instruction);

        // See if there's a cached version with no predicate first
        if let Some(results) = results_for_instruction.and_then(|map| map.get(&None)) {
            return Some(results);
        }

        let predicate =
            instruction.requires_acir_gen_predicate(dfg).then_some(side_effects_enabled_var);

        results_for_instruction.and_then(|map| map.get(&predicate))
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
            map::Id,
            types::Type,
            value::{Value, ValueId},
        },
    };
    use acvm::{acir::AcirField, FieldElement};

    #[test]
    fn simple_constant_fold() {
        // fn main f0 {
        //   b0(v0: Field):
        //     v1 = add v0, Field 1
        //     v2 = mul v1, Field 3
        //     return v2
        // }
        //
        // After constructing this IR, we set the value of v0 to 2.
        // The expected return afterwards should be 9.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::field());

        let one = builder.field_constant(1u128);
        let two = builder.field_constant(2u128);
        let three = builder.field_constant(3u128);

        let v1 = builder.insert_binary(v0, BinaryOp::Add, one);
        let v2 = builder.insert_binary(v1, BinaryOp::Mul, three);
        builder.terminate_with_return(vec![v2]);

        let mut ssa = builder.finish();
        let main = ssa.main_mut();
        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        // Expected output:
        //
        // fn main f0 {
        //   b0(Field 2: Field):
        //     return Field 9
        // }
        main.dfg.set_value_from_id(v0, two);

        let ssa = ssa.fold_constants();
        let main = ssa.main();
        let block = &main.dfg[main.entry_block()];
        assert_eq!(block.instructions().len(), 0);

        match block.terminator() {
            Some(TerminatorInstruction::Return { return_values, .. }) => {
                let value = main
                    .dfg
                    .get_numeric_constant(return_values[0])
                    .expect("Expected constant 9")
                    .to_u128();
                assert_eq!(value, 9);
            }
            _ => unreachable!("b0 should have a return terminator"),
        }
    }

    #[test]
    fn redundant_truncation() {
        // fn main f0 {
        //   b0(v0: u16, v1: u16):
        //     v2 = div v0, v1
        //     v3 = truncate v2 to 8 bits, max_bit_size: 16
        //     return v3
        // }
        //
        // After constructing this IR, we set the value of v1 to 2^8.
        // The expected return afterwards should be v2.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::unsigned(16));
        let v1 = builder.add_parameter(Type::unsigned(16));

        // Note that this constant guarantees that `v0/constant < 2^8`. We then do not need to truncate the result.
        let constant = 2_u128.pow(8);
        let constant = builder.numeric_constant(constant, Type::field());

        let v2 = builder.insert_binary(v0, BinaryOp::Div, v1);
        let v3 = builder.insert_truncate(v2, 8, 16);
        builder.terminate_with_return(vec![v3]);

        let mut ssa = builder.finish();
        let main = ssa.main_mut();
        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        // Expected output:
        //
        // fn main f0 {
        //   b0(Field 2: Field):
        //     return Field 9
        // }
        main.dfg.set_value_from_id(v1, constant);

        let ssa = ssa.fold_constants();
        let main = ssa.main();

        println!("{ssa}");

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 1);
        let instruction = &main.dfg[instructions[0]];

        assert_eq!(
            instruction,
            &Instruction::Binary(Binary { lhs: v0, operator: BinaryOp::Div, rhs: constant })
        );
    }

    #[test]
    fn non_redundant_truncation() {
        // fn main f0 {
        //   b0(v0: u16, v1: u16):
        //     v2 = div v0, v1
        //     v3 = truncate v2 to 8 bits, max_bit_size: 16
        //     return v3
        // }
        //
        // After constructing this IR, we set the value of v1 to 2^8 - 1.
        // This should not result in the truncation being removed.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::unsigned(16));
        let v1 = builder.add_parameter(Type::unsigned(16));

        // Note that this constant does not guarantee that `v0/constant < 2^8`. We must then truncate the result.
        let constant = 2_u128.pow(8) - 1;
        let constant = builder.numeric_constant(constant, Type::field());

        let v2 = builder.insert_binary(v0, BinaryOp::Div, v1);
        let v3 = builder.insert_truncate(v2, 8, 16);
        builder.terminate_with_return(vec![v3]);

        let mut ssa = builder.finish();
        let main = ssa.main_mut();
        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        // Expected output:
        //
        // fn main f0 {
        //   b0(v0: u16, Field 255: Field):
        //      v6 = div v0, Field 255
        //      v7 = truncate v6 to 8 bits, max_bit_size: 16
        //      return v7
        // }
        main.dfg.set_value_from_id(v1, constant);

        let ssa = ssa.fold_constants();
        let main = ssa.main();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 2);

        assert_eq!(
            &main.dfg[instructions[0]],
            &Instruction::Binary(Binary { lhs: v0, operator: BinaryOp::Div, rhs: constant })
        );
        assert_eq!(
            &main.dfg[instructions[1]],
            &Instruction::Truncate { value: ValueId::test_new(6), bit_size: 8, max_bit_size: 16 }
        );
    }

    #[test]
    fn arrays_elements_are_updated() {
        // fn main f0 {
        //   b0(v0: Field):
        //     v1 = add v0, Field 1
        //     return [v1]
        // }
        //
        // After constructing this IR, we run constant folding with no expected benefit, but to
        // ensure that all new values ids are correctly propagated.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::field());
        let one = builder.field_constant(1u128);
        let v1 = builder.insert_binary(v0, BinaryOp::Add, one);

        let array_type = Type::Array(Rc::new(vec![Type::field()]), 1);
        let arr = builder.current_function.dfg.make_array(vec![v1].into(), array_type);
        builder.terminate_with_return(vec![arr]);

        let ssa = builder.finish().fold_constants();
        let main = ssa.main();
        let entry_block_id = main.entry_block();
        let entry_block = &main.dfg[entry_block_id];
        assert_eq!(entry_block.instructions().len(), 1);
        let new_add_instr = entry_block.instructions().first().unwrap();
        let new_add_instr_result = main.dfg.instruction_results(*new_add_instr)[0];
        assert_ne!(new_add_instr_result, v1);

        let return_value_id = match entry_block.unwrap_terminator() {
            TerminatorInstruction::Return { return_values, .. } => return_values[0],
            _ => unreachable!("Should have terminator instruction"),
        };
        let return_element = match &main.dfg[return_value_id] {
            Value::Array { array, .. } => array[0],
            _ => unreachable!("Return type should be array"),
        };
        // The return element is expected to refer to the new add instruction result.
        assert_eq!(main.dfg.resolve(new_add_instr_result), main.dfg.resolve(return_element));
    }

    #[test]
    fn instruction_deduplication() {
        // fn main f0 {
        //   b0(v0: u16):
        //     v1 = cast v0 as u32
        //     v2 = cast v0 as u32
        //     constrain v1 v2
        // }
        //
        // After constructing this IR, we run constant folding which should replace the second cast
        // with a reference to the results to the first. This then allows us to optimize away
        // the constrain instruction as both inputs are known to be equal.
        //
        // The first cast instruction is retained and will be removed in the dead instruction elimination pass.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::unsigned(16));

        let v1 = builder.insert_cast(v0, Type::unsigned(32));
        let v2 = builder.insert_cast(v0, Type::unsigned(32));
        builder.insert_constrain(v1, v2, None);

        let mut ssa = builder.finish();
        let main = ssa.main_mut();
        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 3);

        // Expected output:
        //
        // fn main f0 {
        //   b0(v0: u16):
        //     v1 = cast v0 as u32
        // }
        let ssa = ssa.fold_constants();
        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();

        assert_eq!(instructions.len(), 1);
        let instruction = &main.dfg[instructions[0]];

        assert_eq!(instruction, &Instruction::Cast(v0, Type::unsigned(32)));
    }

    #[test]
    fn constant_index_array_access_deduplication() {
        // fn main f0 {
        //   b0(v0: [Field; 4], v1: u32, v2: bool, v3: bool):
        //     enable_side_effects v2
        //     v4 = array_get v0 u32 0
        //     v5 = array_get v0 v1
        //     enable_side_effects v3
        //     v6 = array_get v0 u32 0
        //     v7 = array_get v0 v1
        //     constrain v4 v6
        // }
        //
        // After constructing this IR, we run constant folding which should replace the second constant-index array get
        // with a reference to the results to the first. This then allows us to optimize away
        // the constrain instruction as both inputs are known to be equal.
        //
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.add_parameter(Type::Array(Rc::new(vec![Type::field()]), 4));
        let v1 = builder.add_parameter(Type::unsigned(32));
        let v2 = builder.add_parameter(Type::unsigned(1));
        let v3 = builder.add_parameter(Type::unsigned(1));

        let zero = builder.numeric_constant(FieldElement::zero(), Type::length_type());

        builder.insert_enable_side_effects_if(v2);
        let v4 = builder.insert_array_get(v0, zero, Type::field());
        let _v5 = builder.insert_array_get(v0, v1, Type::field());

        builder.insert_enable_side_effects_if(v3);
        let v6 = builder.insert_array_get(v0, zero, Type::field());
        let _v7 = builder.insert_array_get(v0, v1, Type::field());

        builder.insert_constrain(v4, v6, None);

        let ssa = builder.finish();

        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 7);

        // Expected output:
        //
        // fn main f0 {
        //   b0(v0: [Field; 4], v1: u32, v2: bool, v3: bool):
        //     enable_side_effects v2
        //     v10 = array_get v0 u32 0
        //     v11 = array_get v0 v1
        //     enable_side_effects v3
        //     v12 = array_get v0 v1
        // }
        let ssa = ssa.fold_constants();

        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();

        assert_eq!(instructions.len(), 5);
    }

    #[test]
    fn constraint_decomposition() {
        // fn main f0 {
        //   b0(v0: u1, v1: u1, v2: u1):
        //     v3 = mul v0 v1
        //     v4 = not v2
        //     v5 = mul v3 v4
        //     constrain v4 u1 1
        // }
        //
        // When constructing this IR, we should automatically decompose the constraint to be in terms of `v0`, `v1` and `v2`.
        //
        // The mul instructions are retained and will be removed in the dead instruction elimination pass.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::bool());
        let v1 = builder.add_parameter(Type::bool());
        let v2 = builder.add_parameter(Type::bool());

        let v3 = builder.insert_binary(v0, BinaryOp::Mul, v1);
        let v4 = builder.insert_not(v2);
        let v5 = builder.insert_binary(v3, BinaryOp::Mul, v4);

        // This constraint is automatically decomposed when it is inserted.
        let v_true = builder.numeric_constant(true, Type::bool());
        builder.insert_constrain(v5, v_true, None);

        let v_false = builder.numeric_constant(false, Type::bool());

        // Expected output:
        //
        // fn main f0 {
        //   b0(v0: u1, v1: u1, v2: u1):
        //     v3 = mul v0 v1
        //     v4 = not v2
        //     v5 = mul v3 v4
        //     constrain v0 u1 1
        //     constrain v1 u1 1
        //     constrain v2 u1 0
        // }

        let ssa = builder.finish();
        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();

        assert_eq!(instructions.len(), 6);

        assert_eq!(
            main.dfg[instructions[0]],
            Instruction::Binary(Binary { lhs: v0, operator: BinaryOp::Mul, rhs: v1 })
        );
        assert_eq!(main.dfg[instructions[1]], Instruction::Not(v2));
        assert_eq!(
            main.dfg[instructions[2]],
            Instruction::Binary(Binary { lhs: v3, operator: BinaryOp::Mul, rhs: v4 })
        );
        assert_eq!(main.dfg[instructions[3]], Instruction::Constrain(v0, v_true, None));
        assert_eq!(main.dfg[instructions[4]], Instruction::Constrain(v1, v_true, None));
        assert_eq!(main.dfg[instructions[5]], Instruction::Constrain(v2, v_false, None));
    }

    // Regression for #4600
    #[test]
    fn array_get_regression() {
        // fn main f0 {
        //   b0(v0: u1, v1: u64):
        //     enable_side_effects_if v0
        //     v2 = array_get [Field 0, Field 1], index v1
        //     v3 = not v0
        //     enable_side_effects_if v3
        //     v4 = array_get [Field 0, Field 1], index v1
        // }
        //
        // We want to make sure after constant folding both array_gets remain since they are
        // under different enable_side_effects_if contexts and thus one may be disabled while
        // the other is not. If one is removed, it is possible e.g. v4 is replaced with v2 which
        // is disabled (only gets from index 0) and thus returns the wrong result.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::bool());
        let v1 = builder.add_parameter(Type::unsigned(64));

        builder.insert_enable_side_effects_if(v0);

        let zero = builder.field_constant(0u128);
        let one = builder.field_constant(1u128);

        let typ = Type::Array(Rc::new(vec![Type::field()]), 2);
        let array = builder.array_constant(vec![zero, one].into(), typ);

        let _v2 = builder.insert_array_get(array, v1, Type::field());
        let v3 = builder.insert_not(v0);

        builder.insert_enable_side_effects_if(v3);
        let _v4 = builder.insert_array_get(array, v1, Type::field());

        // Expected output is unchanged
        let ssa = builder.finish();
        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        let starting_instruction_count = instructions.len();
        assert_eq!(starting_instruction_count, 5);

        let ssa = ssa.fold_constants();
        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        let ending_instruction_count = instructions.len();
        assert_eq!(starting_instruction_count, ending_instruction_count);
    }

    #[test]
    fn deduplicate_instructions_with_predicates() {
        // fn main f0 {
        //   b0(v0: bool, v1: bool, v2: [u32; 2]):
        //     enable_side_effects v0
        //     v3 = array_get v2, index u32 0
        //     v4 = array_set v2, index u32 1, value: u32 2
        //     v5 = array_get v4, index u32 0
        //     constrain_eq v3, v5
        //     enable_side_effects v1
        //     v6 = array_get v2, index u32 0
        //     v7 = array_set v2, index u32 1, value: u32 2
        //     v8 = array_get v7, index u32 0
        //     constrain_eq v6, v8
        //     enable_side_effects v0
        //     v9 = array_get v2, index u32 0
        //     v10 = array_set v2, index u32 1, value: u32 2
        //     v11 = array_get v10, index u32 0
        //     constrain_eq v9, v11
        // }
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.add_parameter(Type::bool());
        let v1 = builder.add_parameter(Type::bool());
        let v2 = builder.add_parameter(Type::Array(Rc::new(vec![Type::field()]), 2));

        let zero = builder.numeric_constant(0u128, Type::length_type());
        let one = builder.numeric_constant(1u128, Type::length_type());
        let two = builder.numeric_constant(2u128, Type::length_type());

        builder.insert_enable_side_effects_if(v0);
        let v3 = builder.insert_array_get(v2, zero, Type::length_type());
        let v4 = builder.insert_array_set(v2, one, two);
        let v5 = builder.insert_array_get(v4, zero, Type::length_type());
        builder.insert_constrain(v3, v5, None);

        builder.insert_enable_side_effects_if(v1);
        let v6 = builder.insert_array_get(v2, zero, Type::length_type());
        let v7 = builder.insert_array_set(v2, one, two);
        let v8 = builder.insert_array_get(v7, zero, Type::length_type());
        builder.insert_constrain(v6, v8, None);

        // We expect all these instructions after the 'enable side effects' instruction to be removed.
        builder.insert_enable_side_effects_if(v0);
        let v9 = builder.insert_array_get(v2, zero, Type::length_type());
        let v10 = builder.insert_array_set(v2, one, two);
        let v11 = builder.insert_array_get(v10, zero, Type::length_type());
        builder.insert_constrain(v9, v11, None);

        let ssa = builder.finish();
        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 15);

        // Expected output:
        //
        // fn main f0 {
        //   b0(v0: bool, v1: bool, v2: [Field; 2]):
        //     enable_side_effects v0
        //     v3 = array_get v2, index Field 0
        //     v4 = array_set v2, index Field 1, value: Field 2
        //     v5 = array_get v4, index Field 0
        //     constrain_eq v3, v5
        //     enable_side_effects v1
        //     v7 = array_set v2, index Field 1, value: Field 2
        //     v8 = array_get v7, index Field 0
        //     constrain_eq v3, v8
        //     enable_side_effects v0
        // }
        let ssa = ssa.fold_constants_using_constraints();
        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 10);
    }

    // Regression for #5736
    #[test]
    fn keccak256_regression() {
        // fn main f0 {
        //   b0(v0: u8):
        //     v1 = cast v0 as Field
        //     v3 = add v1, Field 2⁵
        //     v4 = truncate v3 to 64 bits, max_bit_size: 254
        //     v5 = cast v3 as u64
        //     v10 = call keccakf1600([v5, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0])
        //     v11 = call keccakf1600([v5, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0])
        // }
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::unsigned(8));

        let v1 = builder.insert_cast(v0, Type::field());
        let thirty_two = builder.field_constant(32u128);
        let v3 = builder.insert_binary(v1, BinaryOp::Add, thirty_two);
        let v4 = builder.insert_truncate(v3, 64, 254);
        let v5 = builder.insert_cast(v4, Type::unsigned(64));

        let zero = builder.numeric_constant(0u128, Type::unsigned(64));
        let two_pow_sixty_three = builder.numeric_constant(FieldElement::from(2u128).pow(&FieldElement::from(63u128)), Type::unsigned(64));
        let typ = Type::Array(Rc::new(vec![Type::unsigned(64)]), 25);
        let array = builder.array_constant(vec![v5, zero, zero, zero, zero, zero, zero, zero, zero, zero,zero, zero, zero, zero, zero,zero, two_pow_sixty_three, zero, zero, zero,zero, zero, zero, zero, zero].into(), typ.clone());


        let keccakf1600 = builder.import_intrinsic("keccakf1600").expect("keccakf1600 intrinsic should exist");
        let _v10 = builder.insert_call(keccakf1600, vec![array], vec![typ.clone()]);
        let _v11 = builder.insert_call(keccakf1600, vec![array], vec![typ.clone()]);
      
        let ssa = builder.finish();

        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        let starting_instruction_count = instructions.len();
        assert_eq!(starting_instruction_count, 6);

        let ssa = ssa.fold_constants();

        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        let ending_instruction_count = instructions.len();
        assert_eq!(ending_instruction_count, 5);
    }
}
