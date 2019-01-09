extern crate tdmath;

use nom::*;
use tdmath::Vector3;

/*
    Basic Parsers
*/

named!(space<&str, &str>,
    tag!(" ")
);

fn is_space(c: char) -> bool {
    c == ' '
}

named!(spaces<&str, &str>,
    take_while1!(is_space)
);

named!(name<&str, &str>,
    take_until!("\n")
);

named!(filename<&str, &str>,
    take_until!("\n")
);

named!(line_end<&str, &str>,
    preceded!(
        opt!(spaces),
        alt!(tag!("\n") | comment)
    )
);

/*
    Comments
*/

named!(comment<&str, &str>,
    do_parse!(
        tag!("#") >>
        comment: take_until!("\n") >>
        tag!("\n") >>

        (comment)
    )
);

/*
    Object Name
*/

named!(object_name_specifier<&str, &str>,
    tag!("o")
);

named!(object_name<&str, &str>,
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

named!(vertex<&str, Vector3>,
    do_parse!(
        tag!("v") >>
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

/*
    Texture Coordinates
*/

named!(texture_coordinates<&str, Vector3>,
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

/*
    Vertex Normals
*/

named!(vertex_normal<&str, Vector3>,
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

/*
    Materials
*/

named!(material_file<&str, &str>,
    do_parse!(
        tag!("mtllib") >>
        spaces >>
        name: filename >>
        line_end >>

        (name)
    )
);

named!(usemtl<&str, &str>,
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

fn str_to_bool(s: &str) -> Result<bool, &str> {
    if s == "on" {
        Ok(true)
    } else if s == "off" {
        Ok(false)
    } else {
        Err("Cannot convert string to bool")
    }
}

named!(smooth_shading<&str, bool>,
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

named!(face_index<&str, (usize, Option<usize>, usize)>,
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

named!(face<&str, FaceIndexed>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_object_name_specifier() {
        assert_eq!(object_name_specifier("o cube"), Ok((" cube", "o")));
    }

    #[test]
    fn test_parse_name() {
        assert_eq!(name("cube\n"), Ok(("\n", "cube")));
    }

    #[test]
    fn test_parse_object_name() {
        assert_eq!(object_name("o cube\n"), Ok(("", "cube")));
    }

    #[test]
    fn test_parse_vertex() {
        match vertex("v 1.000000 1.000000 -1.000000\n") {
            Ok((remainder, v)) => {
                assert_eq!(remainder, "");
                assert_eq!(v.x, 1.0);
                assert_eq!(v.y, 1.0);
                assert_eq!(v.z, -1.0);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_vertex_with_comment() {
        match vertex("v 1.000000 1.000000 -1.000000 #Vertex 1\n") {
            Ok((remainder, v)) => {
                assert_eq!(remainder, "");
                assert_eq!(v.x, 1.0);
                assert_eq!(v.y, 1.0);
                assert_eq!(v.z, -1.0);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_comment() {
        assert_eq!(comment("#this is a comment\n"), Ok(("", "this is a comment")));
    }

    #[test]
    fn test_parse_spaces() {
        assert_eq!(spaces("   spaces  "), Ok(("spaces  ", "   ")));
    }

    #[test]
    fn test_parse_texture_coordinates() {
        match texture_coordinates("vt 0.333134 0.000200\n") {
            Ok((remainder, v)) => {
                assert_eq!(remainder, "");
                assert_eq!(v.x, 0.333134);
                assert_eq!(v.y, 0.000200);
                assert_eq!(v.z, 0.0);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_vertex_normal() {
        match vertex_normal("vn 0.0000 1.0000 0.0000\n") {
            Ok((remainder, v)) => {
                assert_eq!(remainder, "");
                assert_eq!(v.x, 0.0);
                assert_eq!(v.y, 1.0);
                assert_eq!(v.z, 0.0);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_face_index() {
        assert_eq!(face_index("1/16/10005 "), Ok((" ", (1, Some(16), 10005))));
    }

    #[test]
    fn test_parse_face() {
        match face("f 5/1/1 3/2/1 1/3/1\n") {
            Ok((remainder, face)) => {
                assert_eq!(remainder, "");
                assert_eq!(face.vertexes, [5, 3, 1]);
                assert_eq!(face.texture_coordinates, [Some(1), Some(2), Some(3)]);
                assert_eq!(face.vertex_normals, [1, 1, 1]);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_face_missing_texture_coordinates() {
        match face("f 5//1 3//1 1//1\n") {
            Ok((remainder, face)) => {
                assert_eq!(remainder, "");
                assert_eq!(face.vertexes, [5, 3, 1]);
                assert_eq!(face.texture_coordinates, [None, None, None]);
                assert_eq!(face.vertex_normals, [1, 1, 1]);
            },
            Err(err) => panic!(err)
        }
    }

    #[test]
    fn test_parse_usemtl() {
        assert_eq!(usemtl("usemtl Material\n"), Ok(("", "Material")));
    }

    #[test]
    fn test_parse_material_file() {
        assert_eq!(material_file("mtllib cube_uv.mtl\n"), Ok(("", "cube_uv.mtl")))
    }

    #[test]
    fn test_parse_smooth_shading() {
        assert_eq!(smooth_shading("s off\n"), Ok(("", false)));
        assert_eq!(smooth_shading("s on\n"), Ok(("", true)));
    }
}
