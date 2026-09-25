use wtransport::ClientConfig;

pub fn configure_client(insecure: bool) -> ClientConfig {
    let builder = ClientConfig::builder().with_bind_default();

    if insecure {
        builder.with_no_cert_validation().build()
    } else {
        builder.with_native_certs().build()
    }
}
