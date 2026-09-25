#[cxx::bridge]
mod cpp {
    struct MeshAttribute {
        dim: u32,
        data_type: u32,
        offset: u32,
        length: u32,
        unique_id: u32,
    }

    struct MeshConfig {
        vertex_count: u32,
        index_count: u32,
        index_length: u32,
        buffer_size: usize,
        attributes: Vec<MeshAttribute>,
    }

    unsafe extern "C++" {
        include!("decoder_api.h");

        type DracoMesh;
        type DracoPointCloud;

        pub fn create_mesh(data: &[u8]) -> UniquePtr<DracoMesh>;

        pub fn compute_mesh_config(mesh: &DracoMesh, config: &mut MeshConfig) -> bool;

        pub unsafe fn decode_mesh_to_buffer(
            mesh: &DracoMesh,
            out_ptr: *mut u8,
            out_len: usize,
        ) -> usize;

        pub fn create_point_cloud(data: &[u8]) -> UniquePtr<DracoPointCloud>;

        pub fn compute_point_cloud_config(pc: &DracoPointCloud, config: &mut MeshConfig) -> bool;

        pub unsafe fn decode_point_cloud_to_buffer(
            pc: &DracoPointCloud,
            out_ptr: *mut u8,
            out_len: usize,
        ) -> usize;
    }
}

fn convert_config(cpp_config: cpp::MeshConfig) -> crate::DracoDecodeConfig {
    let mut config = crate::DracoDecodeConfig::new(
        cpp_config.vertex_count,
        cpp_config.index_count,
        cpp_config.buffer_size,
    );

    for attr in cpp_config.attributes {
        let data_type = crate::AttributeDataType::from_draco_data_type(attr.data_type as i32);
        config.add_attribute_with_id(
            attr.dim,
            data_type,
            attr.offset,
            attr.length,
            attr.unique_id,
        );
    }

    config
}

pub fn decode_mesh_with_config(data: &[u8]) -> Option<crate::MeshDecodeResult> {
    let mesh = cpp::create_mesh(data);
    if mesh.is_null() {
        panic!("Failed to create mesh from data");
    }

    let mut cpp_config = cpp::MeshConfig {
        vertex_count: 0,
        index_count: 0,
        index_length: 0,
        buffer_size: 0,
        attributes: Vec::new(),
    };

    if !cpp::compute_mesh_config(&mesh, &mut cpp_config) {
        panic!("Failed to compute mesh config");
    }

    let buffer_size = cpp_config.buffer_size;
    let config = convert_config(cpp_config);
    let mut buffer = vec![0u8; buffer_size];

    let written = unsafe { cpp::decode_mesh_to_buffer(&mesh, buffer.as_mut_ptr(), buffer.len()) };

    if written == 0 {
        panic!("Failed to decode mesh to buffer");
    }

    buffer.truncate(written);

    Some(crate::MeshDecodeResult {
        data: buffer,
        config,
    })
}

/// Decodes a Draco point cloud into a packed attribute buffer plus its
/// layout config (`index_count`/`index_length` are 0).
///
/// Returns `None` on any failure (malformed input, null geometry, write
/// overflow) instead of panicking — callers like asset loaders feed
/// untrusted files.
pub fn decode_point_cloud_with_config(data: &[u8]) -> Option<crate::MeshDecodeResult> {
    let point_cloud = cpp::create_point_cloud(data);
    if point_cloud.is_null() {
        return None;
    }

    let mut cpp_config = cpp::MeshConfig {
        vertex_count: 0,
        index_count: 0,
        index_length: 0,
        buffer_size: 0,
        attributes: Vec::new(),
    };

    if !cpp::compute_point_cloud_config(&point_cloud, &mut cpp_config) {
        return None;
    }

    let buffer_size = cpp_config.buffer_size;
    let config = convert_config(cpp_config);
    let mut buffer = vec![0u8; buffer_size];

    let written = unsafe {
        cpp::decode_point_cloud_to_buffer(&point_cloud, buffer.as_mut_ptr(), buffer.len())
    };

    if written == 0 {
        return None;
    }

    buffer.truncate(written);

    Some(crate::MeshDecodeResult {
        data: buffer,
        config,
    })
}
