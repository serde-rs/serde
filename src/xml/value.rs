use std::collections::BTreeMap;
use de;

#[derive(Clone, PartialEq)]
pub struct Element {
    attributes: BTreeMap<String, String>,
    members: Content,
}

#[derive(Clone, PartialEq)]
pub enum Content {
    Members(BTreeMap<String, Element>),
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
                    map.insert(key, try!(visitor.visit_value()));
                    content = Content::Members(map); // move back
                },
                Content::Nothing => {
                    // try to push to attributes
                    match try!(visitor.visit_value()) {
                        Helper::Member(el) => {
                            let mut m = BTreeMap::new();
                            m.insert(key, el);
                            content = Content::Members(m);
                        },
                        Helper::Text(s) => {
                            attributes.insert(key, s);
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
