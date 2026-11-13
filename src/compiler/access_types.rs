//! Readonly qualifiers are static access restrictions with the underlying ABI.
use super::*;
use crate::frontend::ast::{AccessAnnotation, AccessMode};

pub(super) fn inner(ty: &str) -> Option<&str> {
    let (base, args) = tuples::type_parts(ty)?;
    (base == "__RuskReadonly" && args.len() == 1).then(|| args[0])
}
pub(super) fn underlying(mut ty: &str) -> &str {
    while let Some(base) = inner(ty) {
        ty = base;
    }
    ty
}
pub(super) fn same(a: &str, b: &str) -> bool {
    underlying(a) == underlying(b)
}
pub(super) fn info(span: &Span, name: &str) -> crate::ir::hir::AccessInfo {
    crate::ir::hir::AccessInfo {
        name: name.into(),
        declaration: span.clone(),
        annotation: AccessAnnotation {
            mode: AccessMode::Readonly,
            span: span.clone(),
        },
    }
}
impl Compiler {
    pub(super) fn declared_access_summary(&self, id: usize) -> crate::analysis::readonly::Summary {
        let f = &self.functions[id];
        let mut result = if inner(&f.result).is_some() {
            crate::analysis::readonly::Summary::readonly_result(info(&f.name.span, &f.name.text))
        } else {
            Default::default()
        };
        if f.body.is_none() {
            let contract = self.native_environment_contract(id).unwrap_or_else(|| {
                crate::analysis::effects::Summary::opaque(
                    f.params.len() + self.capture_parameters(id).len(),
                )
            });
            for (index, ty) in f.params.iter().enumerate() {
                if inner(ty).is_none() {
                    continue;
                }
                if contract.returned.unknown && contract.returned.may_share(index) {
                    result.restrict_result(info(&f.name.span, &f.name.text));
                } else {
                    for alias in &contract.returned.aliases {
                        if alias.parameter == index {
                            result.restrict_result_path(
                                alias.result_path.clone(),
                                info(&f.name.span, &f.name.text),
                            );
                        }
                    }
                }
                if contract.parameters[index].retained != crate::analysis::effects::Effect::No
                    && !contract.parameters[index].readonly_sealed
                {
                    result.retain_readonly(info(&f.name.span, &f.name.text));
                }
            }
        }
        result
    }
    pub(super) fn prepare_readonly_type(&mut self, name: &Name) -> Result<bool, String> {
        let Some(base) = inner(&name.text) else {
            return Ok(false);
        };
        self.check_type(&Name {
            text: base.into(),
            span: name.span.clone(),
            constant: None,
        })?;
        self.types
            .insert(name.text.clone(), self.types[base].clone());
        Ok(true)
    }
    pub(super) fn readonly_view(&mut self, value: Value, span: &Span) -> Value {
        let base = underlying(&value.ty).to_string();
        if base == value.ty {
            return value;
        }
        let result = self.emit(&base, Rvalue::Share(value));
        self.context.body.push(Instruction::AnnotateAccess {
            value: result.clone(),
            info: info(span, "readonly type"),
        });
        result
    }
    pub(super) fn qualify_value(&mut self, value: Value, ty: &str, span: &Span) -> Value {
        if inner(ty).is_none() || value.ty == ty {
            return value;
        }
        if !same(&value.ty, ty) {
            return value;
        }
        let result = self.emit(ty, Rvalue::Share(value));
        self.context.body.push(Instruction::AnnotateAccess {
            value: result.clone(),
            info: info(span, "readonly type"),
        });
        result
    }
}

