use namada_sdk::key::common::PublicKey as NamadaPublicKey;

pub struct PublicKey(pub String);

impl From<NamadaPublicKey> for PublicKey {
    fn from(pk: NamadaPublicKey) -> Self {
        PublicKey(pk.to_string())
    }
}
