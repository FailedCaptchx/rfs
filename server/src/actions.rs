use std::{io::Bytes, os::unix::ffi::OsStrExt};

use quinn::Connection;

pub(crate) async fn list_files(conn: &mut Connection, mut data: Bytes<&[u8]>, base: &str) {
    match std::fs::read_dir(base) {
        Ok(r) => {
            let list = r.filter(|x| x.is_ok());
            let id = data.next().unwrap().unwrap();
            for o in list {
                let file = o.unwrap().file_name();
                let head = [0, id.clone()];
                let res = [&head, file.as_bytes()].concat();
                println!("{:?}", res);
                conn.send_datagram(bytes::Bytes::from(Box::from(res)))
                    .unwrap()
            }
        }
        Err(e) => println!("{}", e),
    }
}
pub(crate) async fn file_exists(conn: &mut Connection, data: Bytes<&[u8]>, base: &str) {
    std::fs::read_dir(base);
    println!("Client asked if file exists")
}
