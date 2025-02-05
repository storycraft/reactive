use syn::{
    spanned::Spanned, visit_mut::{visit_expr_field_mut, VisitMut}, Expr, ExprField, ExprPath, Ident
};

pub struct ComputedExpr {
    pub dependencies: Vec<Ident>,
    pub expr: Expr,
}

impl ComputedExpr {
    pub fn calc_deps(expr: Expr) -> Self {
        let dependencies = Vec::new();

        Self { dependencies, expr }
    }
}

fn extract(expr: &Expr, deps: &mut Vec<Ident>) {}

struct Visitor<'a> {
    pub deps: &'a mut Vec<Ident>,
}

impl VisitMut for Visitor<'_> {
    fn visit_ident_mut(&mut self, i: &mut Ident) {
        if i == "self" {
            *i = Ident::new("this", i.span());
        }
    }

    fn visit_expr_field_mut(&mut self, i: &mut ExprField) {
        visit_expr_field_mut(self, i);

        if let Expr::Path(ExprPath { ref mut path, .. }) = &mut *i.base {
            if let Some(this) = path.segments.first_mut() {
                if this.ident == "self" {
                    this.ident = Ident::new("this", this.span());
                }
            }
        }
    }
}

struct SelfFix;

impl VisitMut for SelfFix {
    fn visit_ident_mut(&mut self, i: &mut Ident) {
        if i == "self" {
            *i = Ident::new("this", i.span());
        }
    }
}