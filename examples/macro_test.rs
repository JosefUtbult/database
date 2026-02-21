use database_macro::build_database;

#[derive(Debug, Clone, Copy)]
struct MyNonCountingStruct {
    bingo: u8,
    bango: bool,
    bongo: f32,
}

build_database!(
    #[allow(dead_code)]
    struct MyMacroInnerInnerData {
        param6: u8,
    },
    #[allow(dead_code)]
    struct MyMacroInnerData {
        param4: u8,
        param5: bool,
        inner3: MyMacroInnerInnerData,
        non_counting: MyNonCountingStruct,
    },
    #[allow(dead_code)]
    struct MyMacroDatabase {
        param1: u8,
        param2: bool,
        param3: u8,
        inner1: MyMacroInnerData,
        inner2: MyMacroInnerData,
    }
);

fn main() {
    let _key = MyMacroDatabaseAbsKey::Inner1(MyMacroInnerDataAbsKey::Inner3(
        MyMacroInnerInnerDataAbsKey::Param6,
    ));
}
