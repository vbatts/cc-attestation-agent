// Copyright (c) 2021 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

use serde::{Deserialize, Serialize};

// Add your specific kbc declaration here.
// For example: "pub mod sample_kbc;"
#[allow(dead_code)]
#[cfg(feature = "cc_kbc")]
pub mod cc_kbc;

#[cfg(feature = "eaa_kbc")]
pub mod eaa_kbc;

#[cfg(feature = "offline_fs_kbc")]
pub mod offline_fs_kbc;

#[cfg(feature = "offline_sev_kbc")]
pub mod offline_sev_kbc;

#[cfg(feature = "online_sev_kbc")]
pub mod online_sev_kbc;

#[cfg(feature = "sample_kbc")]
pub mod sample_kbc;

use anyhow::*;
use async_trait::async_trait;
use std::collections::HashMap;

// KbcInterface is a standard interface that all KBC modules need to implement.
#[async_trait]
pub trait KbcInterface: Send {
    fn check(&self) -> Result<KbcCheckInfo>;
    async fn decrypt_payload(&mut self, annotation: &str) -> Result<Vec<u8>>;
    async fn get_resource(&mut self, _description: String) -> Result<Vec<u8>> {
        Err(anyhow!("Get Resource API of this KBC is unimplement!"))
    }
}

pub type KbcInstance = Box<dyn KbcInterface + Sync + Send>;
type KbcInstantiateFunc = Box<dyn Fn(String) -> KbcInstance + Send + Sync>;

// KbcCheckInfo is used by KBC module instances to report their internal status to AA.
pub struct KbcCheckInfo {
    pub kbs_info: HashMap<String, String>,
    // In the future, more KBC status fields will be expanded here.
}

pub struct KbcModuleList {
    mod_list: HashMap<String, KbcInstantiateFunc>,
}

impl KbcModuleList {
    pub fn new() -> KbcModuleList {
        let mut mod_list = HashMap::new();

        #[cfg(feature = "sample_kbc")]
        {
            let instantiate_func: KbcInstantiateFunc = Box::new(|kbs_uri: String| -> KbcInstance {
                Box::new(sample_kbc::SampleKbc::new(kbs_uri))
            });
            mod_list.insert("sample_kbc".to_string(), instantiate_func);
        }

        #[cfg(feature = "cc_kbc")]
        {
            let instantiate_func: KbcInstantiateFunc =
                Box::new(|kbs_uri: String| -> KbcInstance { Box::new(cc_kbc::Kbc::new(kbs_uri)) });
            mod_list.insert("cc_kbc".to_string(), instantiate_func);
        }

        #[cfg(feature = "offline_fs_kbc")]
        {
            let instantiate_func: KbcInstantiateFunc = Box::new(|_: String| -> KbcInstance {
                Box::new(offline_fs_kbc::OfflineFsKbc::new())
            });
            mod_list.insert("offline_fs_kbc".to_string(), instantiate_func);
        }

        #[cfg(feature = "eaa_kbc")]
        {
            let instantiate_func: KbcInstantiateFunc = Box::new(|kbs_uri: String| -> KbcInstance {
                Box::new(eaa_kbc::EAAKbc::new(kbs_uri))
            });
            mod_list.insert("eaa_kbc".to_string(), instantiate_func);
        }

        #[cfg(feature = "offline_sev_kbc")]
        {
            let instantiate_func: KbcInstantiateFunc = Box::new(|_: String| -> KbcInstance {
                Box::new(offline_sev_kbc::OfflineSevKbc::new())
            });
            mod_list.insert("offline_sev_kbc".to_string(), instantiate_func);
        }

        #[cfg(feature = "online_sev_kbc")]
        {
            let instantiate_func: KbcInstantiateFunc = Box::new(|kbs_uri: String| -> KbcInstance {
                Box::new(online_sev_kbc::OnlineSevKbc::new(kbs_uri))
            });
            mod_list.insert("online_sev_kbc".to_string(), instantiate_func);
        }

        KbcModuleList { mod_list }
    }

    pub fn get_func(&self, kbc_name: &str) -> Result<&KbcInstantiateFunc> {
        let instantiate_func: &KbcInstantiateFunc =
            self.mod_list.get(kbc_name).ok_or_else(|| {
                anyhow!(
                    "AA does not support the given KBC module! Module: {}",
                    kbc_name
                )
            })?;
        Ok(instantiate_func)
    }
}

#[derive(EnumString, Display, Debug, PartialEq)]
pub enum ResourceName {
    #[strum(serialize = "Policy")]
    Policy,
    #[strum(serialize = "Sigstore Config")]
    SigstoreConfig,
    #[strum(serialize = "GPG Keyring")]
    GPGPublicKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceDescription {
    name: String,
    optional: HashMap<String, String>,
}
