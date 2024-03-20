use super::super::business::*;
use super::types::*;

#[allow(unused_variables)]
impl Business for InnerState {
    fn business_hashed_find(&self) -> bool {
        self.heap_state.business.hashed
    }
    fn business_files(&self) -> Vec<QueryFile> {
        self.files()
    }
    fn business_download(&self, path: String) -> Vec<u8> {
        self.download(path)
    }
    fn business_download_by(&self, path: String, offset: u64, size: u64) -> Vec<u8> {
        self.download_by(path, offset, size)
    }

    fn business_hashed_update(&mut self, hashed: bool) {
        self.heap_state.business.hashed = hashed;
    }
    fn business_upload(&mut self, args: Vec<UploadingArg>) {
        for arg in args {
            self.put_uploading(arg)
        }
    }
    fn business_delete(&mut self, names: Vec<String>) {
        for name in names {
            self.clean_uploading(&name);
            self.clean_file(&name);
        }
    }

    fn business_assets_get_file(&self, path: &str) -> Option<&AssetFile> {
        self.heap_state.business.files.get(path)
    }
    fn business_assets_get(&self, hash: &HashDigest) -> Option<&AssetData> {
        self.heap_state.business.assets.get(hash)
    }
}
