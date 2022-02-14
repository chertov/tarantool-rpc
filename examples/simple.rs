fn main() {
    let cargo_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut output_path =  cargo_path.clone();
    output_path += "/examples/";
    let mut traits_path =  output_path.clone();
    traits_path += "traits.yaml";

    println!("traits.yaml: {traits_path}");
    println!("output_path: {output_path}");

    tarantool_rpc::generate(
        &traits_path,
        &output_path,
        "my-simple-rpc",
        None
    ).unwrap();
}
