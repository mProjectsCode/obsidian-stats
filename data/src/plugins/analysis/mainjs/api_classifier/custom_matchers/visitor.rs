use std::collections::{BTreeMap, BTreeSet};

use swc_ecma_ast::{
    AssignExpr, CallExpr, Callee, ClassMethod, Expr, FnDecl, MemberExpr, NewExpr, VarDeclarator,
};
use swc_ecma_visit::{Visit, VisitWith};

use super::super::symbol_index::{
    AliasInfo, BindingKey, SymbolCallProvenance, member_prop_name, static_string,
};
use super::SemanticIndex;
use super::ast::{
    assigned_ident, assigned_member, binding_ident, call_member_callee, create_element_tag,
    expr_contains_remote_url, expr_ident, ident_arg, is_append_child_call, is_metadata_cache_call,
    is_string_timer_call, prop_name, string_arg,
};

pub(super) fn collect(program: &swc_ecma_ast::Program, aliases: &AliasInfo) -> SemanticIndex {
    let mut visitor = SemanticVisitor::new(aliases);
    program.visit_with(&mut visitor);
    visitor.index
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ParameterEffect {
    AppendToDocument,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DynamicCallable {
    Eval,
    FunctionConstructor,
}

struct FunctionContext {
    function: BindingKey,
    parameters: BTreeMap<BindingKey, usize>,
}

struct SemanticVisitor<'a> {
    aliases: &'a AliasInfo,
    index: SemanticIndex,
    scripts: BTreeSet<BindingKey>,
    remote_scripts: BTreeSet<BindingKey>,
    remote_resources: BTreeMap<BindingKey, String>,
    remote_resource_bindings: BTreeSet<BindingKey>,
    inputs: BTreeSet<BindingKey>,
    caches: BTreeSet<BindingKey>,
    dynamic_callables: BTreeMap<BindingKey, DynamicCallable>,
    function_effects: BTreeMap<BindingKey, BTreeMap<usize, BTreeSet<ParameterEffect>>>,
    current_function: Option<FunctionContext>,
}

impl<'a> SemanticVisitor<'a> {
    fn new(aliases: &'a AliasInfo) -> Self {
        Self {
            aliases,
            index: SemanticIndex::default(),
            scripts: BTreeSet::new(),
            remote_scripts: BTreeSet::new(),
            remote_resources: BTreeMap::new(),
            remote_resource_bindings: BTreeSet::new(),
            inputs: BTreeSet::new(),
            caches: BTreeSet::new(),
            dynamic_callables: BTreeMap::new(),
            function_effects: BTreeMap::new(),
            current_function: None,
        }
    }

    fn binding(&self, ident: &swc_ecma_ast::Ident) -> BindingKey {
        self.aliases.binding_key(ident)
    }

    fn clear_binding(&mut self, binding: &BindingKey) {
        self.scripts.remove(binding);
        self.remote_scripts.remove(binding);
        self.remote_resources.remove(binding);
        self.remote_resource_bindings.remove(binding);
        self.inputs.remove(binding);
        self.caches.remove(binding);
        self.dynamic_callables.remove(binding);
    }

    fn assign_value(&mut self, binding: BindingKey, value: &Expr) {
        self.clear_binding(&binding);
        if let Some(tag) = create_element_tag(value) {
            match tag.as_str() {
                "script" => {
                    self.scripts.insert(binding);
                }
                "img" | "link" | "style" => {
                    self.remote_resources.insert(binding, tag);
                }
                "input" => {
                    self.inputs.insert(binding);
                }
                _ => {}
            }
        } else if is_metadata_cache_call(value) {
            self.caches.insert(binding);
        } else if let Some(callable) = self.dynamic_callable(value) {
            self.dynamic_callables.insert(binding, callable);
        } else if let Some(source) = expr_ident(value) {
            let source = self.binding(source);
            if self.scripts.contains(&source) {
                self.scripts.insert(binding.clone());
            }
            if self.remote_scripts.contains(&source) {
                self.remote_scripts.insert(binding.clone());
            }
            if let Some(tag) = self.remote_resources.get(&source).cloned() {
                self.remote_resources.insert(binding.clone(), tag);
            }
            if self.remote_resource_bindings.contains(&source) {
                self.remote_resource_bindings.insert(binding.clone());
            }
            if self.inputs.contains(&source) {
                self.inputs.insert(binding.clone());
            }
            if self.caches.contains(&source) {
                self.caches.insert(binding);
            }
        }
    }

    fn dynamic_callable(&self, expr: &Expr) -> Option<DynamicCallable> {
        match expr {
            Expr::Ident(ident) => {
                let binding = self.binding(ident);
                self.dynamic_callables.get(&binding).copied().or_else(|| {
                    let name = ident.sym.as_ref();
                    (self.aliases.call_provenance(name, ident.span) == SymbolCallProvenance::Global)
                        .then_some(match name {
                            "eval" => Some(DynamicCallable::Eval),
                            "Function" => Some(DynamicCallable::FunctionConstructor),
                            _ => None,
                        })
                        .flatten()
                })
            }
            Expr::Member(member) => {
                let property = member_prop_name(&member.prop)?;
                if property == "constructor" && self.is_function_object(&member.obj) {
                    return Some(DynamicCallable::FunctionConstructor);
                }
                if property == "eval"
                    && self
                        .aliases
                        .rooted_member_chain(member)
                        .is_some_and(|chain| {
                            matches!(
                                chain.as_str(),
                                "globalThis.eval" | "window.eval" | "self.eval"
                            )
                        })
                {
                    return Some(DynamicCallable::Eval);
                }
                None
            }
            Expr::Call(call) => {
                let member = call_member_callee(call)?;
                (member_prop_name(&member.prop).as_deref() == Some("bind"))
                    .then(|| self.dynamic_callable(&member.obj))
                    .flatten()
            }
            Expr::Paren(paren) => self.dynamic_callable(&paren.expr),
            Expr::Seq(sequence) => sequence
                .exprs
                .last()
                .and_then(|expr| self.dynamic_callable(expr)),
            _ => None,
        }
    }

    fn is_function_object(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Fn(_) | Expr::Arrow(_) | Expr::Class(_) => true,
            Expr::Ident(_) => {
                self.dynamic_callable(expr) == Some(DynamicCallable::FunctionConstructor)
            }
            Expr::Paren(paren) => self.is_function_object(&paren.expr),
            _ => false,
        }
    }

    fn invoked_dynamic_callable(&self, call: &CallExpr) -> Option<DynamicCallable> {
        let Callee::Expr(callee) = &call.callee else {
            return None;
        };
        self.dynamic_callable(callee).or_else(|| {
            let Expr::Member(member) = &**callee else {
                return None;
            };
            matches!(
                member_prop_name(&member.prop).as_deref(),
                Some("call" | "apply")
            )
            .then(|| self.dynamic_callable(&member.obj))
            .flatten()
        })
    }

    fn record_dynamic_call(&mut self, callable: DynamicCallable) {
        self.index.dynamic_code_sites.insert(
            match callable {
                DynamicCallable::Eval => "eval",
                DynamicCallable::FunctionConstructor => "function_constructor",
            }
            .to_string(),
        );
    }

    fn record_parameter_effect(&mut self, binding: &BindingKey, effect: ParameterEffect) {
        let Some(function) = &self.current_function else {
            return;
        };
        let Some(index) = function.parameters.get(binding).copied() else {
            return;
        };
        self.function_effects
            .entry(function.function.clone())
            .or_default()
            .entry(index)
            .or_default()
            .insert(effect);
    }

    fn apply_function_effects(&mut self, call: &CallExpr) {
        let Callee::Expr(callee) = &call.callee else {
            return;
        };
        let Expr::Ident(function) = &**callee else {
            return;
        };
        let function = self.binding(function);
        let Some(effects) = self.function_effects.get(&function).cloned() else {
            return;
        };
        for (index, effects) in effects {
            let Some(argument) = call.args.get(index) else {
                continue;
            };
            let Some(argument) = expr_ident(&argument.expr) else {
                continue;
            };
            let binding = self.binding(argument);
            if effects.contains(&ParameterEffect::AppendToDocument) {
                self.append_binding(&binding);
            }
        }
    }

    fn append_binding(&mut self, binding: &BindingKey) {
        if self.remote_scripts.contains(binding) {
            self.index.script_appends.insert(binding.clone());
        }
        if self.remote_resource_bindings.contains(binding)
            && let Some(tag) = self.remote_resources.get(binding)
        {
            self.index.remote_resource_appends.insert(tag.clone());
        }
    }
}

