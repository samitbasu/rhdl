use std::collections::HashMap;

use crate::{
    rhif::object::SourceLocation,
    rtl::{
        object::LocatedOpCode,
        spec::{OpCode, Operand},
        Object,
    },
};

use super::{
    db::{PartsDB, PinIx},
    rtl_components::{
        Assign, Binary, Case, CaseArgument, Cast, ComponentKind, Concat, Constant, DynamicIndex,
        DynamicSplice, Index, Select, Splice, Unary,
    },
    rtl_schematic::Schematic,
};

use crate::rtl::spec as tl;

pub struct SchematicBuilder<'a> {
    object: &'a Object,
    db: &'a mut PartsDB,
    schematic: Schematic,
    operand_map: HashMap<Operand, PinIx>,
}

pub fn build_rtl_schematic(object: &Object, parts_db: &mut PartsDB) -> Schematic {
    let mut bob = SchematicBuilder::new(object, parts_db);
    object.ops.iter().for_each(|lop| bob.op(lop));
    bob.schematic
}

impl<'a> SchematicBuilder<'a> {
    fn new(object: &'a Object, parts_db: &'a mut PartsDB) -> Self {
        // Allocate input and output pins.
        let inputs = object
            .arguments
            .iter()
            .enumerate()
            .filter_map(|(ndx, x)| x.map(|reg| (ndx, reg)))
            .map(|(ndx, reg)| parts_db.new_pin(object.register_kind[&reg], &format!("a{ndx}")))
            .collect();
        let output_kind = object.kind(object.return_register);
        let output = parts_db.new_pin(output_kind, "y");
        Self {
            object,
            db: parts_db,
            schematic: Schematic {
                components: Default::default(),
                wires: Default::default(),
                inputs,
                output,
            },
            operand_map: Default::default(),
        }
    }
    fn op(&mut self, lop: &LocatedOpCode) {
        let loc = SourceLocation {
            func: lop.func,
            node: lop.id,
        };
        match &lop.op {
            OpCode::Assign(assign) => self.build_assign(loc, assign),
            OpCode::Binary(binary) => self.build_binary(loc, binary),
            OpCode::Case(case) => self.build_case(loc, case),
            OpCode::Cast(cast) => self.build_cast(loc, cast),
            OpCode::Comment(_) => {}
            OpCode::Concat(concat) => self.build_concat(loc, concat),
            OpCode::DynamicIndex(dynamic_index) => self.build_dynamic_index(loc, dynamic_index),
            OpCode::DynamicSplice(dynamic_splice) => self.build_dynamic_splice(loc, dynamic_splice),
            OpCode::Index(index) => self.build_index(loc, index),
            OpCode::Select(select) => self.build_select(loc, select),
            OpCode::Splice(splice) => self.build_splice(loc, splice),
            OpCode::Unary(unary) => self.build_unary(loc, unary),
        }
    }
    fn operand(&mut self, loc: SourceLocation, operand: Operand) -> PinIx {
        if let Some(pin) = self.operand_map.get(&operand) {
            return *pin;
        }
        match operand {
            Operand::Literal(literal_id) => {
                let bs = &self.object.literals[&literal_id];
                let lhs = self.db.new_pin(bs.into(), &format!("{literal_id:?}"));
                let constant_component = self.db.new_component(
                    ComponentKind::Constant(Constant {
                        bs: bs.clone(),
                        lhs,
                    }),
                    loc,
                );
                self.schematic.components.push(constant_component);
                lhs
            }
            Operand::Register(register_id) => {
                let reg = self.object.register_kind[&register_id];
                self.db.new_pin(reg, &format!("{register_id:?}"))
            }
        }
    }
    fn component(&mut self, component: ComponentKind, location: SourceLocation) {
        let component = self.db.new_component(component, location);
        self.schematic.components.push(component);
    }
    fn build_assign(&mut self, loc: SourceLocation, assign: &tl::Assign) {
        let rhs = self.operand(loc, assign.rhs);
        let lhs = self.operand(loc, assign.lhs);
        self.component(ComponentKind::Assign(Assign { lhs, rhs }), loc);
    }
    fn build_binary(&mut self, loc: SourceLocation, binary: &tl::Binary) {
        let arg1 = self.operand(loc, binary.arg1);
        let arg2 = self.operand(loc, binary.arg2);
        let lhs = self.operand(loc, binary.lhs);
        self.component(
            ComponentKind::Binary(Binary {
                op: binary.op,
                lhs,
                arg1,
                arg2,
            }),
            loc,
        );
    }
    fn build_case(&mut self, loc: SourceLocation, case: &tl::Case) {
        let lhs = self.operand(loc, case.lhs);
        let discriminant = self.operand(loc, case.discriminant);
        let table = case
            .table
            .iter()
            .map(|(argument, operand)| {
                let case = match argument {
                    tl::CaseArgument::Literal(lit_id) => {
                        let bs = &self.object.literals[lit_id];
                        CaseArgument::Literal(bs.clone())
                    }
                    tl::CaseArgument::Wild => CaseArgument::Wild,
                };
                let pin = self.operand(loc, *operand);
                (case, pin)
            })
            .collect();
        self.component(
            ComponentKind::Case(Case {
                lhs,
                discriminant,
                table,
            }),
            loc,
        );
    }
    fn build_cast(&mut self, loc: SourceLocation, cast: &tl::Cast) {
        let lhs = self.operand(loc, cast.lhs);
        let arg = self.operand(loc, cast.arg);
        self.component(
            ComponentKind::Cast(Cast {
                lhs,
                arg,
                len: cast.len,
                signed: cast.signed,
            }),
            loc,
        );
    }
    fn build_concat(&mut self, loc: SourceLocation, concat: &tl::Concat) {
        let lhs = self.operand(loc, concat.lhs);
        let args = concat.args.iter().map(|x| self.operand(loc, *x)).collect();
        self.component(ComponentKind::Concat(Concat { lhs, args }), loc);
    }
    fn build_dynamic_index(&mut self, loc: SourceLocation, dynamic_index: &tl::DynamicIndex) {
        let lhs = self.operand(loc, dynamic_index.lhs);
        let arg = self.operand(loc, dynamic_index.arg);
        let offset = self.operand(loc, dynamic_index.offset);
        self.component(
            ComponentKind::DynamicIndex(DynamicIndex {
                lhs,
                arg,
                offset,
                len: dynamic_index.len,
            }),
            loc,
        );
    }
    fn build_dynamic_splice(&mut self, loc: SourceLocation, dynamic_splice: &tl::DynamicSplice) {
        let lhs = self.operand(loc, dynamic_splice.lhs);
        let arg = self.operand(loc, dynamic_splice.arg);
        let offset = self.operand(loc, dynamic_splice.offset);
        let value = self.operand(loc, dynamic_splice.value);
        self.component(
            ComponentKind::DynamicSplice(DynamicSplice {
                lhs,
                arg,
                offset,
                value,
                len: dynamic_splice.len,
            }),
            loc,
        );
    }
    fn build_index(&mut self, loc: SourceLocation, index: &tl::Index) {
        let lhs = self.operand(loc, index.lhs);
        let arg = self.operand(loc, index.arg);
        self.component(
            ComponentKind::Index(Index {
                lhs,
                arg,
                bit_range: index.bit_range.clone(),
            }),
            loc,
        );
    }
    fn build_select(&mut self, loc: SourceLocation, select: &tl::Select) {
        let lhs = self.operand(loc, select.lhs);
        let cond = self.operand(loc, select.cond);
        let true_value = self.operand(loc, select.true_value);
        let false_value = self.operand(loc, select.false_value);
        self.component(
            ComponentKind::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }),
            loc,
        );
    }
    fn build_splice(&mut self, loc: SourceLocation, splice: &tl::Splice) {
        let lhs = self.operand(loc, splice.lhs);
        let orig = self.operand(loc, splice.orig);
        let value = self.operand(loc, splice.value);
        self.component(
            ComponentKind::Splice(Splice {
                lhs,
                orig,
                bit_range: splice.bit_range.clone(),
                value,
            }),
            loc,
        );
    }
    fn build_unary(&mut self, loc: SourceLocation, unary: &tl::Unary) {
        let lhs = self.operand(loc, unary.lhs);
        let arg1 = self.operand(loc, unary.arg1);
        self.component(
            ComponentKind::Unary(Unary {
                op: unary.op,
                lhs,
                arg1,
            }),
            loc,
        );
    }
}
