extern crate proc_macro;

mod utility;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn geometry_system(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = syn::parse_macro_input!(input as syn::ItemStruct);
    let vis = item_struct.vis;
    let struct_name = item_struct.ident;

    quote::quote! {
        #vis enum #struct_name {

        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn geometry(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let shape_type = args.first().unwrap();

    let item_struct = syn::parse_macro_input!(input as syn::ItemStruct);

    dbg!(item_struct.clone());

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

            #vis fn topology(&mut self, topology: lambda_internal::lambda_vulkan::ModelTopology) -> &mut Self {
                self.topology = topology;
                self
            }

            #vis fn cull_mode(&mut self, cull_mode: lambda_internal::lambda_vulkan::CullMode) -> &mut Self {
                self.cull_mode = cull_mode;
                self
            }

            #vis fn shader(&mut self, shader: lambda_internal::lambda_vulkan::Shader) -> &mut Self {
                self.shader = shader;
                self
            }

            #vis fn texture(&mut self, path: &str) -> &mut Self {
                use std::io::Read;
                // use std::thread;

                // thread::spawn(move || {
                    let file = std::fs::File::open(path);

                    if let Ok(mut texture_file) = file {
                        let mut data = Vec::new();
                        texture_file
                        .read_to_end(&mut data)
                        .expect("Failed to read contents of texture file");
                        self.texture = TextureBuffer(data);
                    }
                // });

                self
            }

            #vis fn no_index(&mut self) -> &mut Self {
                self.indexed = Indexed(false);
                self
            }
        }

        impl lambda_internal::lambda_geometry::GeomBuilder for #struct_name {
            fn features(&self) -> lambda_internal::lambda_vulkan::GeomProperties {
                lambda_internal::lambda_vulkan::GeomProperties::new(
                    &self.texture,
                    self.vertices_and_indices(),
                    self.topology,
                    self.cull_mode,
                    self.shader,
                    *self.indexed,
                )
            }

            fn vertices_and_indices(&self) -> lambda_internal::lambda_space::space::VerticesAndIndices {
                todo!()
            }
        }

        // impl lambda_internal::lambda_geometry::Behavior for #struct_name {}
    }
    .into()
}
