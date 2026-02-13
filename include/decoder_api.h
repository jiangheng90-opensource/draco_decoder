#pragma once
#include "rust/cxx.h"
#include <cstdint>
#include <vector>
#include <memory>

// Forward declarations - defined in ffi.rs.h
struct MeshAttribute;
struct MeshConfig;

// Forward declaration for draco::Mesh
namespace draco {
class Mesh;
}

// DracoMesh class - wraps draco::Mesh
class DracoMesh {
public:
  std::unique_ptr<draco::Mesh> mesh;

  explicit DracoMesh(std::unique_ptr<draco::Mesh> m);
  ~DracoMesh();
};


rust::Vec<uint8_t> decode_point_cloud(rust::Slice<const uint8_t> data);

// Cache API - returns opaque type
std::unique_ptr<DracoMesh> create_mesh(rust::Slice<const uint8_t> data);

// Mesh Config from DracoMesh
bool compute_mesh_config(const DracoMesh &mesh, MeshConfig &config);

// Decode to pre-allocated buffer
size_t decode_mesh_to_buffer(const DracoMesh &mesh, uint8_t *out_ptr, size_t out_len);
