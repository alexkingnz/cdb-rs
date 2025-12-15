extern crate tumu_cdb;
#[cfg(feature = "std")]
use std::fs;
use tumu_cdb as cdb;

macro_rules! noerr {
    ( $e:expr ) => {
        match $e {
            Ok(r) => r,
            Err(x) => panic!("{}", x),
        }
    };
}

const FILENAME: &str = "tests/make.cdb";

#[test]
fn test_make() {
    #[cfg(not(feature = "std"))]
    let mut cdb = cdb::CDBMake::new(cdb::vecbuf::VecBuf::new()).unwrap();
    #[cfg(feature = "std")]
    let mut cdb = cdb::CDBWriter::create(FILENAME).unwrap();
    noerr!(cdb.add(b"one", b"Hello"));
    noerr!(cdb.add(b"two", b"Goodbye"));
    noerr!(cdb.add(b"one", b", World!"));
    noerr!(cdb.add(b"this key will be split across two reads", b"Got it."));
    let v = noerr!(cdb.finish());

    #[cfg(not(feature = "std"))]
    let cdb = cdb::CDB::copy_from_slice(v.get_ref()).unwrap();
    #[cfg(feature = "std")]
    let cdb = cdb::CDB::open(FILENAME).unwrap();
    assert_eq!(cdb.find(b"two").next().unwrap(), b"Goodbye");
    assert_eq!(
        cdb.find(b"this key will be split across two reads")
            .next()
            .unwrap(),
        b"Got it."
    );
    let mut i = cdb.find(b"one");
    assert_eq!(i.next().unwrap(), b"Hello");
    assert_eq!(i.next().unwrap(), b", World!");

    let mut i = cdb.iter();
    let next = i.next().unwrap().unwrap();
    assert_eq!(next.0, b"one");
    assert_eq!(next.1, b"Hello");
    let next = i.next().unwrap().unwrap();
    assert_eq!(next.0, b"two");
    assert_eq!(next.1, b"Goodbye");
    let next = i.next().unwrap().unwrap();
    assert_eq!(next.0, b"one");
    assert_eq!(next.1, b", World!");
    let next = i.next().unwrap().unwrap();
    // Can't do this key easily due to missing trait for [u8; 39]
    //assert_eq!(next.0, b"this key will be split across two reads");
    assert_eq!(next.1, b"Got it.");

    #[cfg(feature = "std")]
    noerr!(fs::remove_file(FILENAME));
}
