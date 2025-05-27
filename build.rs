#[cfg(windows)]

fn main() {
    #[cfg(windows)]
    {
        use winres::WindowsResource;
        
        let mut res = WindowsResource::new();
        res.set_icon("src/icon.ico")
           .set_language(0x0409) // English (US)
           .set("ProductName", "NitroKit")
           .set("FileDescription", "NitroKit Terminal Tool")
           .set("LegalCopyright", "Copyright (C) 2025 Mustafa Genc")
           .set("ProductVersion", env!("CARGO_PKG_VERSION"))
           .set("FileVersion", env!("CARGO_PKG_VERSION"));
        
        if let Err(e) = res.compile() {
            println!("cargo:warning=Failed to compile Windows resources: {}", e);
        }
    }
    
    println!("cargo:rerun-if-changed=src/icon.ico");
}