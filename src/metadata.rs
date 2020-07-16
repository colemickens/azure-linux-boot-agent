extern crate reqwest;
extern crate serde;
extern crate serde_xml_rs;
extern crate tokio;

pub mod instance {
    pub mod compute {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        pub struct PublicKey {
            #[serde(alias = "keyData")]
            pub key_data: String,
            pub path: String,
        }
    }
}
