use std::path::PathBuf;

pub fn split_uuid_to_file_name(uuid: &str) -> (String, String, String) {
    let trimed = uuid.replace("-", "");
    let trimed = trimed.trim();

    (trimed[0..2].into(), trimed[2..4].into(), trimed[4..].into())
}

pub fn append_from_path(pathbuf: &mut PathBuf, uuid: &str) {
    let (p1, p2, p3) = split_uuid_to_file_name(uuid);

    pathbuf.push(p1.as_str());
    pathbuf.push(p2.as_str());
    pathbuf.push(p3.as_str());
}

pub fn join_from_path(pathbuf: &PathBuf, uuid: &str) -> PathBuf {
    let mut pathbuf = pathbuf.clone();

    append_from_path(&mut pathbuf, uuid);

    return pathbuf;
}

#[cfg(test)]
mod test {
    use crate::utils::pathutils::split_uuid_to_file_name;

    #[test]
    fn test() {
        println!("{:?}", split_uuid_to_file_name("skdfjlkasd-jfask-ldfj"));
    }
}
