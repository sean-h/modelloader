# modelloader

modelloader is a Rust library for parsing 3D model files.

## Supported Formats
- .obj

## Dependencies
- [nom](https://github.com/Geal/nom)
- [tdmath](https://github.com/sean-h/tdmath)

## Example
```
use std::fs::File;
use std::io::prelude::*;
use modelloader::*;

fn main() {
    let mut f = File::open("model.obj").expect("Unable to open file");
    let mut file_contents = String::new();
    f.read_to_string(&mut file_contents).expect("Unable to read file");

    let model = parse_obj_file(&file_contents);

    for v in &model.vertices {
        println!("Position: {} {} {} UV: {} {}", v.p.x, v.p.y, v.p.z, v.uv.x, v.uv.y);
    }
}
```