use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref HASHMAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("1.2.840.113549.1.1.11", "sha256WithRSAEncryption");
        m.insert("1.2.840.113549.1.12.1.3", "pbeWithSHAAnd3-KeyTripleDES-CBC");
        m.insert("1.2.840.113549.1.12.10.1.2", "pkcs-8ShroudedKeyBag");
        m.insert("1.2.840.113549.1.12.10.1.3", "pkcs-12-certBag (PKCS #12 BagIds)");
        m.insert("1.2.840.113549.1.7.1", "data (PKCS #7)");
        m.insert("1.2.840.113549.1.7.6", "encryptedData");
        m.insert("1.2.840.113549.1.9.20", "friendlyName");
        m.insert("1.2.840.113549.1.9.21", "localKeyID");
        m.insert("1.2.840.113549.1.9.22.1", "x509Certificate (for PKCS #12)");
        m.insert("1.3.14.3.2.26", "sha1");
        m.insert("2.5.4.10", "organizationName");
        m.insert("2.5.4.6", "countryName");
        m.insert("2.5.4.8", "stateOrProvinceName");
        return m;
    };
}

pub fn find(oid: &str) -> Option<&'static str> {
    return HASHMAP.get(oid).copied();
}
