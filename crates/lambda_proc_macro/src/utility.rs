use quote::quote;

pub(crate) fn geometry_fields() -> proc_macro2::TokenStream {
    let mut struct_fields = proc_macro2::TokenStream::default();
    struct_fields.extend(quote! {
        pub texture: lambda_internal::lambda_vulkan::TextureBuffer,
        pub indexed: lambda_internal::lambda_geometry::Indexed,
        pub topology: lambda_internal::lambda_vulkan::ModelTopology,
        pub cull_mode: lambda_internal::lambda_vulkan::CullMode,
        pub shader: lambda_internal::lambda_vulkan::Shader,
    });
    struct_fields
}
