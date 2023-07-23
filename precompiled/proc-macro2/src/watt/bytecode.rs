pub enum Bytecode {}

impl Bytecode {
    pub const GROUP_PARENTHESIS: u8 = 0;
    pub const GROUP_BRACE: u8 = 1;
    pub const GROUP_BRACKET: u8 = 2;
    pub const GROUP_NONE: u8 = 3;
    pub const IDENT: u8 = 4;
    pub const PUNCT_ALONE: u8 = 5;
    pub const PUNCT_JOINT: u8 = 6;
    pub const LITERAL: u8 = 7;
    pub const LOAD_GROUP: u8 = 8;
    pub const LOAD_IDENT: u8 = 9;
    pub const LOAD_PUNCT: u8 = 10;
    pub const LOAD_LITERAL: u8 = 11;
    pub const SET_SPAN: u8 = 12;
}
