use crate::settings::UploadSettings;
use crate::uploader::UploadBuilder;
use failure::{err_msg, Error};
use url::Url;

pub fn go(settings: &UploadSettings, builder: &mut UploadBuilder) -> Result<Url, Error> {
    let uploader = builder.build().map_err(err_msg)?;
    uploader.upload_file(settings.file.clone())
}
