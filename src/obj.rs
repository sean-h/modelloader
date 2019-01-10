extern crate tdmath;

use nom::*;
use nom::types::CompleteStr;
use tdmath::Vector3;
use crate::model::*;

/*
    Basic Parsers
*/

named!(space<CompleteStr, CompleteStr>,
    tag!(" ")
);

fn is_space(c: char) -> bool {
    c == ' '
}

named!(spaces<CompleteStr, CompleteStr>,
    take_while1!(is_space)
);

named!(name<CompleteStr, CompleteStr>,
    take_until!("\n")
);

named!(filename<CompleteStr, CompleteStr>,
    take_until!("\n")
);

named!(line_end<CompleteStr, CompleteStr>,
    preceded!(
        opt!(spaces),
        alt!(tag!("\n") | comment)
    )
);

named!(empty_line<CompleteStr, CompleteStr>,
    preceded!(
        opt!(spaces),
        tag!("\n")
    )
);

/*
    Comments
*/

named!(comment<CompleteStr, CompleteStr>,
    do_parse!(
        tag!("#") >>
        comment: take_until!("\n") >>
        tag!("\n") >>

        (comment)
    )
);

named!(ignore_line<CompleteStr, CompleteStr>,
    alt!(empty_line | comment)
);

named!(ignore_lines<CompleteStr, Vec<CompleteStr>>,
    many0!(comment)
);

/*
    Object Name
*/

named!(object_name_specifier<CompleteStr, CompleteStr>,
    tag!("o")
);

named!(object_name<CompleteStr, CompleteStr>,
    do_parse!(
        object_name_specifier >>
        space >>
        n: name >>
        tag!("\n") >>

        (n)
    )
);

/*
    Vertex
*/

named!(vertex<CompleteStr, Vector3>,
    do_parse!(
        opt!(spaces) >>
        tag!("v ") >>
        opt!(space) >>
        x: float >>
        space >>
        y: float >>
        space >>
        z: float >>
        line_end >>

        (Vector3::new(x, y, z))
    )
);

named!(vertex_list<CompleteStr, Vec<Vector3>>,
    do_parse!(
        opt!(many0!(ignore_line)) >>
        v: many0!(vertex) >>
        opt!(tag!("\n")) >>

        (v)
    )
);

/*
    Texture Coordinates
*/

named!(texture_coordinates<CompleteStr, Vector3>,
    do_parse!(
        tag!("vt") >>
        space >>
        x: float >>
        space >>
        y: float >>
        opt!(spaces) >>
        alt!(tag!("\n") | comment) >>

        (Vector3::new(x, y, 0.0))
    )
);

named!(texture_coordinate_list<CompleteStr, Vec<Vector3>>,
    do_parse!(
        opt!(many0!(ignore_line)) >>
        uv: many0!(texture_coordinates) >>
        opt!(tag!("\n")) >>

        (uv)
    )
);

/*
    Vertex Normals
*/

named!(vertex_normal<CompleteStr, Vector3>,
    do_parse!(
        tag!("vn") >>
        space >>
        x: float >>
        space >>
        y: float >>
        space >>
        z: float >>
        opt!(spaces) >>
        alt!(tag!("\n") | comment) >>

        (Vector3::new(x, y, z))
    )
);

named!(vertex_normal_list<CompleteStr, Vec<Vector3>>,
    do_parse!(
        opt!(many0!(ignore_line)) >>
        vn: many0!(vertex_normal) >>
        opt!(alt!(tag!("\n") | eof!())) >>

        (vn)
    )
);

/*
    Materials
*/

named!(material_file<CompleteStr, CompleteStr>,
    do_parse!(
        tag!("mtllib") >>
        spaces >>
        name: filename >>
        line_end >>

        (name)
    )
);

named!(usemtl<CompleteStr, CompleteStr>,
    do_parse!(
        tag!("usemtl") >>
        spaces >>
        name: name >>
        opt!(spaces) >>
        alt!(tag!("\n") | comment) >>

        (name)
    )
);

/*
    Smooth Shading
*/