// Keep qualifiers inside nominal arguments: Box<readonly Int> and Box<Int>
// remain distinct nominal specializations. Only root qualifiers lose their ABI.
pub(super) fn erase_type(ty: &mut String) {
    *ty = underlying(ty).to_string();
}
fn erase_value(value: &mut Value) {
    erase_type(&mut value.ty);
}
pub(super) fn erase_body(body: &mut Body) {
    for instruction in &mut body.instructions {
        match instruction {
            Instruction::DescribeBinding { value, .. }
            | Instruction::AnnotateAccess { value, .. }
            | Instruction::AnnotateBinding { value, .. } => erase_value(value),
            Instruction::Object { result, expression } => {
                erase_value(result);
                match expression {
                    Rvalue::Share(v)
                    | Rvalue::BoolCopy(v)
                    | Rvalue::Field { owner: v, .. }
                    | Rvalue::VariantTest { owner: v, .. }
                    | Rvalue::VariantField { owner: v, .. } => erase_value(v),
                    Rvalue::Aggregate(values) | Rvalue::Variant { fields: values, .. } => {
                        values.iter_mut().for_each(erase_value)
                    }
                }
            }
            Instruction::Literal { result, .. } => erase_value(result),
            Instruction::Call(call) => {
                erase_value(&mut call.result);
                call.args.iter_mut().for_each(erase_value);
            }
            Instruction::Declare { result, initial } => {
                erase_value(result);
                if let Some(v) = initial {
                    erase_value(v);
                }
            }
            Instruction::Rebind { target, value, .. } => {
                if let Place::Field { owner, .. } = target {
                    erase_value(owner);
                }
                erase_value(value);
            }
            Instruction::StoreBool { target, value } => {
                erase_value(target);
                erase_value(value);
            }
            Instruction::Scope(body) | Instruction::Loop { body, .. } => erase_body(body),
            Instruction::If { condition, yes, no } => {
                erase_value(condition);
                erase_body(yes);
                erase_body(no);
            }
            Instruction::Return { value: Some(v), .. } => erase_value(v),
            _ => {}
        }
    }
}
pub(super) fn erase_kind(kind: &mut crate::ir::hir::Type) {
    use crate::ir::hir::Type;
    match kind {
        Type::Buffer(t) | Type::Vector(t) | Type::Set(t) => erase_type(t),
        Type::Record(fields) | Type::Shared(fields) => {
            for (_, ty) in fields {
                erase_type(ty);
            }
        }
        Type::Enum(variants) => {
            for (_, fields) in variants {
                for ty in fields {
                    erase_type(ty);
                }
            }
        }
        Type::Callable {
            fields,
            params,
            result,
            ..
        } => {
            for (_, ty) in fields {
                erase_type(ty);
            }
            params.iter_mut().for_each(erase_type);
            erase_type(result);
        }
        Type::FunctionFamily { fields, .. } => {
            for (_, ty) in fields {
                erase_type(ty);
            }
        }
        Type::Dict { key, value } => {
            erase_type(key);
            erase_type(value);
        }
        Type::Iterator { element, retained } => {
            erase_type(element);
            for (_, ty) in retained {
                erase_type(ty);
            }
        }
        _ => {}
    }
}

pub(super) fn readonly_members(types: &BTreeMap<String, crate::ir::hir::Type>) -> BTreeSet<String> {
    fn walk(
        ty: &str,
        types: &BTreeMap<String, crate::ir::hir::Type>,
        seen: &mut BTreeSet<String>,
        depth: usize,
    ) -> bool {
        use crate::ir::hir::Type;
        if inner(ty).is_some() {
            return true;
        }
        if depth > 128 {
            return true;
        }
        if !seen.insert(ty.into()) {
            return false;
        }
        let children: Vec<&str> = match types.get(ty) {
            Some(Type::Vector(t) | Type::Buffer(t) | Type::Set(t)) => vec![t],
            Some(
                Type::Record(f)
                | Type::Shared(f)
                | Type::Callable { fields: f, .. }
                | Type::FunctionFamily { fields: f, .. },
            ) => f.iter().map(|(_, t)| t.as_str()).collect(),
            Some(Type::Enum(v)) => v
                .iter()
                .flat_map(|(_, f)| f.iter().map(String::as_str))
                .collect(),
            Some(Type::Dict { key, value }) => vec![key, value],
            Some(Type::Iterator { element, retained }) => std::iter::once(element.as_str())
                .chain(retained.iter().map(|(_, t)| t.as_str()))
                .collect(),
            _ => vec![],
        };
        children
            .into_iter()
            .any(|t| walk(t, types, seen, depth + 1))
    }
    types
        .keys()
        .filter(|ty| walk(ty, types, &mut BTreeSet::new(), 0))
        .cloned()
        .collect()
}
