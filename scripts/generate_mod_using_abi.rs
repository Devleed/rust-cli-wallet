fn generate_abi_interfaces() {
    let abi_source = "abis/erc20.json";
    let out_file = std::env::temp_dir().join("ierc20.rs");
    if out_file.exists() {
        std::fs::remove_file(&out_file).unwrap();
    }
    Abigen::new("IERC20", abi_source)
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file("src/contracts/ierc20.rs")
        .expect("Failed to generate inteface from ABI");
}
