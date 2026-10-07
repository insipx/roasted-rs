use cfg_aliases::cfg_aliases;

fn main() {
    println!("cargo:rustc-link-arg=-Tlinkall.x");

    // https://docs.rs/cfg_aliases/latest/cfg_aliases
    cfg_aliases! {
        bare_metal: { not(feature = "qemu") },
        emulated: {feature = "qemu" }
    }
}
