use crate::records::confusable::Confusable;

/// Port of `findConfusable` (Ast/src/Confusables.cpp). Returns a NUL-terminated
/// ASCII "skeleton" for a confusable Unicode codepoint, or null if none. The
/// table is sorted ascending by codepoint (binary search relies on that).
#[allow(non_snake_case)]
pub fn find_confusable(codepoint: u32) -> *const core::ffi::c_char {
    match K_CONFUSABLES.binary_search_by(|probe| probe.codepoint.cmp(&codepoint)) {
        Ok(index) => K_CONFUSABLES[index].text.as_ptr(),
        Err(_) => core::ptr::null(),
    }
}

/// Derived from http://www.unicode.org/Public/security/10.0.0/confusables.txt;
/// sorted by codepoint. Faithful transcription of `kConfusables`.
const K_CONFUSABLES: &[Confusable] = &[
    Confusable {
        codepoint: 34,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 48,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 49,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 73,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 96,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 109,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 124,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 160,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 180,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 184,
        text: [44, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 198,
        text: [65, 69, 0, 0, 0],
    },
    Confusable {
        codepoint: 215,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 230,
        text: [97, 101, 0, 0, 0],
    },
    Confusable {
        codepoint: 305,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 306,
        text: [108, 74, 0, 0, 0],
    },
    Confusable {
        codepoint: 307,
        text: [105, 106, 0, 0, 0],
    },
    Confusable {
        codepoint: 329,
        text: [39, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 338,
        text: [79, 69, 0, 0, 0],
    },
    Confusable {
        codepoint: 339,
        text: [111, 101, 0, 0, 0],
    },
    Confusable {
        codepoint: 383,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 385,
        text: [39, 66, 0, 0, 0],
    },
    Confusable {
        codepoint: 388,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 391,
        text: [67, 39, 0, 0, 0],
    },
    Confusable {
        codepoint: 394,
        text: [39, 68, 0, 0, 0],
    },
    Confusable {
        codepoint: 397,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 403,
        text: [71, 39, 0, 0, 0],
    },
    Confusable {
        codepoint: 406,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 408,
        text: [75, 39, 0, 0, 0],
    },
    Confusable {
        codepoint: 416,
        text: [79, 39, 0, 0, 0],
    },
    Confusable {
        codepoint: 417,
        text: [111, 39, 0, 0, 0],
    },
    Confusable {
        codepoint: 420,
        text: [39, 80, 0, 0, 0],
    },
    Confusable {
        codepoint: 422,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 423,
        text: [50, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 428,
        text: [39, 84, 0, 0, 0],
    },
    Confusable {
        codepoint: 435,
        text: [39, 89, 0, 0, 0],
    },
    Confusable {
        codepoint: 439,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 444,
        text: [53, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 445,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 448,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 449,
        text: [108, 108, 0, 0, 0],
    },
    Confusable {
        codepoint: 451,
        text: [33, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 455,
        text: [76, 74, 0, 0, 0],
    },
    Confusable {
        codepoint: 456,
        text: [76, 106, 0, 0, 0],
    },
    Confusable {
        codepoint: 457,
        text: [108, 106, 0, 0, 0],
    },
    Confusable {
        codepoint: 458,
        text: [78, 74, 0, 0, 0],
    },
    Confusable {
        codepoint: 459,
        text: [78, 106, 0, 0, 0],
    },
    Confusable {
        codepoint: 460,
        text: [110, 106, 0, 0, 0],
    },
    Confusable {
        codepoint: 497,
        text: [68, 90, 0, 0, 0],
    },
    Confusable {
        codepoint: 498,
        text: [68, 122, 0, 0, 0],
    },
    Confusable {
        codepoint: 499,
        text: [100, 122, 0, 0, 0],
    },
    Confusable {
        codepoint: 540,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 546,
        text: [56, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 547,
        text: [56, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 577,
        text: [63, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 593,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 609,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 611,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 617,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 618,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 623,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 651,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 655,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 660,
        text: [63, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 675,
        text: [100, 122, 0, 0, 0],
    },
    Confusable {
        codepoint: 678,
        text: [116, 115, 0, 0, 0],
    },
    Confusable {
        codepoint: 682,
        text: [108, 115, 0, 0, 0],
    },
    Confusable {
        codepoint: 683,
        text: [108, 122, 0, 0, 0],
    },
    Confusable {
        codepoint: 697,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 698,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 699,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 700,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 701,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 702,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 706,
        text: [60, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 707,
        text: [62, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 708,
        text: [94, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 710,
        text: [94, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 712,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 714,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 715,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 720,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 727,
        text: [45, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 731,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 732,
        text: [126, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 733,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 750,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 756,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 758,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 760,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 884,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 890,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 894,
        text: [59, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 895,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 900,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 913,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 914,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 917,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 918,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 919,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 921,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 922,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 924,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 925,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 927,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 929,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 932,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 933,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 935,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 945,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 947,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 953,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 957,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 959,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 961,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 963,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 965,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 978,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 988,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1000,
        text: [50, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1009,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1010,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1011,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1017,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1018,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1029,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1030,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1032,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1040,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1042,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1045,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1047,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1050,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1052,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1053,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1054,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1056,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1057,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1058,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1059,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1061,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1067,
        text: [98, 108, 0, 0, 0],
    },
    Confusable {
        codepoint: 1068,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1070,
        text: [108, 79, 0, 0, 0],
    },
    Confusable {
        codepoint: 1072,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1073,
        text: [54, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1075,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1077,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1086,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1088,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1089,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1091,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1093,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1109,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1110,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1112,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1121,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1140,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1141,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1169,
        text: [114, 39, 0, 0, 0],
    },
    Confusable {
        codepoint: 1198,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1199,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1211,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1213,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1216,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1231,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1236,
        text: [65, 69, 0, 0, 0],
    },
    Confusable {
        codepoint: 1237,
        text: [97, 101, 0, 0, 0],
    },
    Confusable {
        codepoint: 1248,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1281,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1292,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1307,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1308,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1309,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1357,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1359,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1365,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1370,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1373,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1377,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1379,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1382,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1392,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1400,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1404,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1405,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1409,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1412,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1413,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1417,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1472,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1475,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1493,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1496,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1497,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1503,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1505,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1520,
        text: [108, 108, 0, 0, 0],
    },
    Confusable {
        codepoint: 1521,
        text: [108, 39, 0, 0, 0],
    },
    Confusable {
        codepoint: 1522,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1523,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1524,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1549,
        text: [44, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1575,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1607,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1632,
        text: [46, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1633,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1637,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1639,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1643,
        text: [44, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1645,
        text: [42, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1726,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1729,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1748,
        text: [45, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1749,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1776,
        text: [46, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1777,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1781,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1783,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1793,
        text: [46, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1794,
        text: [46, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1795,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1796,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1984,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 1994,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2036,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2037,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2042,
        text: [95, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2307,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2406,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2429,
        text: [63, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2534,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2538,
        text: [56, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2541,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2662,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2663,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2666,
        text: [56, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2691,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2790,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2819,
        text: [56, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2848,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2918,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 2920,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 3046,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 3074,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 3174,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 3202,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 3302,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 3330,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 3360,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 3430,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 3437,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 3458,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 3664,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 3792,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 4125,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 4160,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 4327,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 4351,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 4608,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 4816,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5024,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5025,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5026,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5028,
        text: [79, 39, 0, 0, 0],
    },
    Confusable {
        codepoint: 5029,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5033,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5034,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5035,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5036,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5038,
        text: [63, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5043,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5047,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5051,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5053,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5056,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5058,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5059,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5070,
        text: [52, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5071,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5074,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5076,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5077,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5081,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5082,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5086,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5087,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5090,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5094,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5095,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5102,
        text: [54, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5107,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5108,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5120,
        text: [61, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5167,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5171,
        text: [62, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5176,
        text: [60, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5194,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5196,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5223,
        text: [85, 39, 0, 0, 0],
    },
    Confusable {
        codepoint: 5229,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5231,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5254,
        text: [80, 39, 0, 0, 0],
    },
    Confusable {
        codepoint: 5255,
        text: [100, 39, 0, 0, 0],
    },
    Confusable {
        codepoint: 5261,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5290,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5311,
        text: [50, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5441,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5500,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5501,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5511,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5551,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5556,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5573,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5598,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5610,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5616,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5623,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5741,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5742,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5760,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5810,
        text: [60, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5815,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5825,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5836,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5845,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5846,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5868,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5869,
        text: [43, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 5941,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 6147,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 6153,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7379,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7428,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7439,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7441,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7452,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7456,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7457,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7458,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7462,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7531,
        text: [117, 101, 0, 0, 0],
    },
    Confusable {
        codepoint: 7555,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7564,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7837,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 7935,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8125,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8126,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8127,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8128,
        text: [126, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8175,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8189,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8190,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8192,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8193,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8194,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8195,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8196,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8197,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8198,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8199,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8200,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8201,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8202,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8208,
        text: [45, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8209,
        text: [45, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8210,
        text: [45, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8211,
        text: [45, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8214,
        text: [108, 108, 0, 0, 0],
    },
    Confusable {
        codepoint: 8216,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8217,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8218,
        text: [44, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8219,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8220,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8221,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8223,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8228,
        text: [46, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8229,
        text: [46, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 8230,
        text: [46, 46, 46, 0, 0],
    },
    Confusable {
        codepoint: 8232,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8233,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8239,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8242,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8243,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8244,
        text: [39, 39, 39, 0, 0],
    },
    Confusable {
        codepoint: 8245,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8246,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8247,
        text: [39, 39, 39, 0, 0],
    },
    Confusable {
        codepoint: 8249,
        text: [60, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8250,
        text: [62, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8252,
        text: [33, 33, 0, 0, 0],
    },
    Confusable {
        codepoint: 8257,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8259,
        text: [45, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8260,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8263,
        text: [63, 63, 0, 0, 0],
    },
    Confusable {
        codepoint: 8264,
        text: [63, 33, 0, 0, 0],
    },
    Confusable {
        codepoint: 8265,
        text: [33, 63, 0, 0, 0],
    },
    Confusable {
        codepoint: 8270,
        text: [42, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8275,
        text: [126, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8279,
        text: [39, 39, 39, 39, 0],
    },
    Confusable {
        codepoint: 8282,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8287,
        text: [32, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8360,
        text: [82, 115, 0, 0, 0],
    },
    Confusable {
        codepoint: 8374,
        text: [108, 116, 0, 0, 0],
    },
    Confusable {
        codepoint: 8448,
        text: [97, 47, 99, 0, 0],
    },
    Confusable {
        codepoint: 8449,
        text: [97, 47, 115, 0, 0],
    },
    Confusable {
        codepoint: 8450,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8453,
        text: [99, 47, 111, 0, 0],
    },
    Confusable {
        codepoint: 8454,
        text: [99, 47, 117, 0, 0],
    },
    Confusable {
        codepoint: 8458,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8459,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8460,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8461,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8462,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8464,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8465,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8466,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8467,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8469,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8470,
        text: [78, 111, 0, 0, 0],
    },
    Confusable {
        codepoint: 8473,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8474,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8475,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8476,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8477,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8481,
        text: [84, 69, 76, 0, 0],
    },
    Confusable {
        codepoint: 8484,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8488,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8490,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8492,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8493,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8494,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8495,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8496,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8497,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8499,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8500,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8505,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8507,
        text: [70, 65, 88, 0, 0],
    },
    Confusable {
        codepoint: 8509,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8517,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8518,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8519,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8520,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8521,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8544,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8545,
        text: [108, 108, 0, 0, 0],
    },
    Confusable {
        codepoint: 8546,
        text: [108, 108, 108, 0, 0],
    },
    Confusable {
        codepoint: 8547,
        text: [108, 86, 0, 0, 0],
    },
    Confusable {
        codepoint: 8548,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8549,
        text: [86, 108, 0, 0, 0],
    },
    Confusable {
        codepoint: 8550,
        text: [86, 108, 108, 0, 0],
    },
    Confusable {
        codepoint: 8551,
        text: [86, 108, 108, 108, 0],
    },
    Confusable {
        codepoint: 8552,
        text: [108, 88, 0, 0, 0],
    },
    Confusable {
        codepoint: 8553,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8554,
        text: [88, 108, 0, 0, 0],
    },
    Confusable {
        codepoint: 8555,
        text: [88, 108, 108, 0, 0],
    },
    Confusable {
        codepoint: 8556,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8557,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8558,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8559,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8560,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8561,
        text: [105, 105, 0, 0, 0],
    },
    Confusable {
        codepoint: 8562,
        text: [105, 105, 105, 0, 0],
    },
    Confusable {
        codepoint: 8563,
        text: [105, 118, 0, 0, 0],
    },
    Confusable {
        codepoint: 8564,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8565,
        text: [118, 105, 0, 0, 0],
    },
    Confusable {
        codepoint: 8566,
        text: [118, 105, 105, 0, 0],
    },
    Confusable {
        codepoint: 8567,
        text: [118, 105, 105, 105, 0],
    },
    Confusable {
        codepoint: 8568,
        text: [105, 120, 0, 0, 0],
    },
    Confusable {
        codepoint: 8569,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8570,
        text: [120, 105, 0, 0, 0],
    },
    Confusable {
        codepoint: 8571,
        text: [120, 105, 105, 0, 0],
    },
    Confusable {
        codepoint: 8572,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8573,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8574,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8575,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 8722,
        text: [45, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8725,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8726,
        text: [92, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8727,
        text: [42, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8734,
        text: [111, 111, 0, 0, 0],
    },
    Confusable {
        codepoint: 8739,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8741,
        text: [108, 108, 0, 0, 0],
    },
    Confusable {
        codepoint: 8744,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8746,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8758,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8764,
        text: [126, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8810,
        text: [60, 60, 0, 0, 0],
    },
    Confusable {
        codepoint: 8811,
        text: [62, 62, 0, 0, 0],
    },
    Confusable {
        codepoint: 8868,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8897,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8899,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 8920,
        text: [60, 60, 60, 0, 0],
    },
    Confusable {
        codepoint: 8921,
        text: [62, 62, 62, 0, 0],
    },
    Confusable {
        codepoint: 8959,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 9075,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 9076,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 9082,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 9213,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 9290,
        text: [92, 92, 0, 0, 0],
    },
    Confusable {
        codepoint: 9332,
        text: [40, 108, 41, 0, 0],
    },
    Confusable {
        codepoint: 9333,
        text: [40, 50, 41, 0, 0],
    },
    Confusable {
        codepoint: 9334,
        text: [40, 51, 41, 0, 0],
    },
    Confusable {
        codepoint: 9335,
        text: [40, 52, 41, 0, 0],
    },
    Confusable {
        codepoint: 9336,
        text: [40, 53, 41, 0, 0],
    },
    Confusable {
        codepoint: 9337,
        text: [40, 54, 41, 0, 0],
    },
    Confusable {
        codepoint: 9338,
        text: [40, 55, 41, 0, 0],
    },
    Confusable {
        codepoint: 9339,
        text: [40, 56, 41, 0, 0],
    },
    Confusable {
        codepoint: 9340,
        text: [40, 57, 41, 0, 0],
    },
    Confusable {
        codepoint: 9341,
        text: [40, 108, 79, 41, 0],
    },
    Confusable {
        codepoint: 9342,
        text: [40, 108, 108, 41, 0],
    },
    Confusable {
        codepoint: 9343,
        text: [40, 108, 50, 41, 0],
    },
    Confusable {
        codepoint: 9344,
        text: [40, 108, 51, 41, 0],
    },
    Confusable {
        codepoint: 9345,
        text: [40, 108, 52, 41, 0],
    },
    Confusable {
        codepoint: 9346,
        text: [40, 108, 53, 41, 0],
    },
    Confusable {
        codepoint: 9347,
        text: [40, 108, 54, 41, 0],
    },
    Confusable {
        codepoint: 9348,
        text: [40, 108, 55, 41, 0],
    },
    Confusable {
        codepoint: 9349,
        text: [40, 108, 56, 41, 0],
    },
    Confusable {
        codepoint: 9350,
        text: [40, 108, 57, 41, 0],
    },
    Confusable {
        codepoint: 9351,
        text: [40, 50, 79, 41, 0],
    },
    Confusable {
        codepoint: 9352,
        text: [108, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 9353,
        text: [50, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 9354,
        text: [51, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 9355,
        text: [52, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 9356,
        text: [53, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 9357,
        text: [54, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 9358,
        text: [55, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 9359,
        text: [56, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 9360,
        text: [57, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 9361,
        text: [108, 79, 46, 0, 0],
    },
    Confusable {
        codepoint: 9362,
        text: [108, 108, 46, 0, 0],
    },
    Confusable {
        codepoint: 9363,
        text: [108, 50, 46, 0, 0],
    },
    Confusable {
        codepoint: 9364,
        text: [108, 51, 46, 0, 0],
    },
    Confusable {
        codepoint: 9365,
        text: [108, 52, 46, 0, 0],
    },
    Confusable {
        codepoint: 9366,
        text: [108, 53, 46, 0, 0],
    },
    Confusable {
        codepoint: 9367,
        text: [108, 54, 46, 0, 0],
    },
    Confusable {
        codepoint: 9368,
        text: [108, 55, 46, 0, 0],
    },
    Confusable {
        codepoint: 9369,
        text: [108, 56, 46, 0, 0],
    },
    Confusable {
        codepoint: 9370,
        text: [108, 57, 46, 0, 0],
    },
    Confusable {
        codepoint: 9371,
        text: [50, 79, 46, 0, 0],
    },
    Confusable {
        codepoint: 9372,
        text: [40, 97, 41, 0, 0],
    },
    Confusable {
        codepoint: 9373,
        text: [40, 98, 41, 0, 0],
    },
    Confusable {
        codepoint: 9374,
        text: [40, 99, 41, 0, 0],
    },
    Confusable {
        codepoint: 9375,
        text: [40, 100, 41, 0, 0],
    },
    Confusable {
        codepoint: 9376,
        text: [40, 101, 41, 0, 0],
    },
    Confusable {
        codepoint: 9377,
        text: [40, 102, 41, 0, 0],
    },
    Confusable {
        codepoint: 9378,
        text: [40, 103, 41, 0, 0],
    },
    Confusable {
        codepoint: 9379,
        text: [40, 104, 41, 0, 0],
    },
    Confusable {
        codepoint: 9380,
        text: [40, 105, 41, 0, 0],
    },
    Confusable {
        codepoint: 9381,
        text: [40, 106, 41, 0, 0],
    },
    Confusable {
        codepoint: 9382,
        text: [40, 107, 41, 0, 0],
    },
    Confusable {
        codepoint: 9383,
        text: [40, 108, 41, 0, 0],
    },
    Confusable {
        codepoint: 9384,
        text: [40, 114, 110, 41, 0],
    },
    Confusable {
        codepoint: 9385,
        text: [40, 110, 41, 0, 0],
    },
    Confusable {
        codepoint: 9386,
        text: [40, 111, 41, 0, 0],
    },
    Confusable {
        codepoint: 9387,
        text: [40, 112, 41, 0, 0],
    },
    Confusable {
        codepoint: 9388,
        text: [40, 113, 41, 0, 0],
    },
    Confusable {
        codepoint: 9389,
        text: [40, 114, 41, 0, 0],
    },
    Confusable {
        codepoint: 9390,
        text: [40, 115, 41, 0, 0],
    },
    Confusable {
        codepoint: 9391,
        text: [40, 116, 41, 0, 0],
    },
    Confusable {
        codepoint: 9392,
        text: [40, 117, 41, 0, 0],
    },
    Confusable {
        codepoint: 9393,
        text: [40, 118, 41, 0, 0],
    },
    Confusable {
        codepoint: 9394,
        text: [40, 119, 41, 0, 0],
    },
    Confusable {
        codepoint: 9395,
        text: [40, 120, 41, 0, 0],
    },
    Confusable {
        codepoint: 9396,
        text: [40, 121, 41, 0, 0],
    },
    Confusable {
        codepoint: 9397,
        text: [40, 122, 41, 0, 0],
    },
    Confusable {
        codepoint: 9585,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 9587,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10088,
        text: [40, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10089,
        text: [41, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10094,
        text: [60, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10095,
        text: [62, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10098,
        text: [40, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10099,
        text: [41, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10100,
        text: [123, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10101,
        text: [125, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10133,
        text: [43, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10134,
        text: [45, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10187,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10189,
        text: [92, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10201,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10539,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10540,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10741,
        text: [92, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10744,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10745,
        text: [92, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10784,
        text: [62, 62, 0, 0, 0],
    },
    Confusable {
        codepoint: 10799,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 10868,
        text: [58, 58, 61, 0, 0],
    },
    Confusable {
        codepoint: 10869,
        text: [61, 61, 0, 0, 0],
    },
    Confusable {
        codepoint: 10870,
        text: [61, 61, 61, 0, 0],
    },
    Confusable {
        codepoint: 10917,
        text: [62, 60, 0, 0, 0],
    },
    Confusable {
        codepoint: 11003,
        text: [47, 47, 47, 0, 0],
    },
    Confusable {
        codepoint: 11005,
        text: [47, 47, 0, 0, 0],
    },
    Confusable {
        codepoint: 11397,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11406,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11410,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11412,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11416,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11418,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11422,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11423,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11426,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11427,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11428,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11429,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11430,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11432,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11436,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11450,
        text: [45, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11462,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11466,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11468,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11472,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11474,
        text: [54, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11513,
        text: [92, 92, 0, 0, 0],
    },
    Confusable {
        codepoint: 11576,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11577,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11599,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11601,
        text: [33, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11604,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11605,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11613,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 11816,
        text: [40, 40, 0, 0, 0],
    },
    Confusable {
        codepoint: 11817,
        text: [41, 41, 0, 0, 0],
    },
    Confusable {
        codepoint: 11840,
        text: [61, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 12034,
        text: [92, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 12035,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 12291,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 12295,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 12308,
        text: [40, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 12309,
        text: [41, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 12339,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 12448,
        text: [61, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 12494,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 12755,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 12756,
        text: [92, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 20022,
        text: [92, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 20031,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42192,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42193,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42194,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42195,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42196,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42198,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42199,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42201,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42202,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42204,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42205,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42207,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42208,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42209,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42210,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42211,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42214,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42215,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42218,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42219,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42220,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42222,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42224,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42226,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42227,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42228,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42232,
        text: [46, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42233,
        text: [44, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42234,
        text: [46, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 42235,
        text: [46, 44, 0, 0, 0],
    },
    Confusable {
        codepoint: 42237,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42238,
        text: [45, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 42239,
        text: [61, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42510,
        text: [46, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42564,
        text: [50, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42567,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42648,
        text: [79, 79, 0, 0, 0],
    },
    Confusable {
        codepoint: 42649,
        text: [111, 111, 0, 0, 0],
    },
    Confusable {
        codepoint: 42719,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42731,
        text: [63, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42735,
        text: [50, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42792,
        text: [84, 51, 0, 0, 0],
    },
    Confusable {
        codepoint: 42801,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42802,
        text: [65, 65, 0, 0, 0],
    },
    Confusable {
        codepoint: 42803,
        text: [97, 97, 0, 0, 0],
    },
    Confusable {
        codepoint: 42804,
        text: [65, 79, 0, 0, 0],
    },
    Confusable {
        codepoint: 42805,
        text: [97, 111, 0, 0, 0],
    },
    Confusable {
        codepoint: 42806,
        text: [65, 85, 0, 0, 0],
    },
    Confusable {
        codepoint: 42807,
        text: [97, 117, 0, 0, 0],
    },
    Confusable {
        codepoint: 42808,
        text: [65, 86, 0, 0, 0],
    },
    Confusable {
        codepoint: 42809,
        text: [97, 118, 0, 0, 0],
    },
    Confusable {
        codepoint: 42810,
        text: [65, 86, 0, 0, 0],
    },
    Confusable {
        codepoint: 42811,
        text: [97, 118, 0, 0, 0],
    },
    Confusable {
        codepoint: 42812,
        text: [65, 89, 0, 0, 0],
    },
    Confusable {
        codepoint: 42813,
        text: [97, 121, 0, 0, 0],
    },
    Confusable {
        codepoint: 42830,
        text: [79, 79, 0, 0, 0],
    },
    Confusable {
        codepoint: 42831,
        text: [111, 111, 0, 0, 0],
    },
    Confusable {
        codepoint: 42842,
        text: [50, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42858,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42862,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42871,
        text: [116, 102, 0, 0, 0],
    },
    Confusable {
        codepoint: 42872,
        text: [38, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42889,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42892,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42904,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42905,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42911,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42923,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42930,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42931,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 42932,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43826,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43829,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43837,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43847,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43848,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43854,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43858,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43866,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43875,
        text: [117, 111, 0, 0, 0],
    },
    Confusable {
        codepoint: 43893,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43905,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43907,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43923,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43945,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43946,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 43951,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 64256,
        text: [102, 102, 0, 0, 0],
    },
    Confusable {
        codepoint: 64257,
        text: [102, 105, 0, 0, 0],
    },
    Confusable {
        codepoint: 64258,
        text: [102, 108, 0, 0, 0],
    },
    Confusable {
        codepoint: 64259,
        text: [102, 102, 105, 0, 0],
    },
    Confusable {
        codepoint: 64260,
        text: [102, 102, 108, 0, 0],
    },
    Confusable {
        codepoint: 64262,
        text: [115, 116, 0, 0, 0],
    },
    Confusable {
        codepoint: 64422,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 64423,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 64424,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 64425,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 64426,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 64427,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 64428,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 64429,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 64830,
        text: [40, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 64831,
        text: [41, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65072,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65101,
        text: [95, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65102,
        text: [95, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65103,
        text: [95, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65112,
        text: [45, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65128,
        text: [92, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65165,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65166,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65257,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65258,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65259,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65260,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65281,
        text: [33, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65282,
        text: [34, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65287,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65306,
        text: [58, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65313,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65314,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65315,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65317,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65320,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65321,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65322,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65323,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65325,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65326,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65327,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65328,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65331,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65332,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65336,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65337,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65338,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65339,
        text: [40, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65340,
        text: [92, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65341,
        text: [41, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65344,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65345,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65347,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65349,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65351,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65352,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65353,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65354,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65356,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65359,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65360,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65363,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65366,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65368,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65369,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 65512,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66178,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66182,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66183,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66186,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66192,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66194,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66197,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66198,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66199,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66203,
        text: [43, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66208,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66209,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66210,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66213,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66219,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66224,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66225,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66226,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66228,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66255,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66293,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66305,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66306,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66313,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66321,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66325,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66327,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66330,
        text: [56, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66335,
        text: [42, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66336,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66338,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66564,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66581,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66587,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66592,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66604,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66621,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66632,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66740,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66754,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66766,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66770,
        text: [55, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66794,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66806,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66835,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66838,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66840,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66844,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66845,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66853,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66854,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 66855,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 68176,
        text: [46, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 70864,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71424,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 71430,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71434,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71438,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71439,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71840,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71842,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71843,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71844,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71846,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71849,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71852,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71854,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71855,
        text: [52, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71858,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71861,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71864,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71867,
        text: [53, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71868,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71872,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71873,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71874,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71875,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71876,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71878,
        text: [55, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71880,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71882,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71884,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71893,
        text: [54, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71894,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71895,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71896,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71900,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71904,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71907,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 71909,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71910,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71913,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71916,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71919,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 71922,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 93960,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 93962,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 93974,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 93992,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 94005,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 94010,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 94011,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 94015,
        text: [62, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 94016,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 94018,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 94019,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 94033,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 94034,
        text: [39, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119060,
        text: [123, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119149,
        text: [46, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119302,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119309,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119311,
        text: [92, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119314,
        text: [55, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119315,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119318,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119338,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119350,
        text: [60, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119351,
        text: [62, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119354,
        text: [47, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119355,
        text: [92, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119808,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119809,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119810,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119811,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119812,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119813,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119814,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119815,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119816,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119817,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119818,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119819,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119820,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119821,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119822,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119823,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119824,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119825,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119826,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119827,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119828,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119829,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119830,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119831,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119832,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119833,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119834,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119835,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119836,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119837,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119838,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119839,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119840,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119841,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119842,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119843,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119844,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119845,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119846,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 119847,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119848,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119849,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119850,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119851,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119852,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119853,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119854,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119855,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119856,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119857,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119858,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119859,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119860,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119861,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119862,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119863,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119864,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119865,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119866,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119867,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119868,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119869,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119870,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119871,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119872,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119873,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119874,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119875,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119876,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119877,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119878,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119879,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119880,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119881,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119882,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119883,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119884,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119885,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119886,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119887,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119888,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119889,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119890,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119891,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119892,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119894,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119895,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119896,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119897,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119898,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 119899,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119900,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119901,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119902,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119903,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119904,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119905,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119906,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119907,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119908,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119909,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119910,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119911,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119912,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119913,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119914,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119915,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119916,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119917,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119918,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119919,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119920,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119921,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119922,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119923,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119924,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119925,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119926,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119927,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119928,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119929,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119930,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119931,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119932,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119933,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119934,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119935,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119936,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119937,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119938,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119939,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119940,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119941,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119942,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119943,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119944,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119945,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119946,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119947,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119948,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119949,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119950,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 119951,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119952,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119953,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119954,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119955,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119956,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119957,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119958,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119959,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119960,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119961,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119962,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119963,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119964,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119966,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119967,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119970,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119973,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119974,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119977,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119978,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119979,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119980,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119982,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119983,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119984,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119985,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119986,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119987,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119988,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119989,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119990,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119991,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119992,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119993,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119995,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119997,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119998,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 119999,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120000,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120001,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120002,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 120003,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120005,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120006,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120007,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120008,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120009,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120010,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120011,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120012,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120013,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120014,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120015,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120016,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120017,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120018,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120019,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120020,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120021,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120022,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120023,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120024,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120025,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120026,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120027,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120028,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120029,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120030,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120031,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120032,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120033,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120034,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120035,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120036,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120037,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120038,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120039,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120040,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120041,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120042,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120043,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120044,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120045,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120046,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120047,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120048,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120049,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120050,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120051,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120052,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120053,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120054,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 120055,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120056,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120057,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120058,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120059,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120060,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120061,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120062,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120063,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120064,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120065,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120066,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120067,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120068,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120069,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120071,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120072,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120073,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120074,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120077,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120078,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120079,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120080,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120081,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120082,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120083,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120084,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120086,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120087,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120088,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120089,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120090,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120091,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120092,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120094,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120095,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120096,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120097,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120098,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120099,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120100,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120101,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120102,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120103,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120104,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120105,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120106,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 120107,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120108,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120109,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120110,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120111,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120112,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120113,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120114,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120115,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120116,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120117,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120118,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120119,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120120,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120121,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120123,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120124,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120125,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120126,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120128,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120129,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120130,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120131,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120132,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120134,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120138,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120139,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120140,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120141,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120142,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120143,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120144,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120146,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120147,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120148,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120149,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120150,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120151,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120152,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120153,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120154,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120155,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120156,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120157,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120158,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 120159,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120160,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120161,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120162,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120163,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120164,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120165,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120166,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120167,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120168,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120169,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120170,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120171,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120172,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120173,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120174,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120175,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120176,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120177,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120178,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120179,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120180,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120181,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120182,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120183,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120184,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120185,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120186,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120187,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120188,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120189,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120190,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120191,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120192,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120193,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120194,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120195,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120196,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120197,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120198,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120199,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120200,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120201,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120202,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120203,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120204,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120205,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120206,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120207,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120208,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120209,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120210,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 120211,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120212,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120213,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120214,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120215,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120216,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120217,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120218,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120219,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120220,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120221,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120222,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120223,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120224,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120225,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120226,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120227,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120228,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120229,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120230,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120231,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120232,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120233,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120234,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120235,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120236,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120237,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120238,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120239,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120240,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120241,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120242,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120243,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120244,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120245,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120246,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120247,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120248,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120249,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120250,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120251,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120252,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120253,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120254,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120255,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120256,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120257,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120258,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120259,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120260,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120261,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120262,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 120263,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120264,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120265,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120266,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120267,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120268,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120269,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120270,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120271,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120272,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120273,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120274,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120275,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120276,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120277,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120278,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120279,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120280,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120281,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120282,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120283,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120284,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120285,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120286,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120287,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120288,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120289,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120290,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120291,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120292,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120293,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120294,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120295,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120296,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120297,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120298,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120299,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120300,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120301,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120302,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120303,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120304,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120305,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120306,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120307,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120308,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120309,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120310,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120311,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120312,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120313,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120314,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 120315,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120316,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120317,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120318,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120319,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120320,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120321,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120322,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120323,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120324,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120325,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120326,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120327,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120328,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120329,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120330,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120331,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120332,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120333,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120334,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120335,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120336,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120337,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120338,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120339,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120340,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120341,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120342,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120343,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120344,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120345,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120346,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120347,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120348,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120349,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120350,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120351,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120352,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120353,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120354,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120355,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120356,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120357,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120358,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120359,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120360,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120361,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120362,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120363,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120364,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120365,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120366,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 120367,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120368,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120369,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120370,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120371,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120372,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120373,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120374,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120375,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120376,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120377,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120378,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120379,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120380,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120381,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120382,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120383,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120384,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120385,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120386,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120387,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120388,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120389,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120390,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120391,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120392,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120393,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120394,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120395,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120396,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120397,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120398,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120399,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120400,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120401,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120402,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120403,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120404,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120405,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120406,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120407,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120408,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120409,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120410,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120411,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120412,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120413,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120414,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120415,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120416,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120417,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120418,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 120419,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120420,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120421,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120422,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120423,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120424,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120425,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120426,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120427,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120428,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120429,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120430,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120431,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120432,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120433,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120434,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120435,
        text: [68, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120436,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120437,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120438,
        text: [71, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120439,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120440,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120441,
        text: [74, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120442,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120443,
        text: [76, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120444,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120445,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120446,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120447,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120448,
        text: [81, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120449,
        text: [82, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120450,
        text: [83, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120451,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120452,
        text: [85, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120453,
        text: [86, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120454,
        text: [87, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120455,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120456,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120457,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120458,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120459,
        text: [98, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120460,
        text: [99, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120461,
        text: [100, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120462,
        text: [101, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120463,
        text: [102, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120464,
        text: [103, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120465,
        text: [104, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120466,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120467,
        text: [106, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120468,
        text: [107, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120469,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120470,
        text: [114, 110, 0, 0, 0],
    },
    Confusable {
        codepoint: 120471,
        text: [110, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120472,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120473,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120474,
        text: [113, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120475,
        text: [114, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120476,
        text: [115, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120477,
        text: [116, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120478,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120479,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120480,
        text: [119, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120481,
        text: [120, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120482,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120483,
        text: [122, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120484,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120488,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120489,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120492,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120493,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120494,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120496,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120497,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120499,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120500,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120502,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120504,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120507,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120508,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120510,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120514,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120516,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120522,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120526,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120528,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120530,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120532,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120534,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120544,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120546,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120547,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120550,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120551,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120552,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120554,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120555,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120557,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120558,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120560,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120562,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120565,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120566,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120568,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120572,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120574,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120580,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120584,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120586,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120588,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120590,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120592,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120602,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120604,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120605,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120608,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120609,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120610,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120612,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120613,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120615,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120616,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120618,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120620,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120623,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120624,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120626,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120630,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120632,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120638,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120642,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120644,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120646,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120648,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120650,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120660,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120662,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120663,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120666,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120667,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120668,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120670,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120671,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120673,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120674,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120676,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120678,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120681,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120682,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120684,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120688,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120690,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120696,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120700,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120702,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120704,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120706,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120708,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120718,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120720,
        text: [65, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120721,
        text: [66, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120724,
        text: [69, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120725,
        text: [90, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120726,
        text: [72, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120728,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120729,
        text: [75, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120731,
        text: [77, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120732,
        text: [78, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120734,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120736,
        text: [80, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120739,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120740,
        text: [89, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120742,
        text: [88, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120746,
        text: [97, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120748,
        text: [121, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120754,
        text: [105, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120758,
        text: [118, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120760,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120762,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120764,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120766,
        text: [117, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120776,
        text: [112, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120778,
        text: [70, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120782,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120783,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120784,
        text: [50, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120785,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120786,
        text: [52, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120787,
        text: [53, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120788,
        text: [54, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120789,
        text: [55, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120790,
        text: [56, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120791,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120792,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120793,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120794,
        text: [50, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120795,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120796,
        text: [52, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120797,
        text: [53, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120798,
        text: [54, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120799,
        text: [55, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120800,
        text: [56, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120801,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120802,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120803,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120804,
        text: [50, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120805,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120806,
        text: [52, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120807,
        text: [53, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120808,
        text: [54, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120809,
        text: [55, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120810,
        text: [56, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120811,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120812,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120813,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120814,
        text: [50, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120815,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120816,
        text: [52, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120817,
        text: [53, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120818,
        text: [54, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120819,
        text: [55, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120820,
        text: [56, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120821,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120822,
        text: [79, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120823,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120824,
        text: [50, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120825,
        text: [51, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120826,
        text: [52, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120827,
        text: [53, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120828,
        text: [54, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120829,
        text: [55, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120830,
        text: [56, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 120831,
        text: [57, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 125127,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 125131,
        text: [56, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 126464,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 126500,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 126564,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 126592,
        text: [108, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 126596,
        text: [111, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 127232,
        text: [79, 46, 0, 0, 0],
    },
    Confusable {
        codepoint: 127233,
        text: [79, 44, 0, 0, 0],
    },
    Confusable {
        codepoint: 127234,
        text: [108, 44, 0, 0, 0],
    },
    Confusable {
        codepoint: 127235,
        text: [50, 44, 0, 0, 0],
    },
    Confusable {
        codepoint: 127236,
        text: [51, 44, 0, 0, 0],
    },
    Confusable {
        codepoint: 127237,
        text: [52, 44, 0, 0, 0],
    },
    Confusable {
        codepoint: 127238,
        text: [53, 44, 0, 0, 0],
    },
    Confusable {
        codepoint: 127239,
        text: [54, 44, 0, 0, 0],
    },
    Confusable {
        codepoint: 127240,
        text: [55, 44, 0, 0, 0],
    },
    Confusable {
        codepoint: 127241,
        text: [56, 44, 0, 0, 0],
    },
    Confusable {
        codepoint: 127242,
        text: [57, 44, 0, 0, 0],
    },
    Confusable {
        codepoint: 127248,
        text: [40, 65, 41, 0, 0],
    },
    Confusable {
        codepoint: 127249,
        text: [40, 66, 41, 0, 0],
    },
    Confusable {
        codepoint: 127250,
        text: [40, 67, 41, 0, 0],
    },
    Confusable {
        codepoint: 127251,
        text: [40, 68, 41, 0, 0],
    },
    Confusable {
        codepoint: 127252,
        text: [40, 69, 41, 0, 0],
    },
    Confusable {
        codepoint: 127253,
        text: [40, 70, 41, 0, 0],
    },
    Confusable {
        codepoint: 127254,
        text: [40, 71, 41, 0, 0],
    },
    Confusable {
        codepoint: 127255,
        text: [40, 72, 41, 0, 0],
    },
    Confusable {
        codepoint: 127256,
        text: [40, 108, 41, 0, 0],
    },
    Confusable {
        codepoint: 127257,
        text: [40, 74, 41, 0, 0],
    },
    Confusable {
        codepoint: 127258,
        text: [40, 75, 41, 0, 0],
    },
    Confusable {
        codepoint: 127259,
        text: [40, 76, 41, 0, 0],
    },
    Confusable {
        codepoint: 127260,
        text: [40, 77, 41, 0, 0],
    },
    Confusable {
        codepoint: 127261,
        text: [40, 78, 41, 0, 0],
    },
    Confusable {
        codepoint: 127262,
        text: [40, 79, 41, 0, 0],
    },
    Confusable {
        codepoint: 127263,
        text: [40, 80, 41, 0, 0],
    },
    Confusable {
        codepoint: 127264,
        text: [40, 81, 41, 0, 0],
    },
    Confusable {
        codepoint: 127265,
        text: [40, 82, 41, 0, 0],
    },
    Confusable {
        codepoint: 127266,
        text: [40, 83, 41, 0, 0],
    },
    Confusable {
        codepoint: 127267,
        text: [40, 84, 41, 0, 0],
    },
    Confusable {
        codepoint: 127268,
        text: [40, 85, 41, 0, 0],
    },
    Confusable {
        codepoint: 127269,
        text: [40, 86, 41, 0, 0],
    },
    Confusable {
        codepoint: 127270,
        text: [40, 87, 41, 0, 0],
    },
    Confusable {
        codepoint: 127271,
        text: [40, 88, 41, 0, 0],
    },
    Confusable {
        codepoint: 127272,
        text: [40, 89, 41, 0, 0],
    },
    Confusable {
        codepoint: 127273,
        text: [40, 90, 41, 0, 0],
    },
    Confusable {
        codepoint: 127274,
        text: [40, 83, 41, 0, 0],
    },
    Confusable {
        codepoint: 128768,
        text: [81, 69, 0, 0, 0],
    },
    Confusable {
        codepoint: 128775,
        text: [65, 82, 0, 0, 0],
    },
    Confusable {
        codepoint: 128844,
        text: [67, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 128860,
        text: [115, 115, 115, 0, 0],
    },
    Confusable {
        codepoint: 128872,
        text: [84, 0, 0, 0, 0],
    },
    Confusable {
        codepoint: 128875,
        text: [77, 66, 0, 0, 0],
    },
    Confusable {
        codepoint: 128876,
        text: [86, 66, 0, 0, 0],
    },
];
