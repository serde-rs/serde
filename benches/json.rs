#![feature(core, plugin, test)]
#![plugin(serde_macros)]

extern crate serde;
extern crate "rustc-serialize" as rustc_serialize;
extern crate test;

use std::collections::BTreeMap;
use std::string;
use rustc_serialize as serialize;
use test::Bencher;

use serde::de::Token;

use serde::json::{Parser, Value, from_str};

macro_rules! treemap {
    ($($k:expr => $v:expr),*) => ({
        let mut _m = ::std::collections::BTreeMap::new();
        $(_m.insert($k, $v);)*
        _m
    })
}

fn json_str(count: usize) -> string::String {
    let mut src = "[".to_string();
    for _ in range(0, count) {
        src.push_str(r#"{"a":true,"b":null,"c":3.1415,"d":"Hello world","e":[1,2,3]},"#);
    }
    src.push_str("{}]");
    src
}

fn pretty_json_str(count: usize) -> string::String {
    let mut src = "[\n".to_string();
    for _ in range(0, count) {
        src.push_str(
            concat!(
                "  {\n",
                "    \"a\": true,\n",
                "    \"b\": null,\n",
                "    \"c\": 3.1415,\n",
                "    \"d\": \"Hello world\",\n",
                "    \"e\": [\n",
                "      1,\n",
                "      2,\n",
                "      3\n",
                "    ]\n",
                "  },\n"
            )
        );
    }
    src.push_str("  {}\n]");
    src
}

fn encoder_json(count: usize) -> serialize::json::Json {
    use rustc_serialize::json::Json;

    let mut list = vec!();
    for _ in range(0, count) {
        list.push(Json::Object(treemap!(
            "a".to_string() => Json::Boolean(true),
            "b".to_string() => Json::Null,
            "c".to_string() => Json::F64(3.1415),
            "d".to_string() => Json::String("Hello world".to_string()),
            "e".to_string() => Json::Array(vec!(
                Json::U64(1),
                Json::U64(2),
                Json::U64(3)
            ))
        )));
    }
    list.push(Json::Object(BTreeMap::new()));
    Json::Array(list)
}

fn serializer_json(count: usize) -> Value {
    let mut list = vec!();
    for _ in range(0, count) {
        list.push(Value::Object(treemap!(
            "a".to_string() => Value::Boolean(true),
            "b".to_string() => Value::Null,
            "c".to_string() => Value::Floating(3.1415),
            "d".to_string() => Value::String("Hello world".to_string()),
            "e".to_string() => Value::Array(vec!(
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3)
            ))
        )));
    }
    list.push(Value::Object(BTreeMap::new()));
    Value::Array(list)
}

fn bench_encoder(b: &mut Bencher, count: usize) {
    let src = json_str(count);
    let json = encoder_json(count);

    b.iter(|| {
        assert_eq!(json.to_string(), src);
    });
}

fn bench_encoder_pretty(b: &mut Bencher, count: usize) {
    let src = pretty_json_str(count);
    let json = encoder_json(count);

    b.iter(|| {
        assert_eq!(json.pretty().to_string(), src);
    });
}

fn bench_serializer(b: &mut Bencher, count: usize) {
    let src = json_str(count);
    let json = serializer_json(count);

    b.iter(|| {
        assert_eq!(json.to_string(), src);
    });
}

fn bench_serializer_pretty(b: &mut Bencher, count: usize) {
    let src = pretty_json_str(count);
    let json = serializer_json(count);

    b.iter(|| {
        assert_eq!(json.to_pretty_string(), src);
    });
}

fn bench_decoder(b: &mut Bencher, count: usize) {
    let src = json_str(count);
    let json = encoder_json(count);
    b.iter(|| {
        assert_eq!(json, serialize::json::Json::from_str(&src).unwrap());
    });
}

fn bench_deserializer(b: &mut Bencher, count: usize) {
    let src = json_str(count);
    let json = encoder_json(count);
    b.iter(|| {
        assert_eq!(json, serialize::json::Json::from_str(&src).unwrap());
    });
}

fn bench_decoder_streaming(b: &mut Bencher, count: usize) {
    let src = json_str(count);

    b.iter( || {
        use rustc_serialize::json::{Parser, JsonEvent, StackElement};

        let mut parser = Parser::new(src.chars());
        assert_eq!(parser.next(), Some(JsonEvent::ArrayStart));
        for _ in range(0, count) {
            assert_eq!(parser.next(), Some(JsonEvent::ObjectStart));

            assert_eq!(parser.next(), Some(JsonEvent::BooleanValue(true)));
            assert_eq!(parser.stack().top(), Some(StackElement::Key("a")));

            assert_eq!(parser.next(), Some(JsonEvent::NullValue));
            assert_eq!(parser.stack().top(), Some(StackElement::Key("b")));

            assert_eq!(parser.next(), Some(JsonEvent::F64Value(3.1415)));
            assert_eq!(parser.stack().top(), Some(StackElement::Key("c")));

            assert_eq!(parser.next(), Some(JsonEvent::StringValue("Hello world".to_string())));
            assert_eq!(parser.stack().top(), Some(StackElement::Key("d")));

            assert_eq!(parser.next(), Some(JsonEvent::ArrayStart));
            assert_eq!(parser.stack().top(), Some(StackElement::Key("e")));
            assert_eq!(parser.next(), Some(JsonEvent::U64Value(1)));
            assert_eq!(parser.next(), Some(JsonEvent::U64Value(2)));
            assert_eq!(parser.next(), Some(JsonEvent::U64Value(3)));
            assert_eq!(parser.next(), Some(JsonEvent::ArrayEnd));

            assert_eq!(parser.next(), Some(JsonEvent::ObjectEnd));
        }
        assert_eq!(parser.next(), Some(JsonEvent::ObjectStart));
        assert_eq!(parser.next(), Some(JsonEvent::ObjectEnd));
        assert_eq!(parser.next(), Some(JsonEvent::ArrayEnd));
        assert_eq!(parser.next(), None);
    });
}

