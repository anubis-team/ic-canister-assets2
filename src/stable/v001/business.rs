use super::super::business::*;
use super::types::*;

#[allow(unused_variables)]
impl Business for InnerState {
    fn business_files(&self) -> Vec<QueryFile> {
        self.business.assets.files()
    }
    fn business_download(&self, path: String) -> Vec<u8> {
        self.business.assets.download(path)
    }
    fn business_download_by(&self, path: String, offset: u64, offset_end: u64) -> Vec<u8> {
        self.business.assets.download_by(path, offset, offset_end)
    }

    fn business_upload(&mut self, args: Vec<UploadingArg>) {
        for arg in args {
            // 1. 先暂存本次内容
            let done = self.business.uploading.put(arg);

            if let Some(file) = done {
                // 2. 如果完成了, 需要升级本文件
                self.business.assets.put(file);

                // 3. 删除这个文件
                let path = file.path.clone();
                self.business.uploading.clean(&path);
            }
        }
    }

    fn business_delete(&mut self, names: Vec<String>) {
        for name in names {
            self.business.uploading.clean(&name);
            self.business.assets.clean(&name);
        }
    }

    fn business_assets_files(&self) -> &HashMap<String, AssetFile> {
        &self.business.assets.files
    }
    fn business_assets_assets(&self) -> &HashMap<HashDigest, AssetData> {
        &self.business.assets.assets
    }
}
