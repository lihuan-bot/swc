use crate::{
    config::{LintRuleReaction, RuleConfig},
    rule::{visitor_rule, Rule},
};
use swc_atoms::JsWord;
use swc_common::{collections::AHashSet, errors::HANDLER, Span, SyntaxContext};
use swc_ecma_ast::*;
use swc_ecma_utils::{collect_decls_with_ctxt, ident::IdentLike};
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};

const FN_NAMES: &[&str] = &["alert", "confirm", "prompt"];
const GLOBAL_THIS_PROP: &str = "globalThis";
const OBJ_NAMES: &[&str] = &["window", GLOBAL_THIS_PROP];

pub fn no_alert(
    program: &Program,
    config: &RuleConfig<()>,
    top_level_ctxt: SyntaxContext,
    es_version: EsVersion,
) -> Option<Box<dyn Rule>> {
    let top_level_declared_vars: AHashSet<Id> = collect_decls_with_ctxt(program, top_level_ctxt);
    let rule_reaction = config.get_rule_reaction();

    match rule_reaction {
        LintRuleReaction::Off => None,
        _ => Some(visitor_rule(NoAlert::new(
            *rule_reaction,
            top_level_declared_vars,
            top_level_ctxt,
            es_version,
        ))),
    }
}

#[derive(Debug, Default)]
struct NoAlert {
    expected_reaction: LintRuleReaction,
    top_level_ctxt: SyntaxContext,
    top_level_declared_vars: AHashSet<Id>,
    pass_call_on_global_this: bool,
    inside_callee: bool,
    obj: Option<JsWord>,
    prop: Option<JsWord>,
}

impl NoAlert {
    fn new(
        expected_reaction: LintRuleReaction,
        top_level_declared_vars: AHashSet<Id>,
        top_level_ctxt: SyntaxContext,
        es_version: EsVersion,
    ) -> Self {
        Self {
            expected_reaction,
            top_level_ctxt,
            top_level_declared_vars,
            pass_call_on_global_this: es_version < EsVersion::Es2020,
            inside_callee: false,
            obj: None,
            prop: None,
        }
    }

    fn emit_report(&self, span: Span, fn_name: &str) {
        let message = format!("Unexpected {}", fn_name);

        HANDLER.with(|handler| match self.expected_reaction {
            LintRuleReaction::Error => {
                handler.struct_span_err(span, &message).emit();
            }
            LintRuleReaction::Warning => {
                handler.struct_span_warn(span, &message).emit();
            }
            _ => {}
        });
    }

    fn check(&self, call_span: Span, obj: &Option<JsWord>, prop: &JsWord) {
        if let Some(obj) = obj {
            let obj_name: &str = &*obj;

            if self.pass_call_on_global_this && obj_name == GLOBAL_THIS_PROP {
                return;
            }

            if !OBJ_NAMES.contains(&obj_name) {
                return;
            }
        }

        let fn_name: &str = &*prop;

        if FN_NAMES.contains(&fn_name) {
            self.emit_report(call_span, fn_name);
        }
    }

    fn is_satisfying_indent(&self, ident: &Ident) -> bool {
        if ident.span.ctxt != self.top_level_ctxt {
            return false;
        }

        if self.top_level_declared_vars.contains(&ident.to_id()) {
            return false;
        }

        true
    }

    fn handle_callee(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(ident) => {
                if self.is_satisfying_indent(ident) {
                    self.prop = Some(ident.sym.clone());
                }
            }
            Expr::Member(member_expr) => {
                let MemberExpr { obj, prop, .. } = member_expr;

                if let Expr::Ident(obj) = obj.as_ref() {
                    if !self.is_satisfying_indent(obj) {
                        return;
                    }

                    self.obj = Some(obj.sym.clone());

                    match prop {
                        MemberProp::Ident(Ident { sym, .. }) => {
                            self.prop = Some(sym.clone());
                        }
                        MemberProp::Computed(comp) => {
                            if let Expr::Lit(Lit::Str(Str { value, .. })) = comp.expr.as_ref() {
                                self.prop = Some(value.clone());
                            }
                        }
                        _ => {}
                    }
                }

                // TODO: handle call alert on "this"
            }
            Expr::OptChain(opt_chain) => {
                opt_chain.visit_children_with(self);
            }
            Expr::Paren(paren) => {
                paren.visit_children_with(self);
            }
            _ => {}
        }
    }

    fn handle_call(&mut self, call_expr: &CallExpr) {
        if let Some(callee) = call_expr.callee.as_expr() {
            self.inside_callee = true;

            callee.visit_with(self);

            self.inside_callee = false;
        }

        if let Some(prop) = &self.prop {
            self.check(call_expr.span, &self.obj, prop);

            self.obj = None;
            self.prop = None;
        }
    }
}

impl Visit for NoAlert {
    noop_visit_type!();

    fn visit_expr(&mut self, expr: &Expr) {
        if self.inside_callee {
            self.handle_callee(expr);
        } else {
            if let Expr::Call(call_expr) = expr {
                self.handle_call(call_expr);
            }

            expr.visit_children_with(self);
        }
    }
}
