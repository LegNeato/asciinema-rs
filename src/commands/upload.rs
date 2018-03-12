use failure::{err_msg, Error};
use settings::UploadSettings;
use settings::install::InstallInfo;
use uploader::UploadBuilder;
use url::Url;

pub fn go(settings: UploadSettings, builder: &mut UploadBuilder) -> Result<Url, Error> {
    // Load install id from a file or generate a new one.
    // Note: the reference python version doesn't fail when
    // there is no existing install id, so we don't either.
    let install_info = InstallInfo::new()?;

    let id = install_info.id;

    // Persist the install id to a file.
    // Note: the reference python version persists the install id
    // so we do as well.
    if !install_info.is_saved {
        install_info.save()?;
    }

    let uploader = builder.install_id(id).build().map_err(err_msg)?;

    uploader.upload_file(settings.file)
}
