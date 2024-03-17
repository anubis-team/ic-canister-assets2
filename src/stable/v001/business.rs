use super::super::business::*;
use super::types::*;

#[allow(unused_variables)]
impl Business for InnerState {
    fn business_files(&self) -> Vec<QueryFile> {
        self.business.files()
    }
    fn business_download(&self, path: String) -> Vec<u8> {
        self.business.download(path)
    }
    fn business_download_by(&self, path: String, offset: u64, offset_end: u64) -> Vec<u8> {
        self.business.download_by(path, offset, offset_end)
    }

    fn business_upload(&mut self, args: Vec<UploadingArg>) {
        for arg in args {
            self.business.put_uploading(arg)
        }
    }

    fn business_delete(&mut self, names: Vec<String>) {
        for name in names {
            self.business.clean_uploading(&name);
            self.business.clean_file(&name);
        }
    }

    fn business_assets_files(&self) -> &HashMap<String, AssetFile> {
        &self.business.files
    }
    fn business_assets_assets(&self) -> &HashMap<HashDigest, AssetData> {
        &self.business.assets
    }
}
