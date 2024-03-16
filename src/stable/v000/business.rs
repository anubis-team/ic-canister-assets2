use super::super::business::*;
use super::types::*;

#[allow(clippy::panic)] // ? SAFETY
#[allow(unused_variables)]
impl Business for InnerState {
    fn business_files(&self) -> Vec<crate::stable::QueryFile> {
        panic!("Not supported operation by this version.")
    }
    fn business_download(&self, path: String) -> Vec<u8> {
        panic!("Not supported operation by this version.")
    }
    fn business_download_by(&self, path: String, offset: u64, offset_end: u64) -> Vec<u8> {
        panic!("Not supported operation by this version.")
    }

    fn business_upload(&mut self, args: Vec<crate::stable::UploadingArg>) {
        panic!("Not supported operation by this version.")
    }

    fn business_delete(&mut self, names: Vec<String>) {
        panic!("Not supported operation by this version.")
    }

    fn business_assets_files(&self) -> &HashMap<String, crate::stable::AssetFile> {
        panic!("Not supported operation by this version.")
    }
    fn business_assets_assets(
        &self,
    ) -> &HashMap<crate::stable::HashDigest, crate::stable::AssetData> {
        panic!("Not supported operation by this version.")
    }
}
