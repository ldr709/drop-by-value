extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::Tokens;
use quote::ToTokens;
use syn::*;

#[proc_macro_derive(DropByValue, attributes(DropByValue))]
pub fn drop_by_value(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();

    let inner_name = &ast.ident;

    let mut name: Option<&str> = None;
    let mut visibility: Option<String> = None;
    let mut attrs: Vec<&MetaItem> = Vec::new();

    let attr = get_drop_by_value_attr(&ast.attrs);
    for item in attr {
        match *item {
            NestedMetaItem::MetaItem(MetaItem::NameValue(ref key, Lit::Str(ref value, _))) => {
                match key.as_ref() {
                    "name" => {
                        if name == None {
                            name = Some(value);
                        } else {
                            panic!("Cannot have multiple names.");
                        }
                    }
                    "vis" => {
                        if visibility == None {
                            visibility = Some(value.clone());
                        } else {
                            panic!("Cannot have multiple visibilities.");
                        }
                    }
                    _ => panic!("Unrecognized DropByValue attribute argument \"{}\".", key),
                }
            }
            NestedMetaItem::MetaItem(ref x) => attrs.push(x),
            ref x => {
                panic!(
                    "Unrecognized DropByValue attribute argument \"{}\".",
                    tokens_to_string(x)
                )
            }
        }
    }
    let name: quote::Ident = name.expect("Drop by value type must have a name.").into();

    // Fake it as an ident so that it doesn't get put in quotes.
    let visibility: quote::Ident = visibility
        .unwrap_or_else(|| tokens_to_string(&ast.vis))
        .into();

    let generics = &ast.generics;
    let generics_rhs = Generics {
        lifetimes: generics
            .lifetimes
            .iter()
            .map(|lt| {
                LifetimeDef {
                    attrs: Vec::new(),
                    bounds: Vec::new(),
                    lifetime: lt.lifetime.clone(),
                }
            })
            .collect(),
        ty_params: generics
            .ty_params
            .iter()
            .map(|ty| {
                TyParam {
                    attrs: Vec::new(),
                    bounds: Vec::new(),
                    ident: ty.ident.clone(),
                    default: ty.default.clone(),
                }
            })
            .collect(),
        where_clause: WhereClause { predicates: Vec::new() },
    };

    let where_clause = &generics.where_clause;

    let destructure_vis = if let Body::Struct(ref struct_body) = ast.body {
        match *struct_body {
            VariantData::Struct(ref x) => &x[..],
            VariantData::Tuple(ref x) => &x[..],
            VariantData::Unit => &[],
        }.iter()
            .map(|f| &f.vis)
            .fold(&ast.vis, visibility_max)
    } else {
        &ast.vis
    };

    let output =
        quote! {

        #(#[#attrs])*
        #visibility struct #name#generics
        (#destructure_vis ::drop_by_value::DropByValueWrapper<#inner_name#generics_rhs>)
        #where_clause;

        impl#generics ::drop_by_value::internal::Destructure<#inner_name#generics_rhs>
            for #name#generics_rhs
        #where_clause {
            fn destructure(mut self_: Self) -> #inner_name#generics_rhs {
                let x = unsafe { ::std::ptr::read(self_.0.deref_mut()) };
                ::std::mem::forget(self_);
                x
            }
        }

        impl#generics Drop for #name#generics_rhs
        #where_clause {
            fn drop(&mut self) {
                let self_ = unsafe { ::std::ptr::read(self) };
                let drop_ref = ::drop_by_value::internal::drop_ref_new(self_);
                ::drop_by_value::DropValue::drop_value(drop_ref);
            }
        }
    };

    output.parse().unwrap()
}

// Find the DropByValue attribute in the list and return the list of its arguments.
fn get_drop_by_value_attr(attrs: &[Attribute]) -> &[NestedMetaItem] {
    let mut attr_iter = attrs.iter().filter(|x| {
        if let MetaItem::List(ref name, _) = x.value {
            return name == "DropByValue";
        }
        false
    });

    let exactly_one_msg = "Drop by value types must exactly one attribute \"DropByValue\".";
    let attr = attr_iter.next().expect(exactly_one_msg);
    assert!(attr_iter.next() == None, exactly_one_msg);

    if let Attribute {
        style: AttrStyle::Outer,
        value: MetaItem::List(_, ref args),
        is_sugared_doc: false,
    } = *attr
    {
        args
    } else {
        panic!("Drop by value attribute must be of the form \"#[DropByValue(...)]\".");
    }
}

fn tokens_to_string<T: ToTokens>(t: &T) -> String {
    let mut tokens = Tokens::new();
    t.to_tokens(&mut tokens);
    tokens.into_string()
}

fn visibility_max<'a>(x: &'a Visibility, y: &'a Visibility) -> &'a Visibility {
    match (x, y) {
        (&Visibility::Public, _) => y,
        (_, &Visibility::Public) => x,

        (&Visibility::Crate, _) => y,
        (_, &Visibility::Crate) => x,

        (&Visibility::Inherited, _) => x,
        (_, &Visibility::Inherited) => y,

        (&Visibility::Restricted(_), &Visibility::Restricted(_)) => {
            // TODO: Find the intersection of the two paths.

            static OUT: Visibility = Visibility::Inherited;
            &OUT
        }
    }
}
