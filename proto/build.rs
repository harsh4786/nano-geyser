use tonic_build::configure;

fn main() {
    configure()
        .compile(
            &[
                "proto/nano_geyser.proto",
            ],
            &["proto"],
        )
        .unwrap();
}