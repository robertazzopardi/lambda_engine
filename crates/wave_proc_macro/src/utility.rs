use quote::quote;

pub(crate) fn geometry_fields() -> proc_macro2::TokenStream {
    let mut struct_fields = proc_macro2::TokenStream::default();
    struct_fields.extend(quote! {
        pub texture: wave_internal::wave_vulkan::TextureBuffer,
        pub indexed: wave_internal::wave_geometry::Indexed,
        pub topology: wave_internal::wave_vulkan::ModelTopology,
        pub cull_mode: wave_internal::wave_vulkan::CullMode,
        pub shader: wave_internal::wave_vulkan::Shader,
    });
    struct_fields
}
