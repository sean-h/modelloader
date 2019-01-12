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

fn is_name_char(c: char) -> bool {
    c.is_alphabetic() || c.is_digit(10) || c == '.' || c == '_'
}

named!(name<CompleteStr, CompleteStr>,
    take_while1!(is_name_char)
);

named!(filename<CompleteStr, CompleteStr>,
    take_while1!(is_name_char)
);

named!(line_end<CompleteStr, CompleteStr>,
    preceded!(
        opt!(spaces),
        alt!(line_ending | comment)
    )
);

named!(empty_line<CompleteStr, CompleteStr>,
    preceded!(
        opt!(spaces),
        line_ending
    )
);

/*
    Comments
*/

named!(comment<CompleteStr, CompleteStr>,
    do_parse!(
        tag!("#") >>
        comment: take_until_either!("\r\n") >>
        opt!(tag!("\r")) >>
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

named!(object_name<CompleteStr, Option<CompleteStr>>,
    opt!(
        do_parse!(
            opt!(many0!(line_end)) >>
            opt!(spaces) >>
            tag!("o") >>
            spaces >>
            n: name >>
            line_end >>

            (n)
        )
    )
);

/*
    Vertex
*/

named!(vertex<CompleteStr, Vector3>,
    do_parse!(
        opt!(many0!(line_end)) >>
        opt!(spaces) >>
        tag!("v ") >>
        opt!(spaces) >>
        x: float >>
        spaces >>
        y: float >>
        spaces >>
        z: float >>
        line_end >>

        (Vector3::new(x, y, z))
    )
);

named!(vertex_list<CompleteStr, Vec<Vector3>>,
    do_parse!(
        opt!(many0!(ignore_line)) >>
        v: many0!(vertex) >>

        (v)
    )
);

/*
    Texture Coordinates
*/

named!(texture_coordinates<CompleteStr, Vector3>,
    do_parse!(
        opt!(many0!(line_end)) >>
        opt!(spaces) >>
        tag!("vt") >>
        spaces >>
        x: float >>
        spaces >>
        y: float >>
        opt!(space) >>
        opt!(float) >>
        line_end >>

        (Vector3::new(x, y, 0.0))
    )
);

named!(texture_coordinate_list<CompleteStr, Vec<Vector3>>,
    do_parse!(
        opt!(many0!(ignore_line)) >>
        uv: many0!(texture_coordinates) >>

        (uv)
    )
);

/*
    Vertex Normals
*/

named!(vertex_normal<CompleteStr, Vector3>,
    do_parse!(
        opt!(many0!(line_end)) >>
        opt!(spaces) >>
        tag!("vn") >>
        spaces >>
        x: float >>
        spaces >>
        y: float >>
        spaces >>
        z: float >>
        line_end >>

        (Vector3::new(x, y, z))
    )
);

named!(vertex_normal_list<CompleteStr, Vec<Vector3>>,
    do_parse!(
        opt!(many0!(ignore_line)) >>
        vn: many0!(vertex_normal) >>

        (vn)
    )
);

/*
    Materials
*/

named!(material_file<CompleteStr, Option<CompleteStr>>,
    opt!(
        do_parse!(
            opt!(many0!(line_end)) >>
            opt!(spaces) >>
            tag!("mtllib") >>
            spaces >>
            name: filename >>
            line_end >>

            (name)
        )
    )
);

named!(usemtl<CompleteStr, Option<CompleteStr>>,
    opt!(
        do_parse!(
            opt!(many0!(line_end)) >>
            opt!(spaces) >>
            tag!("usemtl") >>
            spaces >>
            name: name >>
            line_end >>

            (name)
        )
    )
);

/*
    Smooth Shading
*/

fn str_to_bool(s: CompleteStr) -> Result<bool, CompleteStr> {
    if s == CompleteStr("on") || s == CompleteStr("1") {
        Ok(true)
    } else if s == CompleteStr("off") || s == CompleteStr("0") {
        Ok(false)
    } else {
        Err(CompleteStr("Cannot convert string to bool"))
    }
}

named!(smooth_shading<CompleteStr, Option<bool>>,
    opt!(
        do_parse!(
            opt!(many0!(line_end)) >>
            opt!(spaces) >>
            tag!("s") >>
            spaces >>
            b: map_res!(take_until!("\n"), str_to_bool) >>
            line_end >>

            (b)
        )
    )
);

/*
    Polygon Group
*/

named!(polygon_group<CompleteStr, Option<CompleteStr>>,
    opt!(
        do_parse!(
            opt!(many0!(line_end)) >>
            opt!(spaces) >>
            tag!("g") >>
            spaces >>
            n: name >>
            line_end >>

            (n)
        )
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
        opt!(many0!(line_end)) >>
        opt!(spaces) >>
        tag!("f") >>
        spaces >>
        i1: face_index >>
        spaces >>
        i2: face_index >>
        spaces >>
        i3: face_index >>
        line_end >>

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

        (f)
    )
);

fn discard_comments(data: CompleteStr) -> CompleteStr {
    match ignore_lines(data) {
        Ok((remainder, _)) => remainder,
        Err(_) => panic!("Unable to parse OBJ file: error reading leading comments")
    }
}

pub fn parse_obj_file(data: &str) -> Model {
    // Leading comments
    let remainder = discard_comments(CompleteStr(data));

    let (remainder, _) = match material_file(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading material file")
    };

    let (remainder, obj_name) = match object_name(remainder) {
        Ok((remainder, obj_name)) => {
            match obj_name {
                Some(x) => (remainder, x),
                None => (remainder, CompleteStr("Object"))
            }
        },
        Err(_) => panic!("Unable to parse OBJ file: error reading object name")
    };
    
    let (remainder, vertex_positions) = match vertex_list(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading vertex positions")
    };

    let (remainder, uvs) = match texture_coordinate_list(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading UV coordinates")
    };

    let (remainder, _) = match vertex_normal_list(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading vertex normals")
    };

    let (remainder, _) = match usemtl(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading usemtl")
    };

    // Parse 1 polygon group at the start of the face list. Ignore the polygon group.
    let (remainder, _) = match polygon_group(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading polygon group")
    };

    let (remainder, _) = match smooth_shading(remainder) {
        Ok(x) => x,
        Err(_) => panic!("Unable to parse OBJ file: error reading smooth shading")
    };

    let (_, faces) = match face_list(remainder) {
        Ok(f) => f,
        Err(_) => panic!("Unable to parse OBJ file: error reading faces")
    };

    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for f in faces {
        for i in 0..3 {
            let p = vertex_positions[f.vertexes[i] - 1];
            let uv = match f.texture_coordinates[i] {
                Some(index) => uvs[index - 1],
                None => Vector3::zero()
            };
            let v = Vertex {
                p,
                uv,
            };
            triangles.push(vertices.len());
            vertices.push(v);
        }
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
        let expected_output = Some(CompleteStr("cube"));
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
    fn test_parse_vertex_with_integer_dimension() {
        let input = CompleteStr("v 1.000000 1 -1.000000\n");
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
    fn test_parse_vertex_crlf() {
        let input = CompleteStr("v 1.000000 1.000000 -1.000000\r\n");
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
    fn test_parse_vertex_list_crlf() {
        let input = CompleteStr("v 1.000000 1.000000 -1.000000\r\nv 1.000000 -1.000000 -1.000000\r\nv 1.000000 1.000000 1.000000\r\n");
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
    fn test_parse_vertex_list_with_following_texture_coordinates_crlf() {
        let input = CompleteStr("v 1.000000 1.000000 -1.000000\r\nv 1.000000 -1.000000 -1.000000\r\nv 1.000000 1.000000 1.000000\r\nvt 0.333134 0.000200\r\n");
        let expected_remainder = CompleteStr("vt 0.333134 0.000200\r\n");

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
    fn test_parse_comment_crlf() {
        let input = CompleteStr("#this is a comment\r\nNext Line");
        let expected_remainder = CompleteStr("Next Line");
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
    fn test_parse_texture_coordinates_with_three_dimensions() {
        let input = CompleteStr("vt 0.333134 0.000200 0.000\n");
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
    fn test_parse_texture_coordinates_multiple_leading_spaces() {
        let input = CompleteStr("vt   0.333134 0.000200\n");
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
        let input = CompleteStr("vn 0.0000 1.0000 0.0000\nvn 0.0000 0.0000 1.0000\nvn -1.0000 0.0000 0.0000\n");
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
        let expected_output = Some(CompleteStr("Material"));

        assert_eq!(usemtl(input), Ok((expected_remainder, expected_output)));
    }

    #[test]
    fn test_parse_material_file() {
        let input = CompleteStr("mtllib cube_uv.mtl\n");
        let expected_remainder = CompleteStr("");
        let expected_output = Some(CompleteStr("cube_uv.mtl"));

        assert_eq!(material_file(input), Ok((expected_remainder, expected_output)))
    }

    #[test]
    fn test_parse_smooth_shading() {
        let input = CompleteStr("s off\n");
        let expected_remainder = CompleteStr("");

        assert_eq!(smooth_shading(input), Ok((expected_remainder, Some(false))));

        let input = CompleteStr("s on\n");
        assert_eq!(smooth_shading(input), Ok((expected_remainder, Some(true))));
    }

    #[test]
    fn test_parse_polygon_group() {
        let input = CompleteStr("g group1\n");
        let expected_remainder = CompleteStr("");
        let expected_output = Some(CompleteStr("group1"));
        assert_eq!(polygon_group(input), Ok((expected_remainder, expected_output)));
    }

    #[test]
    fn test_parse_obj_file() {
        let s = include_str!("../assets/cube_uv.obj");

        let model = parse_obj_file(s);

        assert_eq!(model.name, "Cube");

        assert_eq!(model.vertices.len(), 12 * 3);
        assert_eq!(model.vertices[0].p.x, -1.0);
        assert_eq!(model.vertices[0].p.y, 1.0);
        assert_eq!(model.vertices[0].p.z, -1.0);
        assert_eq!(model.vertices[6].p.x, -1.0);
        assert_eq!(model.vertices[6].p.y, 1.0);
        assert_eq!(model.vertices[6].p.z, 1.0);
        assert_eq!(model.vertices[35].p.x, 1.0);
        assert_eq!(model.vertices[35].p.y, -1.0);
        assert_eq!(model.vertices[35].p.z, -1.0);

        assert_eq!(model.triangles.len(), 12 * 3);
        assert_eq!(model.triangles[0], 0);
        assert_eq!(model.triangles[1], 1);
        assert_eq!(model.triangles[2], 2);
        assert_eq!(model.triangles[35], 35);
    }

    #[test]
    fn test_parse_obj_file_stripped() {
        let s = include_str!("../assets/cube_stripped.obj");

        let model = parse_obj_file(s);

        assert_eq!(model.name, "Object");

        assert_eq!(model.vertices.len(), 12 * 3);
        assert_eq!(model.vertices[0].p.x, -1.0);
        assert_eq!(model.vertices[0].p.y, 1.0);
        assert_eq!(model.vertices[0].p.z, -1.0);
        assert_eq!(model.vertices[6].p.x, -1.0);
        assert_eq!(model.vertices[6].p.y, 1.0);
        assert_eq!(model.vertices[6].p.z, 1.0);
        assert_eq!(model.vertices[35].p.x, 1.0);
        assert_eq!(model.vertices[35].p.y, -1.0);
        assert_eq!(model.vertices[35].p.z, -1.0);

        assert_eq!(model.triangles.len(), 12 * 3);
        assert_eq!(model.triangles[0], 0);
        assert_eq!(model.triangles[1], 1);
        assert_eq!(model.triangles[2], 2);
        assert_eq!(model.triangles[35], 35);
    }

    #[test]
    fn test_parse_obj_file_commented() {
        let s = include_str!("../assets/cube_commented.obj");

        let model = parse_obj_file(s);

        assert_eq!(model.name, "Object");

        assert_eq!(model.vertices.len(), 12 * 3);
        assert_eq!(model.vertices[0].p.x, -1.0);
        assert_eq!(model.vertices[0].p.y, 1.0);
        assert_eq!(model.vertices[0].p.z, -1.0);
        assert_eq!(model.vertices[6].p.x, -1.0);
        assert_eq!(model.vertices[6].p.y, 1.0);
        assert_eq!(model.vertices[6].p.z, 1.0);
        assert_eq!(model.vertices[35].p.x, 1.0);
        assert_eq!(model.vertices[35].p.y, -1.0);
        assert_eq!(model.vertices[35].p.z, -1.0);

        assert_eq!(model.triangles.len(), 12 * 3);
        assert_eq!(model.triangles[0], 0);
        assert_eq!(model.triangles[1], 1);
        assert_eq!(model.triangles[2], 2);
        assert_eq!(model.triangles[35], 35);
    }

    #[test]
    fn test_parse_obj_file_polygon_groups() {
        let s = include_str!("../assets/cube_polygon_groups.obj");

        let model = parse_obj_file(s);

        assert_eq!(model.name, "Cube");

        assert_eq!(model.vertices.len(), 12 * 3);
        assert_eq!(model.vertices[0].p.x, -1.0);
        assert_eq!(model.vertices[0].p.y, 1.0);
        assert_eq!(model.vertices[0].p.z, -1.0);
        assert_eq!(model.vertices[6].p.x, -1.0);
        assert_eq!(model.vertices[6].p.y, 1.0);
        assert_eq!(model.vertices[6].p.z, 1.0);
        assert_eq!(model.vertices[35].p.x, 1.0);
        assert_eq!(model.vertices[35].p.y, -1.0);
        assert_eq!(model.vertices[35].p.z, -1.0);

        assert_eq!(model.triangles.len(), 12 * 3);
        assert_eq!(model.triangles[0], 0);
        assert_eq!(model.triangles[1], 1);
        assert_eq!(model.triangles[2], 2);
        assert_eq!(model.triangles[35], 35);
    }
}
