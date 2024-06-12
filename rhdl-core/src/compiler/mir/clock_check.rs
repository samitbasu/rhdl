fn try_select(&mut self, id: NodeId, op: &TypeSelect) -> Result<()> {
    if self
        .enforce_clocks(id, &[op.selector, op.true_value, op.false_value, op.lhs])
        .is_err()
    {
        return Err(Box::new(RHDLClockCoherenceViolation {
            src: self.mir.symbols.source.source.clone(),
            elements: vec![
                (
                    format!(
                        "Selector belongs to {} clock domain",
                        self.clock_domain_for_error(op.selector)
                    ),
                    self.mir.symbols.source.span(op.selector.id).into(),
                ),
                (
                    format!(
                        "True value belongs to {} clock domain",
                        self.clock_domain_for_error(op.true_value)
                    ),
                    self.mir.symbols.source.span(op.true_value.id).into(),
                ),
                (
                    format!(
                        "False value belongs to {} clock domain",
                        self.clock_domain_for_error(op.false_value)
                    ),
                    self.mir.symbols.source.span(op.false_value.id).into(),
                ),
            ],
            cause_description:
                "Select operation requires that all three be coherent to the same clock domain"
                    .to_string(),
            cause_span: self.mir.symbols.source.span(id).into(),
        })
        .into());
    }
    self.enforce_data_types_binary(id, op.lhs, op.true_value, op.false_value)?;
    Ok(())
}

fn enforce_clocks(&mut self, id: NodeId, t: &[TypeId]) -> Result<()> {
    let clocks = t
        .iter()
        .filter_map(|ty| self.ctx.project_signal_clock(*ty))
        .collect::<Vec<_>>();
    for (first, second) in clocks.iter().zip(clocks.iter().skip(1)) {
        self.unify(id, *first, *second)?;
    }
    Ok(())
}

fn try_binop(&mut self, id: NodeId, op: &TypeBinOp) -> Result<()> {
    let a1 = op.arg1;
    let a2 = op.arg2;
    if self.enforce_clocks(id, &[a1, a2, op.lhs]).is_err() {
        return Err(Box::new(RHDLClockCoherenceViolation {
            src: self.mir.symbols.source.source.clone(),
            elements: vec![
                (
                    format!(
                        "First argument belongs to {} clock domain",
                        self.clock_domain_for_error(a1)
                    ),
                    self.mir.symbols.source.span(a1.id).into(),
                ),
                (
                    format!(
                        "Second argument belongs to {} clock domain",
                        self.clock_domain_for_error(a2)
                    ),
                    self.mir.symbols.source.span(a2.id).into(),
                ),
            ],
            cause_description:
                "Binary operation requires both arguments belong to the same clock domain"
                    .to_string(),
            cause_span: self.mir.symbols.source.span(id).into(),
        })
        .into());
    }
    match &op.op {
        AluBinary::Add
        | AluBinary::Mul
        | AluBinary::BitAnd
        | AluBinary::BitOr
        | AluBinary::BitXor
        | AluBinary::Sub => {
            self.enforce_data_types_binary(id, op.lhs, op.arg1, op.arg2)?;
        }
        AluBinary::Eq
        | AluBinary::Lt
        | AluBinary::Le
        | AluBinary::Ne
        | AluBinary::Ge
        | AluBinary::Gt => {
            if let Some(arg1_clock) = self.ctx.project_signal_clock(op.arg1) {
                let lhs_var = self.ctx.ty_bool(id);
                let lhs_sig = self.ctx.ty_signal(id, lhs_var, arg1_clock);
                self.unify(id, op.lhs, lhs_sig)?;
            }
            if let Some(arg2_clock) = self.ctx.project_signal_clock(op.arg2) {
                let lhs_var = self.ctx.ty_bool(id);
                let lhs_sig = self.ctx.ty_signal(id, lhs_var, arg2_clock);
                self.unify(id, op.lhs, lhs_sig)?;
            }
            if !self.ctx.is_signal(op.arg1) && !self.ctx.is_signal(op.arg2) {
                let lhs_var = self.ctx.ty_bool(id);
                self.unify(id, op.lhs, lhs_var)?;
            }
            if let (Some(arg1_data), Some(arg2_data)) = (
                self.ctx.project_signal_value(op.arg1),
                self.ctx.project_signal_value(op.arg2),
            ) {
                self.unify(id, arg1_data, arg2_data)?;
            }
        }
        AluBinary::Shl | AluBinary::Shr => {
            if let Some(arg2) = self.ctx.project_signal_value(a2) {
                eprintln!("Project signal value flag for {}", self.ctx.desc(a2));
                if let Some(flag) = self.ctx.project_sign_flag(arg2) {
                    eprintln!("Project sign flag for {}", self.ctx.desc(a2));
                    let unsigned_flag = self.ctx.ty_sign_flag(id, SignFlag::Unsigned);
                    self.unify(id, flag, unsigned_flag)?;
                }
            }
            if let (Some(lhs_data), Some(arg1_data)) = (
                self.ctx.project_signal_value(op.lhs),
                self.ctx.project_signal_value(op.arg1),
            ) {
                self.unify(id, lhs_data, arg1_data)?;
            } else {
                self.unify(id, op.lhs, op.arg1)?;
            }
        }
    }
    Ok(())
}

fn try_index(&mut self, id: NodeId, op: &TypeIndex) -> Result<()> {
    eprintln!(
        "Try to apply index to {} with path {:?}",
        self.ctx.desc(op.arg),
        op.path
    );
    let mut all_slots = vec![op.lhs, op.arg];
    all_slots.extend(op.path.dynamic_slots().map(|slot| self.slot_ty(*slot)));
    if self.enforce_clocks(id, &all_slots).is_err() {
        let arg_domain = self.clock_domain_for_error(op.arg);
        let arg_span = self.mir.symbols.source.span(op.arg.id);
        return Err(Box::new(RHDLClockCoherenceViolation {
            src: self.mir.symbols.source.source.clone(),
            elements: op
                .path
                .dynamic_slots()
                .map(|slot| {
                    let ty = self.slot_ty(*slot);
                    (
                        format!(
                            "Index belongs to {} clock domain",
                            self.clock_domain_for_error(ty)
                        ),
                        self.mir.symbols.source.span(ty.id).into(),
                    )
                })
                .chain(std::iter::once((
                    format!("Object being indexed belongs to {arg_domain} clock domain",),
                    arg_span.into(),
                )))
                .collect(),
            cause_description:
                "Index operation requires all slots to be coherent with the object being indexed"
                    .to_string(),
            cause_span: self.mir.symbols.source.span(id).into(),
        })
        .into());
    }
    match self.ty_path_project(op.arg, &op.path, id) {
        Ok(ty) => self.unify(id, op.lhs, ty),
        Err(err) => {
            eprintln!("Error: {}", err);
            Ok(())
        }
    }
}
