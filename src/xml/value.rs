use std::collections::BTreeMap;
use de;
use de::Error;

#[derive(Clone, PartialEq, Debug)]
pub struct Element {
    pub attributes: BTreeMap<String, Vec<String>>,
    pub members: Content,
}

impl Element {
    pub fn new_text(s: String) -> Element {
        Element {
            attributes: BTreeMap::new(),
            members: Content::Text(s),
        }
    }
    pub fn new_empty() -> Element {
        Element {
            attributes: BTreeMap::new(),
            members: Content::Nothing,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Content {
    Members(BTreeMap<String, Vec<Element>>),
    Text(String),
    Nothing,
}

impl de::Deserialize for Element {
    fn deserialize<D>(deserializer: &mut D) -> Result<Element, D::Error>
        where D: de::Deserializer,
    {
        deserializer.visit_map(ElementVisitor)
    }
}

enum Helper {
    Member(Element),
    Text(String),
}

impl de::Deserialize for Helper {
    fn deserialize<D>(deserializer: &mut D) -> Result<Helper, D::Error>
        where D: de::Deserializer,
    {
        let el = try!(deserializer.visit_map(ElementVisitor));
        Ok(match (el.attributes.is_empty(), el.members) {
            (true, Content::Text(s)) => Helper::Text(s),
            (_, c) => Helper::Member(Element {
                attributes: el.attributes,
                members: c
            }),
        })
    }
}

struct ElementVisitor;

impl de::Visitor for ElementVisitor {
    type Value = Element;

    fn visit_str<E>(&mut self, s: &str) -> Result<Element, E>
        where E: Error,
    {
        Ok(Element::new_text(s.to_string()))
    }

    fn visit_string<E>(&mut self, s: String) -> Result<Element, E>
        where E: Error,
    {
        Ok(Element::new_text(s))
    }

    #[inline]
    fn visit_map<V>(&mut self, mut visitor: V) -> Result<Element, V::Error>
        where V: de::MapVisitor,
    {
        let mut attributes = BTreeMap::new();
        let mut content = Content::Nothing;
        while let Some(key) = try!(visitor.visit_key::<String>()) {
            match content {
                Content::Nothing if key == "$value" => {
                    let v = try!(visitor.visit_value());
                    content = Content::Text(v);
                },
                Content::Text(_)=> unreachable!(),
                Content::Members(mut map) => {
                    map.entry(key)
                       .or_insert(vec![])
                       .push(try!(visitor.visit_value()));
                    content = Content::Members(map); // move back
                },
                Content::Nothing => {
                    // try to push to attributes
                    match try!(visitor.visit_value()) {
                        Helper::Member(el) => {
                            let mut m = BTreeMap::new();
                            m.insert(key, vec![el]);
                            content = Content::Members(m);
                        },
                        Helper::Text(s) => {
                            attributes.entry(key).or_insert(vec![]).push(s);
                            content = Content::Nothing; // move back
                        }
                    }
                },
            }
        }
        try!(visitor.end());
        Ok(Element {
            attributes: attributes,
            members: content,
        })
    }
}

/// Shortcut function to decode a Xml `Element` into a `T`
pub fn from_value<T>(value: Element) -> Result<T, super::error::Error>
    where T: de::Deserialize
{
    let mut de = super::de::value::Deserializer::new(value);
    de::Deserialize::deserialize(&mut de)
}
