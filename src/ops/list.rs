use crate::TrashDir;

pub fn list() {
    let home_trash = TrashDir::get_home_trash();
    let mut files = home_trash
        .iter()
        .unwrap()
        .filter_map(|entry| match entry {
            Ok(info) => Some(info),
            Err(err) => {
                eprintln!("failed to get file info: {:?}", err);
                None
            }
        })
        .collect::<Vec<_>>();
    files.sort_unstable_by_key(|info| info.deletion_date);
    for info in files {
        println!("{}\t{}", info.deletion_date, info.path.to_str().unwrap());
    }
}
