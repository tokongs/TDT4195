mod my_format;
mod wavefront;
mod mesh;

use std::env;

/*
   Helping program to convert .obj files to .myf files. The format is a lot faster for loading indexed vertices.
*/
fn main(){
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let mesh = unsafe { wavefront::load(&args[1]) };
    my_format::write(&args[2], mesh);

}