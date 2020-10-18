
pub const DIRS : [&str; 8] = [
    "dir1",
    "dir2/dir21",
    "dir2/dir22",
    "dir3/dir31/dir311",
    "dir3/dir31/dir312",
    "dir3/dir32/dir321",
    "dir3/dir32/dir322",
    "dir3/dir32/dir323/dir3231",
];

pub const FILE1 : &str = "dir1/test1";
pub const FILE2 : &str = "dir2/test2";
pub const FILE3 : &str = "dir3/dir31/test3";
pub const FILE4 : &str = "dir3/dir31/test4";

pub const FILES : [(&str, u64); 4] = [
    (FILE1, 128),
    (FILE2, 128),
    (FILE3, 512),
    (FILE4, 512),
];

pub const COPIES : [&[&str]; 4] = [
    &[
        "dir1/test2",
        "dir3/dir32/dir323/dir3231/test1",
        "dir3/dir32/dir323/test1",
        "dir3/dir32/dir323/test2",
    ],
    &[
        "dir3/dir32/dir323/test3",
        "dir3/dir32/dir323/test4",
    ],
    &[
        "dir3/dir31/dir312/test1",
        "dir3/dir31/test2"
    ],
    &[
        "dir3/dir32/dir323/dir3231/test2",
        "dir3/dir31/dir312/test2",
        "dir1/test3",
    ],
];

pub const LINKS : [&[&str]; 4] = [
    &[
        "dir1/link2",
        "dir3/dir32/dir323/dir3231/link1",
        "dir3/dir32/dir323/link1",
        "dir3/dir32/dir323/link2",
    ],
    &[
        "dir3/dir32/dir323/link3",
        "dir3/dir32/dir323/link4",
    ],
    &[
        "dir3/dir31/dir312/link1",
        "dir3/dir31/link2"
    ],
    &[
        "dir3/dir32/dir323/dir3231/link2",
        "dir3/dir31/dir312/link2",
        "dir1/link3",
    ],
];