fn str_to_bool(s: CompleteStr) -> Result<bool, CompleteStr> {
    if s == CompleteStr("on") {
        Ok(true)
    } else if s == CompleteStr("off") {
        Ok(false)
    } else {
        Err(CompleteStr("Cannot convert string to bool"))
    }
}

named!(smooth_shading<CompleteStr, bool>,
    do_parse!(
        tag!("s") >>
        spaces >>
        b: map_res!(take_until!("\n"), str_to_bool) >>
        opt!(spaces) >>
        alt!(tag!("\n") | comment) >>

        (b)
    )
);

/*
    Face
*/

struct FaceIndexed {
    pub vertexes: [usize; 3],
    pub texture_coordinates: [Option<usize>; 3],
    pub vertex_normals: [usize; 3],
}

named!(face_index<CompleteStr, (usize, Option<usize>, usize)>,
    do_parse!(
        v: digit >>
        opt!(tag!("/")) >>
        t: opt!(digit) >>
        opt!(tag!("/")) >>
        vn: digit >>

        (v.parse::<usize>().unwrap(),
        match t {
            Some(t) => Some(t.parse::<usize>().unwrap()),
            None => None
        },
        vn.parse::<usize>().unwrap())
    )
);

named!(face<CompleteStr, FaceIndexed>,
    do_parse!(
        tag!("f") >>
        space >>
        i1: face_index >>
        space >>
        i2: face_index >>
        space >>
        i3: face_index >>
        opt!(spaces) >>
        alt!(tag!("\n") | comment) >>

        (FaceIndexed {
            vertexes: [i1.0, i2.0, i3.0],
            texture_coordinates: [i1.1, i2.1, i3.1],
            vertex_normals: [i1.2, i2.2, i3.2]
        })
    )
);

named!(face_list<CompleteStr, Vec<FaceIndexed>>,
    do_parse!(
        opt!(many0!(ignore_line)) >>
        f: many0!(face) >>
        opt!(alt!(tag!("\n") | eof!())) >>

        (f)
    )
);

