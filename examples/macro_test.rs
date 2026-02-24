use database_macro::build_database;

use database::{
    AbsFieldConstraints, AbsKeyConstraints, AllVariants, DataFieldAccessor, FlatFieldConstraints,
    FlatKeyConstraints, FocusHandler, LayerDatabase, ToKey, VariantCount,
};

build_database!(
    #[allow(dead_code)]
    #[derive(Debug)]
    struct MyMacroInnerInnerData {
        param6: u8,
        param7: u8,
        param8: u8,
    },
    #[allow(dead_code)]
    #[derive(Debug)]
    struct MyMacroInnerData {
        param4: u8,
        param5: bool,
        inner3: MyMacroInnerInnerData,
        inner4: MyMacroInnerInnerData,
    },
    #[allow(dead_code)]
    #[derive(Debug)]
    struct MyMacroDatabase {
        param1: u8,
        param2: bool,
        param3: u8,
        inner1: MyMacroInnerData,
        inner2: MyMacroInnerData,
        inner5: MyMacroInnerInnerData,
    }
);

fn main() {
    let field = MyMacroDatabaseAbsField::Inner1(MyMacroInnerDataAbsField::Inner3(
        MyMacroInnerInnerDataAbsField::Param6(0),
    ));
    let key = field.to_key();
    println!("Key: {:?}, Field: {:?}", key, field);
}
