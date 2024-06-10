extern crate proc_macro;

mod utility;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

struct InputVec {
    v: Vec<syn::Ident>,
}

impl Parse for InputVec {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut v = Vec::new();

        Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated(input)?
            .iter()
            .for_each(|arg| {
                v.push(arg.clone());
            });

        Ok(Self { v })
    }
}

#[proc_macro_attribute]
pub fn geometry_system(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as InputVec);

    let mut actions = Vec::new();
    let mut vertices_and_indices = Vec::new();
    let mut features = Vec::new();

    args.v.iter().for_each(|arg| {
        let cased = arg.to_token_stream().to_string().to_case(Case::Snake);

        let cased_tokens = syn::Ident::new(&cased, proc_macro2::Span::call_site());

        actions.push(quote::quote! { Self::#arg(#cased_tokens) => #cased_tokens.actions() });
        vertices_and_indices.push(
            quote::quote! { Self::#arg(#cased_tokens) => #cased_tokens.vertices_and_indices() },
        );
        features.push(quote::quote! { Self::#arg(#cased_tokens) => #cased_tokens.features() });
    });

    let item_struct = syn::parse_macro_input!(input as syn::ItemStruct);

    let vis = item_struct.vis;
    let struct_name = item_struct.ident;

    let args = args.v;

    quote::quote! {
        #[wave_internal::wave_geometry::enum_dispatch(GeomBuilder, Behavior)]
        #[derive(Debug, Clone)]
        #vis enum #struct_name {
            #(#args ,)*
        }

        impl Behavior for Geom {
            fn actions(&mut self) {
                match self {
                    #(#actions ,)*
                }
            }
        }

        impl GeomBuilder for Geom {
            fn vertices_and_indices(&self) -> VerticesAndIndices {
                match self {
                    #(#vertices_and_indices ,)*
                }
            }

            fn features(&self) -> wave_engine::wave_vulkan::GeomProperties {
                match self {
                    #(#features ,)*
                }
            }
        }
    }
    .into()
}

struct Input {
    v: syn::Ident,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self { v: input.parse()? })
    }
}

#[proc_macro_attribute]
pub fn geometry(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as Input);
    let shape_type = args.v;

    let item_struct = syn::parse_macro_input!(input as syn::ItemStruct);

    let vis = item_struct.vis;
    let struct_name = item_struct.ident;

    let struct_fields = utility::geometry_fields();

    quote::quote! {
        #[derive(Default, Debug, Clone)]
        #vis struct #struct_name {
            pub properties: #shape_type,
            #struct_fields
        }

        impl #struct_name {
            #vis fn properties(&mut self, properties: #shape_type) -> &mut Self {
                self.properties = properties;
                self
            }

            #vis fn topology(&mut self, topology: wave_internal::wave_vulkan::ModelTopology) -> &mut Self {
                self.topology = topology;
                self
            }

            #vis fn cull_mode(&mut self, cull_mode: wave_internal::wave_vulkan::CullMode) -> &mut Self {
                self.cull_mode = cull_mode;
                self
            }

            #vis fn shader(&mut self, shader: wave_internal::wave_vulkan::Shader) -> &mut Self {
                self.shader = shader;
                self
            }

            #vis fn texture(&mut self, path: &str) -> &mut Self {
                use std::io::Read;

                let path = path.to_owned();

                let texture_handle = std::thread::spawn(move || {
                    let file = std::fs::File::open(path);

                    if let Ok(mut texture_file) = file {
                        let mut data = Vec::new();
                        texture_file
                            .read_to_end(&mut data)
                            .expect("Failed to read contents of texture file");
                        return Some(data);
                    }
                    None
                });

                if let Some(mut texture) = texture_handle.join().unwrap() {
                    self.texture.append(&mut texture);
                }

                self
            }

            #vis fn no_index(&mut self) -> &mut Self {
                self.indexed = Indexed(false);
                self
            }

            #vis fn build(&mut self) -> Self {
                self.to_owned()
            }
        }

        impl GeomBuilder for #struct_name {
            #vis fn features(&self) -> wave_internal::wave_vulkan::GeomProperties {
                wave_internal::wave_vulkan::GeomProperties::new(
                    &self.texture,
                    self.vertices_and_indices(),
                    self.topology,
                    self.cull_mode,
                    self.shader,
                    *self.indexed,
                    self.properties.model
                )
            }

            #vis fn vertices_and_indices(&self) -> wave_internal::wave_space::space::VerticesAndIndices {
                self.properties.vertices_and_indices()
            }
        }

        impl Transformation for #struct_name {
            fn rotate_x(&mut self, angle: f32) {
                self.properties.model *= wave_internal::wave_geometry::utility::scaled_axis_matrix_4(wave_internal::wave_space::space::Pos3::x(), angle);
            }

            fn rotate_y(&mut self, angle: f32) {
                self.properties.model *= wave_internal::wave_geometry::utility::scaled_axis_matrix_4(wave_internal::wave_space::space::Pos3::y(), angle);
            }

            fn rotate_z(&mut self, angle: f32) {
                self.properties.model *= wave_internal::wave_geometry::utility::scaled_axis_matrix_4(wave_internal::wave_space::space::Pos3::z(), angle);
            }

            fn translate_x(&mut self, distance: f32) {
                self.properties.model *= wave_internal::wave_geometry::utility::translate(wave_internal::wave_space::space::Pos3::from_x(distance));
            }

            fn translate_y(&mut self, distance: f32) {
                self.properties.model *= wave_internal::wave_geometry::utility::translate(wave_internal::wave_space::space::Pos3::from_y(distance));
            }

            fn translate_z(&mut self, distance: f32) {
                self.properties.model *= wave_internal::wave_geometry::utility::translate(wave_internal::wave_space::space::Pos3::from_z(distance));
            }
        }
    }
    .into()
}
