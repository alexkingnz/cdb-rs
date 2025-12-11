extern crate cdb;
use std::os::fd::IntoRawFd;

#[test]
fn test_one() {
    #[cfg(feature = "std")]
    let cdb = cdb::CDB::open("tests/test1.cdb").unwrap();
    #[cfg(not(feature = "std"))]
    let cdb = {
        use std::fs::File;
        let file = File::open("tests/test1.cdb").unwrap();
        cdb::CDB::from_filedes(file.into_raw_fd()).unwrap()
    };
    let mut i = cdb.find(b"one");
    assert_eq!(i.next().unwrap().unwrap(), b"Hello");
    assert_eq!(i.next().unwrap().unwrap(), b", World!");
}

#[test]
fn test_two() {
    #[cfg(feature = "std")]
    let cdb = cdb::CDB::open("tests/test1.cdb").unwrap();
    #[cfg(not(feature = "std"))]
    let cdb = {
        use std::fs::File;
        let file = File::open("tests/test1.cdb").unwrap();
        cdb::CDB::from_filedes(file.into_raw_fd()).unwrap()
    };
    assert_eq!(cdb.find(b"two").next().unwrap().unwrap(), b"Goodbye");
    assert_eq!(
        cdb.find(b"this key will be split across two reads")
            .next()
            .unwrap()
            .unwrap(),
        b"Got it."
    );
}
