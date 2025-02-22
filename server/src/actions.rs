use std::path::Path;

pub(crate) async fn list_files(data: Vec<u8>, base: &Path) -> Option<Vec<Vec<u8>>> {
    let path: String = String::from_utf8(data).ok()?;
    match std::fs::read_dir(base.join(path)) {
        Ok(r) => {
            let list: Vec<Vec<u8>> = r
                .filter_map(|x| match x {
                    Ok(l) => Some(l.file_name().as_encoded_bytes().to_owned()),
                    Err(e) => {
                        println!("{}", e);
                        None
                    }
                })
                .collect();
            Some(list)
        }
        Err(e) => {
            println!("{}", e);
            None
        }
    }
}
pub(crate) async fn file_exists(data: Vec<u8>, base: &Path) -> Option<Vec<Vec<u8>>> {
    unimplemented!()
}
pub(crate) async fn file_meta(data: Vec<u8>, base: &Path) -> Option<Vec<Vec<u8>>> {
    unimplemented!()
}
pub(crate) async fn file_size(data: Vec<u8>, base: &Path) -> Option<Vec<Vec<u8>>> {
    unimplemented!()
}
pub(crate) async fn create_dir(data: Vec<u8>, base: &Path) -> Option<Vec<Vec<u8>>> {
    unimplemented!()
}
pub(crate) async fn create_file(data: Vec<u8>, base: &Path) -> Option<Vec<Vec<u8>>> {
    unimplemented!()
}
pub(crate) async fn move_file(data: Vec<u8>, base: &Path) -> Option<Vec<Vec<u8>>> {
    unimplemented!()
}
pub(crate) async fn copy_file(data: Vec<u8>, base: &Path) -> Option<Vec<Vec<u8>>> {
    unimplemented!()
}
pub(crate) async fn remove_file(data: Vec<u8>, base: &Path) -> Option<Vec<Vec<u8>>> {
    unimplemented!()
}
pub(crate) async fn download(data: Vec<u8>, base: &Path) -> Option<Vec<Vec<u8>>> {
    unimplemented!()
}
pub(crate) async fn chmod(data: Vec<u8>, base: &Path) -> Option<Vec<Vec<u8>>> {
    unimplemented!()
}
