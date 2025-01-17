use anyhow::Result;
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::render::{
    mesh::{Indices, Mesh, VertexAttributeValues},
    pipeline::PrimitiveTopology,
};
use bevy::utils::BoxedFuture;
use thiserror::Error;

#[derive(Default)]
pub struct ObjLoader;

impl AssetLoader for ObjLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move { Ok(load_obj(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["obj"];
        EXTENSIONS
    }
}

#[derive(Error, Debug)]
pub enum ObjError {
    #[error("Invalid OBJ file.")]
    Gltf(#[from] obj::ObjError),
    #[error("Unknown vertex format.")]
    UnknownVertexFormat,
}

async fn load_obj<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), ObjError> {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    load_obj_from_bytes(bytes, &mut mesh)?;
    load_context.set_default_asset(LoadedAsset::new(mesh));
    Ok(())
}

fn load_obj_from_bytes(bytes: &[u8], mesh: &mut Mesh) -> Result<(), ObjError> {
    let raw = obj::raw::parse_obj(bytes)?;

    // Get the most complete vertex representation
    //  3 => Position, Normal, Texture
    //  2 => Position, Normal
    //  1 => Position
    let mut pnt = 3;
    for polygon in &raw.polygons {
        use obj::raw::object::Polygon;
        match polygon {
            Polygon::P(_) => pnt = std::cmp::min(pnt, 1),
            Polygon::PN(_) => pnt = std::cmp::min(pnt, 2),
            _ => {}
        }
    }

    match pnt {
        1 => {
            let obj: obj::Obj<obj::Position, u32> = obj::Obj::new(raw)?;
            set_position_data(mesh, obj.vertices.iter().map(|v| v.position).collect());
            set_normal_data(mesh, obj.vertices.iter().map(|_| [0., 0., 0.]).collect());
            set_uv_data(mesh, obj.vertices.iter().map(|_| [0., 0., 0.]).collect());
            set_mesh_indices(mesh, obj);
        }
        2 => {
            let obj: obj::Obj<obj::Vertex, u32> = obj::Obj::new(raw)?;
            set_position_data(mesh, obj.vertices.iter().map(|v| v.position).collect());
            set_normal_data(mesh, obj.vertices.iter().map(|v| v.normal).collect());
            set_uv_data(mesh, obj.vertices.iter().map(|_| [0., 0., 0.]).collect());
            set_mesh_indices(mesh, obj);
        }
        3 => {
            let obj: obj::Obj<obj::TexturedVertex, u32> = obj::Obj::new(raw)?;
            set_position_data(mesh, obj.vertices.iter().map(|v| v.position).collect());
            set_normal_data(mesh, obj.vertices.iter().map(|v| v.normal).collect());
            set_uv_data(
                mesh,
                obj.vertices
                    .iter()
                    // Flip UV for correct values
                    .map(|v| [v.texture[0], 1.0 - v.texture[1], v.texture[2]])
                    .collect(),
            );
            set_mesh_indices(mesh, obj);
        }
        _ => return Err(ObjError::UnknownVertexFormat),
    }

    Ok(())
}

fn set_position_data(mesh: &mut Mesh, data: Vec<[f32; 3]>) {
    let positions = VertexAttributeValues::Float32x3(data);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
}

fn set_normal_data(mesh: &mut Mesh, data: Vec<[f32; 3]>) {
    let normals = VertexAttributeValues::Float32x3(data);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
}

fn set_uv_data(mesh: &mut Mesh, data: Vec<[f32; 3]>) {
    let uvs = VertexAttributeValues::Float32x3(data);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
}

fn set_mesh_indices<T>(mesh: &mut Mesh, obj: obj::Obj<T, u32>) {
    mesh.set_indices(Some(Indices::U32(
        obj.indices.iter().map(|i| *i as u32).collect(),
    )));
}