fn bench_deserializer_streaming(b: &mut Bencher, count: usize) {
    let src = json_str(count);

    b.iter( || {
        let mut parser = Parser::new(src.bytes());

        assert_eq!(parser.next(), Some(Ok(Token::SeqStart(0))));
        for _ in range(0, count) {
            assert_eq!(parser.next(), Some(Ok(Token::MapStart(0))));

            assert_eq!(parser.next(), Some(Ok(Token::String("a".to_string()))));
            assert_eq!(parser.next(), Some(Ok(Token::Bool(true))));

            assert_eq!(parser.next(), Some(Ok(Token::String("b".to_string()))));
            assert_eq!(parser.next(), Some(Ok(Token::Null)));

            assert_eq!(parser.next(), Some(Ok(Token::String("c".to_string()))));
            assert_eq!(parser.next(), Some(Ok(Token::F64(3.1415))));

            assert_eq!(parser.next(), Some(Ok(Token::String("d".to_string()))));
            assert_eq!(parser.next(), Some(Ok(Token::String("Hello world".to_string()))));

            assert_eq!(parser.next(), Some(Ok(Token::String("e".to_string()))));
            assert_eq!(parser.next(), Some(Ok(Token::SeqStart(0))));
            assert_eq!(parser.next(), Some(Ok(Token::I64(1))));
            assert_eq!(parser.next(), Some(Ok(Token::I64(2))));
            assert_eq!(parser.next(), Some(Ok(Token::I64(3))));
            assert_eq!(parser.next(), Some(Ok(Token::End)));

            assert_eq!(parser.next(), Some(Ok(Token::End)));
        }
        assert_eq!(parser.next(), Some(Ok(Token::MapStart(0))));
        assert_eq!(parser.next(), Some(Ok(Token::End)));
        assert_eq!(parser.next(), Some(Ok(Token::End)));
        assert_eq!(parser.next(), None);

        loop {
            match parser.next() {
                None => return,
                Some(Ok(_)) => { }
                Some(Err(err)) => { panic!("error: {:?}", err); }
            }
        }
    });
}

#[bench]
fn bench_encoder_001(b: &mut Bencher) {
    bench_encoder(b, 1)
}

#[bench]
fn bench_encoder_500(b: &mut Bencher) {
    bench_encoder(b, 500)
}

#[bench]
fn bench_encoder_001_pretty(b: &mut Bencher) {
    bench_encoder_pretty(b, 1)
}

#[bench]
fn bench_encoder_500_pretty(b: &mut Bencher) {
    bench_encoder_pretty(b, 500)
}

#[bench]
fn bench_serializer_001(b: &mut Bencher) {
    bench_serializer(b, 1)
}

#[bench]
fn bench_serializer_500(b: &mut Bencher) {
    bench_serializer(b, 500)
}
#[bench]
fn bench_serializer_001_pretty(b: &mut Bencher) {
    bench_serializer_pretty(b, 1)
}

#[bench]
fn bench_serializer_500_pretty(b: &mut Bencher) {
    bench_serializer_pretty(b, 500)
}

#[bench]
fn bench_decoder_001(b: &mut Bencher) {
    bench_decoder(b, 1)
}

#[bench]
fn bench_decoder_500(b: &mut Bencher) {
    bench_decoder(b, 500)
}

#[bench]
fn bench_deserializer_001(b: &mut Bencher) {
    bench_deserializer(b, 1)
}

#[bench]
fn bench_deserializer_500(b: &mut Bencher) {
    bench_deserializer(b, 500)
}

#[bench]
fn bench_decoder_001_streaming(b: &mut Bencher) {
    bench_decoder_streaming(b, 1)
}

#[bench]
fn bench_decoder_500_streaming(b: &mut Bencher) {
    bench_decoder_streaming(b, 500)
}

#[bench]
fn bench_deserializer_001_streaming(b: &mut Bencher) {
    bench_deserializer_streaming(b, 1)
}

#[bench]
fn bench_deserializer_500_streaming(b: &mut Bencher) {
    bench_deserializer_streaming(b, 500)
}
