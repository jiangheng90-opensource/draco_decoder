#pragma once
#include <memory>

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
