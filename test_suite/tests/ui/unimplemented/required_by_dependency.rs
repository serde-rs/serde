struct MyStruct;

fn main() {
    serde_test::assert_ser_tokens(&MyStruct, &[]);
    serde_test::assert_de_tokens(&MyStruct, &[]);
}
