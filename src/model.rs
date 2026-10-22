use crate::math::Vec3;
use std::{
    error::Error,
    fmt::{Display, Formatter},
    fs,
    path::Path,
};

#[derive(Clone, Debug)]
pub struct Face {
    /// Indices converted from OBJ's 1-based format to Rust's 0-based format.
    pub vertex_indices: Vec<usize>,
    pub normal_indices: Vec<Option<usize>>,
}

#[derive(Clone, Debug, Default)]
pub struct Model {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub faces: Vec<Face>,
}

#[derive(Debug)]
pub struct ObjError {
    line: usize,
    message: String,
}

impl ObjError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl Display for ObjError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 {
            write!(formatter, "{}", self.message)
        } else {
            write!(formatter, "línea {}: {}", self.line, self.message)
        }
    }
}

impl Error for ObjError {}

impl Model {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let source = fs::read_to_string(path)?;
        Ok(Self::from_obj_str(&source)?)
    }

    /// Reads the attributes needed by the pipeline: positions (`v`), normals
    /// (`vn`) and faces (`f`). Texture coordinates are accepted but not sampled.
    pub fn from_obj_str(source: &str) -> Result<Self, ObjError> {
        let mut model = Self::default();

        for (line_index, raw_line) in source.lines().enumerate() {
            let line_number = line_index + 1;
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split_whitespace();
            match fields.next() {
                Some("v") => {
                    let coordinates = fields
                        .take(3)
                        .map(|value| {
                            value.parse::<f32>().map_err(|_| {
                                ObjError::new(line_number, format!("coordenada inválida: {value}"))
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    if coordinates.len() != 3 {
                        return Err(ObjError::new(
                            line_number,
                            "un vértice debe tener tres coordenadas",
                        ));
                    }
                    model
                        .vertices
                        .push(Vec3::new(coordinates[0], coordinates[1], coordinates[2]));
                }
                Some("vn") => {
                    let coordinates = fields
                        .take(3)
                        .map(|value| {
                            value.parse::<f32>().map_err(|_| {
                                ObjError::new(line_number, format!("normal inválida: {value}"))
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    if coordinates.len() != 3 {
                        return Err(ObjError::new(
                            line_number,
                            "una normal debe tener tres coordenadas",
                        ));
                    }
                    model
                        .normals
                        .push(Vec3::new(coordinates[0], coordinates[1], coordinates[2]));
                }
                Some("f") => {
                    let mut indices = Vec::new();
                    let mut normal_indices = Vec::new();
                    for element in fields {
                        let mut components = element.split('/');
                        let position = components.next().unwrap_or_default();
                        let _texture = components.next();
                        let normal = components.next().filter(|value| !value.is_empty());

                        indices.push(resolve_obj_index(
                            position,
                            model.vertices.len(),
                            line_number,
                            "vértice",
                        )?);
                        normal_indices.push(
                            normal
                                .map(|value| {
                                    resolve_obj_index(
                                        value,
                                        model.normals.len(),
                                        line_number,
                                        "normal",
                                    )
                                })
                                .transpose()?,
                        );
                    }

                    if indices.len() < 3 {
                        return Err(ObjError::new(
                            line_number,
                            "una cara necesita al menos tres vértices",
                        ));
                    }
                    model.faces.push(Face {
                        vertex_indices: indices,
                        normal_indices,
                    });
                }
                _ => {}
            }
        }

        if model.vertices.is_empty() || model.faces.is_empty() {
            return Err(ObjError::new(0, "el OBJ no contiene vértices y caras"));
        }

        Ok(model)
    }

    pub fn center(&self) -> Vec3 {
        let (minimum, maximum) = self.bounds();
        (minimum + maximum) * 0.5
    }

    pub fn bounding_radius(&self, center: Vec3) -> f32 {
        self.vertices
            .iter()
            .map(|vertex| (*vertex - center).length())
            .fold(0.0, f32::max)
    }

    pub fn triangle_count(&self) -> usize {
        self.faces
            .iter()
            .map(|face| face.vertex_indices.len().saturating_sub(2))
            .sum()
    }

    fn bounds(&self) -> (Vec3, Vec3) {
        let mut minimum = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut maximum = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        for vertex in &self.vertices {
            minimum.x = minimum.x.min(vertex.x);
            minimum.y = minimum.y.min(vertex.y);
            minimum.z = minimum.z.min(vertex.z);
            maximum.x = maximum.x.max(vertex.x);
            maximum.y = maximum.y.max(vertex.y);
            maximum.z = maximum.z.max(vertex.z);
        }
        (minimum, maximum)
    }
}

fn resolve_obj_index(
    value: &str,
    item_count: usize,
    line_number: usize,
    kind: &str,
) -> Result<usize, ObjError> {
    let obj_index = value
        .parse::<isize>()
        .map_err(|_| ObjError::new(line_number, format!("índice de {kind} inválido: {value}")))?;

    let index = if obj_index > 0 {
        (obj_index - 1) as usize
    } else if obj_index < 0 {
        item_count
            .checked_sub(obj_index.unsigned_abs())
            .ok_or_else(|| ObjError::new(line_number, "índice relativo fuera de rango"))?
    } else {
        return Err(ObjError::new(line_number, "OBJ no permite el índice 0"));
    };

    if index >= item_count {
        return Err(ObjError::new(
            line_number,
            format!("el índice de {kind} {obj_index} no existe"),
        ));
    }
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::Model;

    #[test]
    fn parses_face_variants_and_triangulates_count() {
        let source = "\
v 0 0 0
v 1 0 0
v 1 1 0
v 0 1 0
vn 0 0 1
f 1 2/4 3/2/1 4//1
";
        let model = Model::from_obj_str(source).unwrap();
        assert_eq!(model.vertices.len(), 4);
        assert_eq!(model.faces[0].vertex_indices, vec![0, 1, 2, 3]);
        assert_eq!(model.triangle_count(), 2);
    }

    #[test]
    fn parses_negative_indices() {
        let source = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf -3 -2 -1\n";
        let model = Model::from_obj_str(source).unwrap();
        assert_eq!(model.faces[0].vertex_indices, vec![0, 1, 2]);
    }

    #[test]
    fn parses_normal_indices() {
        let source = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvn 0 0 1\nf 1//1 2//1 3//1\n";
        let model = Model::from_obj_str(source).unwrap();
        assert_eq!(model.normals.len(), 1);
        assert_eq!(model.faces[0].normal_indices, vec![Some(0); 3]);
    }
}
