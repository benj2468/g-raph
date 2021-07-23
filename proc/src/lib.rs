use proc_macro::{Ident, TokenStream, TokenTree};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::ParseStream, parse_quote::ParseQuote, punctuated::Punctuated};

#[proc_macro]
pub fn test(input: TokenStream) -> TokenStream {
    let stream = syn::parse(input);
    // let mut iter = input.into_iter();

    println!("{:?}", stream);

    // let punctuated = Punctuated::parse_separated_nonempty(stream);

    // let (file_name, span) = if let TokenTree::Literal(name) = iter.next().unwrap() {
    //     (name.to_string().replace("\"", ""), name.span())
    // } else {
    //     panic!()
    // };

    // iter.next();

    // let nodes = if let TokenTree::Literal(n) = iter.next().unwrap() {
    //     n
    // } else {
    //     panic!()
    // };

    // let func_name = Ident::new(&file_name, span);

    // let quoted = quote! {
    //     #[test]
    //     fn #func_name() {
    //         let file = File::open(format!("./big_graphs/{}", #file_name)).unwrap();

    //         let mut colorers: Vec<_> = (0..(#nodes.log2().floor() as u32))
    //             .into_iter()
    //             .map(|i| {
    //                 let k = (2 as u32).pow(i) as u64;
    //                 StreamColoring::init(#nodes as u32, k)
    //             })
    //             .collect();

    //         for line in io::BufReader::new(file).lines() {
    //             if let Ok(line) = line {
    //                 let mut split = line.split(" ");
    //                 let v1: u32 = split.next().unwrap().parse().unwrap();
    //                 let v2: u32 = split.next().unwrap().parse().unwrap();

    //                 let edge = Edge::<u32, ()>::init(v1, v2);

    //                 for colorer in &mut colorers {
    //                     colorer.feed(edge, true)
    //                 }
    //             }
    //         }

    //         let mut min_color = INFINITY as usize;
    //         for colorer in colorers {
    //             if let Some(coloring) = colorer.query() {
    //                 let count = coloring.iter().unique().count();

    //                 min_color = min(min_color, count);
    //             }
    //         }

    //         min_color
    //     }
    // };

    TokenStream::from(quote! {})
}