impl Visit for SemanticVisitor<'_> {
    fn visit_class_method(&mut self, method: &ClassMethod) {
        if let Some(name) = prop_name(&method.key)
            && matches!(name.as_str(), "onload" | "onunload")
        {
            self.index.lifecycle_methods.insert(name);
        }
        method.visit_children_with(self);
    }

    fn visit_fn_decl(&mut self, declaration: &FnDecl) {
        let previous = self.current_function.take();
        let function = self.binding(&declaration.ident);
        let parameters = declaration
            .function
            .params
            .iter()
            .enumerate()
            .filter_map(|(index, parameter)| {
                binding_ident(&parameter.pat).map(|ident| (self.binding(ident), index))
            })
            .collect();
        self.current_function = Some(FunctionContext {
            function,
            parameters,
        });
        declaration.function.decorators.visit_with(self);
        declaration.function.body.visit_with(self);
        self.current_function = previous;
    }

    fn visit_var_declarator(&mut self, declarator: &VarDeclarator) {
        if let Some(ident) = binding_ident(&declarator.name)
            && let Some(init) = declarator.init.as_deref()
        {
            self.assign_value(self.binding(ident), init);
        }
        declarator.visit_children_with(self);
    }

    fn visit_assign_expr(&mut self, assign: &AssignExpr) {
        if let Some(ident) = assigned_ident(&assign.left) {
            self.assign_value(self.binding(ident), &assign.right);
        } else if let Some((ident, property)) = assigned_member(&assign.left) {
            let object = self.binding(ident);
            if matches!(
                property.as_str(),
                "src" | "text" | "textContent" | "innerHTML"
            ) && self.scripts.contains(&object)
            {
                self.remote_scripts.insert(object.clone());
            }
            if matches!(property.as_str(), "src" | "href")
                && self.remote_resources.contains_key(&object)
                && expr_contains_remote_url(&assign.right)
            {
                self.remote_resource_bindings.insert(object.clone());
            }
            if property == "type"
                && self.inputs.contains(&object)
                && static_string(&assign.right).as_deref() == Some("file")
            {
                self.index.file_inputs.insert(object);
            }
        }
        assign.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, call: &CallExpr) {
        if let Some(callable) = self.invoked_dynamic_callable(call) {
            self.record_dynamic_call(callable);
        }
        if is_string_timer_call(call, self.aliases) {
            self.index
                .dynamic_code_sites
                .insert("string_timer".to_string());
        }

        if is_append_child_call(call)
            && let Some(ident) = ident_arg(call, 0)
        {
            let binding = self.binding(ident);
            self.record_parameter_effect(&binding, ParameterEffect::AppendToDocument);
            self.append_binding(&binding);
        }

        if let Some(member) = call_member_callee(call)
            && let Some(ident) = expr_ident(&member.obj)
            && let object = self.binding(ident)
            && self.scripts.contains(&object)
            && let Some(operation) = member_prop_name(&member.prop)
            && ((operation == "setAttribute"
                && string_arg(call, 0).is_some_and(|attribute| {
                    matches!(
                        attribute.to_ascii_lowercase().as_str(),
                        "src" | "text" | "textcontent" | "innerhtml"
                    )
                }))
                || matches!(
                    operation.as_str(),
                    "append" | "appendChild" | "replaceChildren"
                ))
        {
            self.remote_scripts.insert(object);
        }

        self.apply_function_effects(call);
        call.visit_children_with(self);
    }

    fn visit_new_expr(&mut self, new_expr: &NewExpr) {
        if self.dynamic_callable(&new_expr.callee) == Some(DynamicCallable::FunctionConstructor) {
            self.record_dynamic_call(DynamicCallable::FunctionConstructor);
        }
        new_expr.visit_children_with(self);
    }

    fn visit_member_expr(&mut self, member: &MemberExpr) {
        if let Some(ident) = expr_ident(&member.obj)
            && let object = self.binding(ident)
            && self.caches.contains(&object)
            && let Some(property) = member_prop_name(&member.prop)
            && METADATA_PROPS.contains(&property.as_str())
        {
            self.index.metadata_properties.insert(property);
        }
        member.visit_children_with(self);
    }
}

const METADATA_PROPS: &[&str] = &[
    "tags",
    "links",
    "embeds",
    "blocks",
    "headings",
    "sections",
    "listItems",
];
