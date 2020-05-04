extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};
use syn::parse_quote;
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{parse_macro_input, token::Comma, Block, Expr, ExprCall, FnArg, Ident, ItemFn, Pat};

struct TCO {
    ident: Ident,
    args: Vec<Ident>,
    i: u32,
}

impl TCO {
    fn rewrite_return_to_tco_update(&mut self, node: &mut Expr) -> bool {
        let expr_call: &mut ExprCall = match node {
            Expr::Call(expr_call) => expr_call,
            Expr::Await(await_call) => {
                if self.rewrite_return_to_tco_update(&mut *await_call.base) {
                    *node = *await_call.base.clone();
                }
                return false;
            }
            _ => {
                visit_mut::visit_expr_mut(self, node);
                return false;
            }
        };

        let mut replace_call = false;
        if let Expr::Path(ref mut fn_path) = *expr_call.func {
            if fn_path.attrs.len() == 0
                && fn_path.qself.is_none()
                && fn_path.path.leading_colon.is_none()
                && fn_path.path.segments.len() == 1
            {
                if fn_path.path.segments.first().unwrap().ident == self.ident {
                    replace_call = true;
                }
            }
        }

        if replace_call {
            let tco_ident = format_ident!("__tco_{}", self.i, span = expr_call.span());
            let tup = &mut expr_call.args;
            if !tup.trailing_punct() {
                tup.push_punct(Comma::default());
            }
            let updates = self.args.iter().enumerate().map(|(i, q)| {
                let i = syn::Index::from(i);
                quote!(#q = #tco_ident.#i;)
            });
            *node = parse_quote!({
                let #tco_ident = (#tup);
                #(#updates)*
                continue '__tco_loop;
            });
            return true;
        } else {
            visit_mut::visit_expr_mut(self, node);
            return false;
        }
    }
}

impl VisitMut for TCO {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        self.rewrite_return_to_tco_update(node);
    }
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn rewrite(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut input: ItemFn = parse_macro_input!(item as ItemFn);
    let fn_ident = input.sig.ident.clone();

    let mut tco = TCO {
        ident: fn_ident,
        args: input
            .sig
            .inputs
            .iter()
            .map(|a| match a {
                FnArg::Typed(pat) => match &*pat.pat {
                    Pat::Ident(ident_wrapper) => ident_wrapper.ident.clone(),
                    span => abort!(span, "TCO only supports basic function args"),
                },
                span => abort!(span, "TCO does not support self arg"),
            })
            .collect(),
        i: 0,
    };

    tco.visit_item_fn_mut(&mut input);
    {
        let old_body = input.block;
        let updates = tco.args.iter().map(|q| quote!(let mut #q = #q;));
        let new_body: Block = parse_quote!(
            {
                #(#updates)*
                '__tco_loop: loop {
                    #[deny(unreachable_code)]
                    return #old_body;
                }
            }
        );
        input.block = Box::new(new_body);
    }

    TokenStream::from(quote!(#input))
}
