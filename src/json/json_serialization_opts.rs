pub struct JsonSerializationOpts {
    pub pubkey_as_base58: bool,
    pub n64_as_string: bool,
    pub n128_as_string: bool,
}

impl Default for JsonSerializationOpts {
    fn default() -> Self {
        Self {
            pubkey_as_base58: true,
            n64_as_string: false,
            n128_as_string: false,
        }
    }
}