pub fn parse_obj_file(data: &str) -> Model {
    // Leading comments
    let remainder = match ignore_lines(CompleteStr(data)) {
        Ok((remainder, _)) => remainder,
        Err(_) => panic!("Unable to parse OBJ file: error reading leading comments")
    };

    let (remainder, mtllib) = match material_file(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading material file")
    };

    let (remainder, obj_name) = match object_name(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading object name")
    };

    
    let (remainder, vertex_positions) = match vertex_list(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading vertex positions")
    };

    let mut vertices = Vec::new();
    for v in vertex_positions {
        vertices.push(Vertex { p: v, uv: [0.0, 0.0]});
    }

    let (remainder, uvs) = match texture_coordinate_list(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading UV coordinates")
    };

    let (remainder, vertex_normals) = match vertex_normal_list(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading vertex normals")
    };

    let (remainder, usemtl) = match usemtl(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading usemtl")
    };

    let (remainder, smooth_shading) = match smooth_shading(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading smooth shading")
    };

    let (remainder, faces) = match face_list(remainder) {
        Ok(f) => f,
        Err(_) => panic!("Unable to parse OBJ file: error reading faces")
    };

    let mut triangles = Vec::new();

    for f in faces {
        
    }

    Model {
        name: obj_name.to_string(),
        vertices,
        triangles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_object_name_specifier() {
        let input = CompleteStr("o cube");
        let expected_remainder = CompleteStr(" cube");
        let expected_output = CompleteStr("o");
        assert_eq!(object_name_specifier(input), Ok((expected_remainder, expected_output)));
    }

    #[test]
    fn test_parse_name() {
        let input = CompleteStr("cube\n");
        let expected_remainder = CompleteStr("\n");
        let expected_output = CompleteStr("cube");
        assert_eq!(name(input), Ok((expected_remainder, expected_output)));
    }

    #[test]
    fn test_parse_object_name() {
        let input = CompleteStr("o cube\n");
        let expected_remainder = CompleteStr("");
        let expected_output = CompleteStr("cube");
        assert_eq!(object_name(input), Ok((expected_remainder, expected_output)));
    }

    #[test]
    fn test_parse_vertex() {
        let input = CompleteStr("v 1.000000 1.000000 -1.000000\n");
        let expected_remainder = CompleteStr("");

        match vertex(input) {
            Ok((remainder, v)) => {
                assert_eq!(remainder, expected_remainder);
                assert_eq!(v.x, 1.0);
                assert_eq!(v.y, 1.0);
                assert_eq!(v.z, -1.0);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_vertex_with_comment() {
        let input = CompleteStr("v 1.000000 1.000000 -1.000000 #Vertex 1\n");
        let expected_remainder = CompleteStr("");

        match vertex(input) {
            Ok((remainder, v)) => {
                assert_eq!(remainder, expected_remainder);
                assert_eq!(v.x, 1.0);
                assert_eq!(v.y, 1.0);
                assert_eq!(v.z, -1.0);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_vertex_with_leading_whitespace() {
        let input = CompleteStr("   v 1.000000 1.000000 -1.000000 #Vertex 1\n");
        let expected_remainder = CompleteStr("");

        match vertex(input) {
            Ok((remainder, v)) => {
                assert_eq!(remainder, expected_remainder);
                assert_eq!(v.x, 1.0);
                assert_eq!(v.y, 1.0);
                assert_eq!(v.z, -1.0);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_vertex_list() {
        let input = CompleteStr("v 1.000000 1.000000 -1.000000\nv 1.000000 -1.000000 -1.000000\nv 1.000000 1.000000 1.000000\n");
        let expected_remainder = CompleteStr("");

        match vertex_list(input) {
            Ok((remainder, vertices)) => {
                assert_eq!(remainder, expected_remainder);
                assert_eq!(vertices.len(), 3);
                assert_eq!(vertices[0].x, 1.0);
                assert_eq!(vertices[0].y, 1.0);
                assert_eq!(vertices[0].z, -1.0);
                assert_eq!(vertices[1].x, 1.0);
                assert_eq!(vertices[1].y, -1.0);
                assert_eq!(vertices[1].z, -1.0);
                assert_eq!(vertices[2].x, 1.0);
                assert_eq!(vertices[2].y, 1.0);
                assert_eq!(vertices[2].z, 1.0);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_vertex_list_with_following_texture_coordinates() {
        let input = CompleteStr("v 1.000000 1.000000 -1.000000\nv 1.000000 -1.000000 -1.000000\nv 1.000000 1.000000 1.000000\nvt 0.333134 0.000200\n");
        let expected_remainder = CompleteStr("vt 0.333134 0.000200\n");

        match vertex_list(input) {
            Ok((remainder, vertices)) => {
                assert_eq!(remainder, expected_remainder);
                assert_eq!(vertices.len(), 3);
                assert_eq!(vertices[0].x, 1.0);
                assert_eq!(vertices[0].y, 1.0);
                assert_eq!(vertices[0].z, -1.0);
                assert_eq!(vertices[1].x, 1.0);
                assert_eq!(vertices[1].y, -1.0);
                assert_eq!(vertices[1].z, -1.0);
                assert_eq!(vertices[2].x, 1.0);
                assert_eq!(vertices[2].y, 1.0);
                assert_eq!(vertices[2].z, 1.0);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_comment() {
        let input = CompleteStr("#this is a comment\n");
        let expected_remainder = CompleteStr("");
        let expected_output = CompleteStr("this is a comment");
        
        assert_eq!(comment(input), Ok((expected_remainder, expected_output)));
    }

    #[test]
    fn test_parse_spaces() {
        let input = CompleteStr("   spaces  ");
        let expected_remainder = CompleteStr("spaces  ");
        let expected_output = CompleteStr("   ");

        assert_eq!(spaces(input), Ok((expected_remainder, expected_output)));
    }

    #[test]
    fn test_parse_texture_coordinates() {
        let input = CompleteStr("vt 0.333134 0.000200\n");
        let expected_remainder = CompleteStr("");

        match texture_coordinates(input) {
            Ok((remainder, v)) => {
                assert_eq!(remainder, expected_remainder);
                assert_eq!(v.x, 0.333134);
                assert_eq!(v.y, 0.000200);
                assert_eq!(v.z, 0.0);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_vertex_normal() {
        let input = CompleteStr("vn 0.0000 1.0000 0.0000\n");
        let expected_remainder = CompleteStr("");

        match vertex_normal(input) {
            Ok((remainder, v)) => {
                assert_eq!(remainder, expected_remainder);
                assert_eq!(v.x, 0.0);
                assert_eq!(v.y, 1.0);
                assert_eq!(v.z, 0.0);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_vertex_normal_list() {
        let input = CompleteStr("vn 0.0000 1.0000 0.0000\nvn 0.0000 0.0000 1.0000\nvn -1.0000 0.0000 0.0000\n\n");
        let expected_remainder = CompleteStr("");

        match vertex_normal_list(input) {
            Ok((remainder, vertex_normals)) => {
                assert_eq!(remainder, expected_remainder);
                assert_eq!(vertex_normals.len(), 3);
                assert_eq!(vertex_normals[0].x, 0.0);
                assert_eq!(vertex_normals[0].y, 1.0);
                assert_eq!(vertex_normals[0].z, 0.0);
                assert_eq!(vertex_normals[1].x, 0.0);
                assert_eq!(vertex_normals[1].y, 0.0);
                assert_eq!(vertex_normals[1].z, 1.0);
                assert_eq!(vertex_normals[2].x, -1.0);
                assert_eq!(vertex_normals[2].y, 0.0);
                assert_eq!(vertex_normals[2].z, 0.0);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_face_index() {
        let input = CompleteStr("1/16/10005 ");
        let expected_remainder = CompleteStr(" ");

        assert_eq!(face_index(input), Ok((expected_remainder, (1, Some(16), 10005))));
    }

    #[test]
    fn test_parse_face() {
        let input = CompleteStr("f 5/1/1 3/2/1 1/3/1\n");
        let expected_remainder = CompleteStr("");

        match face(input) {
            Ok((remainder, face)) => {
                assert_eq!(remainder, expected_remainder);
                assert_eq!(face.vertexes, [5, 3, 1]);
                assert_eq!(face.texture_coordinates, [Some(1), Some(2), Some(3)]);
                assert_eq!(face.vertex_normals, [1, 1, 1]);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_face_missing_texture_coordinates() {
        let input = CompleteStr("f 5//1 3//1 1//1\n");
        let expected_remainder = CompleteStr("");

        match face(input) {
            Ok((remainder, face)) => {
                assert_eq!(remainder, expected_remainder);
                assert_eq!(face.vertexes, [5, 3, 1]);
                assert_eq!(face.texture_coordinates, [None, None, None]);
                assert_eq!(face.vertex_normals, [1, 1, 1]);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_usemtl() {
        let input = CompleteStr("usemtl Material\n");
        let expected_remainder = CompleteStr("");
        let expected_output = CompleteStr("Material");

        assert_eq!(usemtl(input), Ok((expected_remainder, expected_output)));
    }

    #[test]
    fn test_parse_material_file() {
        let input = CompleteStr("mtllib cube_uv.mtl\n");
        let expected_remainder = CompleteStr("");
        let expected_output = CompleteStr("cube_uv.mtl");

        assert_eq!(material_file(input), Ok((expected_remainder, expected_output)))
    }

    #[test]
    fn test_parse_smooth_shading() {
        let input = CompleteStr("s off\n");
        let expected_remainder = CompleteStr("");

        assert_eq!(smooth_shading(input), Ok((expected_remainder, false)));

        let input = CompleteStr("s on\n");
        assert_eq!(smooth_shading(input), Ok((expected_remainder, true)));
    }

    #[test]
    fn test_parse_obj_file() {
        let s = include_str!("../assets/cube_uv.obj");

        let model = parse_obj_file(s);

        assert_eq!(model.name, "Cube");
        assert_eq!(model.vertices.len(), 8);
        //assert_eq!(model.triangles.len(), 12 * 3);
    }
}
