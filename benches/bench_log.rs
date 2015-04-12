#![feature(custom_derive, collections, plugin, test)]
#![allow(non_camel_case_types)]
#![plugin(serde_macros)]

extern crate num;
extern crate rustc_serialize;
extern crate serde;
extern crate test;

use std::io::{self, Read, Write};
use num::FromPrimitive;
use test::Bencher;

use serde::de::{self, Deserialize, Deserializer};
use serde::json::ser::escape_str;
use serde::json;
use serde::ser::{self, Serialize, Serializer};

use rustc_serialize::Encodable;

#[derive(Debug, PartialEq, RustcEncodable, RustcDecodable, Serialize, Deserialize)]
struct Http {
    protocol: HttpProtocol,
    status: u32,
    host_status: u32,
    up_status: u32,
    method: HttpMethod,
    content_type: String,
    user_agent: String,
    referer: String,
    request_uri: String,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum HttpProtocol {
    HTTP_PROTOCOL_UNKNOWN,
    HTTP10,
    HTTP11,
}

impl rustc_serialize::Encodable for HttpProtocol {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (*self as usize).encode(s)
    }
}

impl rustc_serialize::Decodable for HttpProtocol {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<HttpProtocol, D::Error> {
        match FromPrimitive::from_usize(try!(d.read_usize())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from usize")),
        }
    }
}

impl FromPrimitive for HttpProtocol {
    fn from_i64(i: i64) -> Option<HttpProtocol> {
        FromPrimitive::from_u64(i as u64)
    }

    fn from_u64(n: u64) -> Option<HttpProtocol> {
        match n {
            0 => Some(HttpProtocol::HTTP_PROTOCOL_UNKNOWN),
            1 => Some(HttpProtocol::HTTP10),
            2 => Some(HttpProtocol::HTTP11),
            _ => None,
        }
    }
}

impl ser::Serialize for HttpProtocol {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        serializer.visit_u8(*self as u8)
    }
}

impl de::Deserialize for HttpProtocol {
    #[inline]
    fn deserialize<
        S: Deserializer,
    >(state: &mut S) -> Result<HttpProtocol, S::Error> {
        state.visit(de::impls::PrimitiveVisitor::new())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum HttpMethod {
    METHOD_UNKNOWN,
    GET,
    POST,
    DELETE,
    PUT,
    HEAD,
    PURGE,
    OPTIONS,
    PROPFIND,
    MKCOL,
    PATCH,
}

impl FromPrimitive for HttpMethod {
    fn from_i64(i: i64) -> Option<HttpMethod> {
        FromPrimitive::from_u64(i as u64)
    }

    fn from_u64(n: u64) -> Option<HttpMethod> {
        match n {
            0 => Some(HttpMethod::METHOD_UNKNOWN),
            1 => Some(HttpMethod::GET),
            2 => Some(HttpMethod::POST),
            3 => Some(HttpMethod::DELETE),
            4 => Some(HttpMethod::PUT),
            5 => Some(HttpMethod::HEAD),
            6 => Some(HttpMethod::PURGE),
            7 => Some(HttpMethod::OPTIONS),
            8 => Some(HttpMethod::PROPFIND),
            9 => Some(HttpMethod::MKCOL),
            10 => Some(HttpMethod::PATCH),
            _ => None,
        }
    }
}

impl rustc_serialize::Encodable for HttpMethod {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (*self as usize).encode(s)
    }
}

impl rustc_serialize::Decodable for HttpMethod {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<HttpMethod, D::Error> {
        match FromPrimitive::from_usize(try!(d.read_usize())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from usize")),
        }
    }
}

impl ser::Serialize for HttpMethod {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        serializer.visit_u8(*self as u8)
    }
}

impl de::Deserialize for HttpMethod {
    #[inline]
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<HttpMethod, S::Error> {
        state.visit(de::impls::PrimitiveVisitor::new())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum CacheStatus {
    CACHESTATUS_UNKNOWN,
    Miss,
    Expired,
    Hit,
}

impl FromPrimitive for CacheStatus {
    fn from_i64(i: i64) -> Option<CacheStatus> {
        FromPrimitive::from_u64(i as u64)
    }

    fn from_u64(n: u64) -> Option<CacheStatus> {
        match n {
            0 => Some(CacheStatus::CACHESTATUS_UNKNOWN),
            1 => Some(CacheStatus::Miss),
            2 => Some(CacheStatus::Expired),
            3 => Some(CacheStatus::Hit),
            _ => None,
        }
    }
}

impl rustc_serialize::Encodable for CacheStatus {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (*self as u8).encode(s)
    }
}

impl rustc_serialize::Decodable for CacheStatus {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<CacheStatus, D::Error> {
        match FromPrimitive::from_u8(try!(d.read_u8())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from u8")),
        }
    }
}

impl ser::Serialize for CacheStatus {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        serializer.visit_u8(*self as u8)
    }
}

impl de::Deserialize for CacheStatus {
    #[inline]
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<CacheStatus, S::Error> {
        state.visit(de::impls::PrimitiveVisitor::new())
    }
}

#[derive(Debug, PartialEq, RustcEncodable, RustcDecodable, Serialize, Deserialize)]
struct Origin {
    ip: String,
    port: u32,
    hostname: String,
    protocol: OriginProtocol,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum OriginProtocol {
    ORIGIN_PROTOCOL_UNKNOWN,
    HTTP,
    HTTPS,
}

impl FromPrimitive for OriginProtocol {
    fn from_i64(i: i64) -> Option<OriginProtocol> {
        FromPrimitive::from_u64(i as u64)
    }

    fn from_u64(n: u64) -> Option<OriginProtocol> {
        match n {
            0 => Some(OriginProtocol::ORIGIN_PROTOCOL_UNKNOWN),
            1 => Some(OriginProtocol::HTTP),
            2 => Some(OriginProtocol::HTTPS),
            _ => None,
        }
    }
}

impl rustc_serialize::Encodable for OriginProtocol {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (*self as u8).encode(s)
    }
}

impl rustc_serialize::Decodable for OriginProtocol {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<OriginProtocol, D::Error> {
        match FromPrimitive::from_u8(try!(d.read_u8())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from u8")),
        }
    }
}

impl ser::Serialize for OriginProtocol {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        serializer.visit_u8(*self as u8)
    }
}

impl de::Deserialize for OriginProtocol {
    #[inline]
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<OriginProtocol, S::Error> {
        state.visit(de::impls::PrimitiveVisitor::new())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ZonePlan {
    ZONEPLAN_UNKNOWN,
    FREE,
    PRO,
    BIZ,
    ENT,
}

impl FromPrimitive for ZonePlan {
    fn from_i64(i: i64) -> Option<ZonePlan> {
        FromPrimitive::from_u64(i as u64)
    }

    fn from_u64(n: u64) -> Option<ZonePlan> {
        match n {
            0 => Some(ZonePlan::ZONEPLAN_UNKNOWN),
            1 => Some(ZonePlan::FREE),
            2 => Some(ZonePlan::PRO),
            3 => Some(ZonePlan::BIZ),
            4 => Some(ZonePlan::ENT),
            _ => None,
        }
    }
}

impl rustc_serialize::Encodable for ZonePlan {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (*self as u8).encode(s)
    }
}

impl rustc_serialize::Decodable for ZonePlan {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<ZonePlan, D::Error> {
        match FromPrimitive::from_u8(try!(d.read_u8())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from u8")),
        }
    }
}

impl ser::Serialize for ZonePlan {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        serializer.visit_u8(*self as u8)
    }
}

impl de::Deserialize for ZonePlan {
    #[inline]
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<ZonePlan, S::Error> {
        state.visit(de::impls::PrimitiveVisitor::new())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Country {
	UNKNOWN,
	A1,
	A2,
	O1,
	AD,
	AE,
	AF,
	AG,
	AI,
	AL,
	AM,
	AO,
	AP,
	AQ,
	AR,
	AS,
	AT,
	AU,
	AW,
	AX,
	AZ,
	BA,
	BB,
	BD,
	BE,
	BF,
	BG,
	BH,
	BI,
	BJ,
	BL,
	BM,
	BN,
	BO,
	BQ,
	BR,
	BS,
	BT,
	BV,
	BW,
	BY,
	BZ,
	CA,
	CC,
	CD,
	CF,
	CG,
	CH,
	CI,
	CK,
	CL,
	CM,
	CN,
	CO,
	CR,
	CU,
	CV,
	CW,
	CX,
	CY,
	CZ,
	DE,
	DJ,
	DK,
	DM,
	DO,
	DZ,
	EC,
	EE,
	EG,
	EH,
	ER,
	ES,
	ET,
	EU,
	FI,
	FJ,
	FK,
	FM,
	FO,
	FR,
	GA,
	GB,
	GD,
	GE,
	GF,
	GG,
	GH,
	GI,
	GL,
	GM,
	GN,
	GP,
	GQ,
	GR,
	GS,
	GT,
	GU,
	GW,
	GY,
	HK,
	HM,
	HN,
	HR,
	HT,
	HU,
	ID,
	IE,
	IL,
	IM,
	IN,
	IO,
	IQ,
	IR,
	IS,
	IT,
	JE,
	JM,
	JO,
	JP,
	KE,
	KG,
	KH,
	KI,
	KM,
	KN,
	KP,
	KR,
	KW,
	KY,
	KZ,
	LA,
	LB,
	LC,
	LI,
	LK,
	LR,
	LS,
	LT,
	LU,
	LV,
	LY,
	MA,
	MC,
	MD,
	ME,
	MF,
	MG,
	MH,
	MK,
	ML,
	MM,
	MN,
	MO,
	MP,
	MQ,
	MR,
	MS,
	MT,
	MU,
	MV,
	MW,
	MX,
	MY,
	MZ,
	NA,
	NC,
	NE,
	NF,
	NG,
	NI,
	NL,
	NO,
	NP,
	NR,
	NU,
	NZ,
	OM,
	PA,
	PE,
	PF,
	PG,
	PH,
	PK,
	PL,
	PM,
	PN,
	PR,
	PS,
	PT,
	PW,
	PY,
	QA,
	RE,
	RO,
	RS,
	RU,
	RW,
	SA,
	SB,
	SC,
	SD,
	SE,
	SG,
	SH,
	SI,
	SJ,
	SK,
	SL,
	SM,
	SN,
	SO,
	SR,
	SS,
	ST,
	SV,
	SX,
	SY,
	SZ,
	TC,
	TD,
	TF,
	TG,
	TH,
	TJ,
	TK,
	TL,
	TM,
	TN,
	TO,
	TR,
	TT,
	TV,
	TW,
	TZ,
	UA,
	UG,
	UM,
	US,
	UY,
	UZ,
	VA,
	VC,
	VE,
	VG,
	VI,
	VN,
	VU,
	WF,
	WS,
	XX,
	YE,
	YT,
	ZA,
	ZM,
	ZW,
}

impl FromPrimitive for Country {
    fn from_i64(i: i64) -> Option<Country> {
        FromPrimitive::from_u64(i as u64)
    }

    fn from_u64(n: u64) -> Option<Country> {
        match n {
            0 => Some(Country::UNKNOWN),
            1 => Some(Country::A1),
            2 => Some(Country::A2),
            3 => Some(Country::O1),
            4 => Some(Country::AD),
            5 => Some(Country::AE),
            6 => Some(Country::AF),
            7 => Some(Country::AG),
            8 => Some(Country::AI),
            9 => Some(Country::AL),
            10 => Some(Country::AM),
            11 => Some(Country::AO),
            12 => Some(Country::AP),
            13 => Some(Country::AQ),
            14 => Some(Country::AR),
            15 => Some(Country::AS),
            16 => Some(Country::AT),
            17 => Some(Country::AU),
            18 => Some(Country::AW),
            19 => Some(Country::AX),
            20 => Some(Country::AZ),
            21 => Some(Country::BA),
            22 => Some(Country::BB),
            23 => Some(Country::BD),
            24 => Some(Country::BE),
            25 => Some(Country::BF),
            26 => Some(Country::BG),
            27 => Some(Country::BH),
            28 => Some(Country::BI),
            29 => Some(Country::BJ),
            30 => Some(Country::BL),
            31 => Some(Country::BM),
            32 => Some(Country::BN),
            33 => Some(Country::BO),
            34 => Some(Country::BQ),
            35 => Some(Country::BR),
            36 => Some(Country::BS),
            37 => Some(Country::BT),
            38 => Some(Country::BV),
            39 => Some(Country::BW),
            40 => Some(Country::BY),
            41 => Some(Country::BZ),
            42 => Some(Country::CA),
            43 => Some(Country::CC),
            44 => Some(Country::CD),
            45 => Some(Country::CF),
            46 => Some(Country::CG),
            47 => Some(Country::CH),
            48 => Some(Country::CI),
            49 => Some(Country::CK),
            50 => Some(Country::CL),
            51 => Some(Country::CM),
            52 => Some(Country::CN),
            53 => Some(Country::CO),
            54 => Some(Country::CR),
            55 => Some(Country::CU),
            56 => Some(Country::CV),
            57 => Some(Country::CW),
            58 => Some(Country::CX),
            59 => Some(Country::CY),
            60 => Some(Country::CZ),
            61 => Some(Country::DE),
            62 => Some(Country::DJ),
            63 => Some(Country::DK),
            64 => Some(Country::DM),
            65 => Some(Country::DO),
            66 => Some(Country::DZ),
            67 => Some(Country::EC),
            68 => Some(Country::EE),
            69 => Some(Country::EG),
            70 => Some(Country::EH),
            71 => Some(Country::ER),
            72 => Some(Country::ES),
            73 => Some(Country::ET),
            74 => Some(Country::EU),
            75 => Some(Country::FI),
            76 => Some(Country::FJ),
            77 => Some(Country::FK),
            78 => Some(Country::FM),
            79 => Some(Country::FO),
            80 => Some(Country::FR),
            81 => Some(Country::GA),
            82 => Some(Country::GB),
            83 => Some(Country::GD),
            84 => Some(Country::GE),
            85 => Some(Country::GF),
            86 => Some(Country::GG),
            87 => Some(Country::GH),
            88 => Some(Country::GI),
            89 => Some(Country::GL),
            90 => Some(Country::GM),
            91 => Some(Country::GN),
            92 => Some(Country::GP),
            93 => Some(Country::GQ),
            94 => Some(Country::GR),
            95 => Some(Country::GS),
            96 => Some(Country::GT),
            97 => Some(Country::GU),
            98 => Some(Country::GW),
            99 => Some(Country::GY),
            100 => Some(Country::HK),
            101 => Some(Country::HM),
            102 => Some(Country::HN),
            103 => Some(Country::HR),
            104 => Some(Country::HT),
            105 => Some(Country::HU),
            106 => Some(Country::ID),
            107 => Some(Country::IE),
            108 => Some(Country::IL),
            109 => Some(Country::IM),
            110 => Some(Country::IN),
            111 => Some(Country::IO),
            112 => Some(Country::IQ),
            113 => Some(Country::IR),
            114 => Some(Country::IS),
            115 => Some(Country::IT),
            116 => Some(Country::JE),
            117 => Some(Country::JM),
            118 => Some(Country::JO),
            119 => Some(Country::JP),
            120 => Some(Country::KE),
            121 => Some(Country::KG),
            122 => Some(Country::KH),
            123 => Some(Country::KI),
            124 => Some(Country::KM),
            125 => Some(Country::KN),
            126 => Some(Country::KP),
            127 => Some(Country::KR),
            128 => Some(Country::KW),
            129 => Some(Country::KY),
            130 => Some(Country::KZ),
            131 => Some(Country::LA),
            132 => Some(Country::LB),
            133 => Some(Country::LC),
            134 => Some(Country::LI),
            135 => Some(Country::LK),
            136 => Some(Country::LR),
            137 => Some(Country::LS),
            138 => Some(Country::LT),
            139 => Some(Country::LU),
            140 => Some(Country::LV),
            141 => Some(Country::LY),
            142 => Some(Country::MA),
            143 => Some(Country::MC),
            144 => Some(Country::MD),
            145 => Some(Country::ME),
            146 => Some(Country::MF),
            147 => Some(Country::MG),
            148 => Some(Country::MH),
            149 => Some(Country::MK),
            150 => Some(Country::ML),
            151 => Some(Country::MM),
            152 => Some(Country::MN),
            153 => Some(Country::MO),
            154 => Some(Country::MP),
            155 => Some(Country::MQ),
            156 => Some(Country::MR),
            157 => Some(Country::MS),
            158 => Some(Country::MT),
            159 => Some(Country::MU),
            160 => Some(Country::MV),
            161 => Some(Country::MW),
            162 => Some(Country::MX),
            163 => Some(Country::MY),
            164 => Some(Country::MZ),
            165 => Some(Country::NA),
            166 => Some(Country::NC),
            167 => Some(Country::NE),
            168 => Some(Country::NF),
            169 => Some(Country::NG),
            170 => Some(Country::NI),
            171 => Some(Country::NL),
            172 => Some(Country::NO),
            173 => Some(Country::NP),
            174 => Some(Country::NR),
            175 => Some(Country::NU),
            176 => Some(Country::NZ),
            177 => Some(Country::OM),
            178 => Some(Country::PA),
            179 => Some(Country::PE),
            180 => Some(Country::PF),
            181 => Some(Country::PG),
            182 => Some(Country::PH),
            183 => Some(Country::PK),
            184 => Some(Country::PL),
            185 => Some(Country::PM),
            186 => Some(Country::PN),
            187 => Some(Country::PR),
            188 => Some(Country::PS),
            189 => Some(Country::PT),
            190 => Some(Country::PW),
            191 => Some(Country::PY),
            192 => Some(Country::QA),
            193 => Some(Country::RE),
            194 => Some(Country::RO),
            195 => Some(Country::RS),
            196 => Some(Country::RU),
            197 => Some(Country::RW),
            198 => Some(Country::SA),
            199 => Some(Country::SB),
            200 => Some(Country::SC),
            201 => Some(Country::SD),
            202 => Some(Country::SE),
            203 => Some(Country::SG),
            204 => Some(Country::SH),
            205 => Some(Country::SI),
            206 => Some(Country::SJ),
            207 => Some(Country::SK),
            208 => Some(Country::SL),
            209 => Some(Country::SM),
            210 => Some(Country::SN),
            211 => Some(Country::SO),
            212 => Some(Country::SR),
            213 => Some(Country::SS),
            214 => Some(Country::ST),
            215 => Some(Country::SV),
            216 => Some(Country::SX),
            217 => Some(Country::SY),
            218 => Some(Country::SZ),
            219 => Some(Country::TC),
            220 => Some(Country::TD),
            221 => Some(Country::TF),
            222 => Some(Country::TG),
            223 => Some(Country::TH),
            224 => Some(Country::TJ),
            225 => Some(Country::TK),
            226 => Some(Country::TL),
            227 => Some(Country::TM),
            228 => Some(Country::TN),
            229 => Some(Country::TO),
            230 => Some(Country::TR),
            231 => Some(Country::TT),
            232 => Some(Country::TV),
            233 => Some(Country::TW),
            234 => Some(Country::TZ),
            235 => Some(Country::UA),
            236 => Some(Country::UG),
            237 => Some(Country::UM),
            238 => Some(Country::US),
            239 => Some(Country::UY),
            240 => Some(Country::UZ),
            241 => Some(Country::VA),
            242 => Some(Country::VC),
            243 => Some(Country::VE),
            244 => Some(Country::VG),
            245 => Some(Country::VI),
            246 => Some(Country::VN),
            247 => Some(Country::VU),
            248 => Some(Country::WF),
            249 => Some(Country::WS),
            250 => Some(Country::XX),
            251 => Some(Country::YE),
            252 => Some(Country::YT),
            253 => Some(Country::ZA),
            254 => Some(Country::ZM),
            255 => Some(Country::ZW),
            _ => None,
        }
    }
}

impl rustc_serialize::Encodable for Country {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (*self as u8).encode(s)
    }
}

impl rustc_serialize::Decodable for Country {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Country, D::Error> {
        match FromPrimitive::from_u8(try!(d.read_u8())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from u8")),
        }
    }
}

impl ser::Serialize for Country {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer,
    {
        serializer.visit_u8(*self as u8)
    }
}

impl de::Deserialize for Country {
    #[inline]
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<Country, S::Error> {
        state.visit(de::impls::PrimitiveVisitor::new())
    }
}

#[derive(Debug, PartialEq, RustcEncodable, RustcDecodable, Serialize, Deserialize)]
struct Log {
    timestamp: i64,
    zone_id: u32,
    zone_plan: ZonePlan,
    http: Http,
    origin: Origin,
    country: Country,
    cache_status: CacheStatus,
    server_ip: String,
    server_name: String,
    remote_ip: String,
    bytes_dlv: u64,
    ray_id: String,
}

impl Log {
    fn new() -> Log {
        Log {
            timestamp: 2837513946597,
            zone_id: 123456,
            zone_plan: ZonePlan::FREE,
            http: Http {
                protocol: HttpProtocol::HTTP11,
                status: 200,
                host_status: 503,
                up_status: 520,
                method: HttpMethod::GET,
                content_type: "text/html".to_string(),
                user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36".to_string(),
                referer: "https://www.cloudflare.com/".to_string(),
                request_uri: "/cdn-cgi/trace".to_string(),
            },
            origin: Origin {
                ip: "1.2.3.4".to_string(),
                port: 8000,
                hostname: "www.example.com".to_string(),
                protocol: OriginProtocol::HTTPS,
            },
            country: Country::US,
            cache_status: CacheStatus::Hit,
            server_ip: "192.168.1.1".to_string(),
            server_name: "metal.cloudflare.com".to_string(),
            remote_ip: "10.1.2.3".to_string(),
            bytes_dlv: 123456,
            ray_id: "10c73629cce30078-LAX".to_string(),
        }
    }
}

macro_rules! likely(
    ($val:expr) => {
        {
            extern {
                #[link_name = "llvm.expect.i8"]
                fn expect(val: u8, expected_val: u8) -> u8;
            }
            let x: bool = $val;
            unsafe { expect(x as u8, 1) != 0 }
        }
    }
);

macro_rules! unlikely(
    ($val:expr) => {
        {
            extern {
                #[link_name = "llvm.expect.i8"]
                fn expect(val: u8, expected_val: u8) -> u8;
            }
            let x: bool = $val;
            unsafe { expect(x as u8, 0) != 0 }
        }
    }
);

struct MyMemWriter0 {
    buf: Vec<u8>,
}

impl MyMemWriter0 {
    pub fn with_capacity(cap: usize) -> MyMemWriter0 {
        MyMemWriter0 {
            buf: Vec::with_capacity(cap)
        }
    }
}


impl Write for MyMemWriter0 {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.push_all(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct MyMemWriter1 {
    buf: Vec<u8>,
}

impl MyMemWriter1 {
    pub fn with_capacity(cap: usize) -> MyMemWriter1 {
        MyMemWriter1 {
            buf: Vec::with_capacity(cap)
        }
    }
}

// LLVM isn't yet able to lower `Vec::push_all` into a memcpy, so this helps
// MemWriter eke out that last bit of performance.
#[inline]
fn push_all_bytes(dst: &mut Vec<u8>, src: &[u8]) {
    let dst_len = dst.len();
    let src_len = src.len();

    dst.reserve(src_len);

    unsafe {
        // we would have failed if `reserve` overflowed.
        dst.set_len(dst_len + src_len);

        ::std::ptr::copy_nonoverlapping(
            src.as_ptr(),
            dst.as_mut_ptr().offset(dst_len as isize),
            src_len);
    }
}

impl Write for MyMemWriter1 {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        push_all_bytes(&mut self.buf, buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

const JSON_STR: &'static str = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":1,"http":{"protocol":2,"status":200,"host_status":503,"up_status":520,"method":1,"content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":2},"country":238,"cache_status":3,"server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

#[test]
fn test_encoder() {
    use rustc_serialize::Encodable;

    let log = Log::new();

    let mut wr = String::with_capacity(1024);

    {
        let mut encoder = rustc_serialize::json::Encoder::new(&mut wr);
        log.encode(&mut encoder).unwrap();
    }

    assert_eq!(&wr, &JSON_STR);
}

#[bench]
fn bench_encoder(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = String::with_capacity(1024);

    {
        let mut encoder = rustc_serialize::json::Encoder::new(&mut wr);
        log.encode(&mut encoder).unwrap();
    }

    b.bytes = wr.len() as u64;

    b.iter(|| {
        wr.clear();

        let mut encoder = rustc_serialize::json::Encoder::new(&mut wr);
        log.encode(&mut encoder).unwrap();
    });
}

#[test]
fn test_serializer() {
    let log = Log::new();
    let json = json::to_vec(&log);
    assert_eq!(json, JSON_STR.as_bytes());
}

#[bench]
fn bench_serializer(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log);
    b.bytes = json.len() as u64;

    b.iter(|| {
        let _ = json::to_vec(&log);
    });
}

#[test]
fn test_serializer_vec() {
    let log = Log::new();
    let wr = Vec::with_capacity(1024);
    let mut serializer = json::Serializer::new(wr);
    log.serialize(&mut serializer).unwrap();

    let json = serializer.into_inner();
    assert_eq!(&json, &JSON_STR.as_bytes());
}

#[bench]
fn bench_serializer_vec(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log);
    b.bytes = json.len() as u64;

    let mut wr = Vec::with_capacity(1024);

    b.iter(|| {
        wr.clear();

        let mut serializer = json::Serializer::new(wr.by_ref());
        log.serialize(&mut serializer).unwrap();
        let _json = serializer.into_inner();
    });
}

#[bench]
fn bench_serializer_slice(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log);
    b.bytes = json.len() as u64;

    let mut buf = [0; 1024];

    b.iter(|| {
        for item in buf.iter_mut(){ *item = 0; }
        let mut wr = &mut buf[..];

        let mut serializer = json::Serializer::new(wr.by_ref());
        log.serialize(&mut serializer).unwrap();
        let _json = serializer.into_inner();
    });
}

#[test]
fn test_serializer_my_mem_writer0() {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);

    {
        let mut serializer = json::Serializer::new(wr.by_ref());
        log.serialize(&mut serializer).unwrap();
        let _json = serializer.into_inner();
    }

    assert_eq!(&wr.buf, &JSON_STR.as_bytes());
}

#[bench]
fn bench_serializer_my_mem_writer0(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log);
    b.bytes = json.len() as u64;

    let mut wr = MyMemWriter0::with_capacity(1024);

    b.iter(|| {
        wr.buf.clear();

        let mut serializer = json::Serializer::new(wr.by_ref());
        log.serialize(&mut serializer).unwrap();
        let _json = serializer.into_inner();
    });
}

#[test]
fn test_serializer_my_mem_writer1() {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);

    {
        let mut serializer = json::Serializer::new(wr.by_ref());
        log.serialize(&mut serializer).unwrap();
        let _json = serializer.into_inner();
    }

    assert_eq!(&wr.buf, &JSON_STR.as_bytes());
}

#[bench]
fn bench_serializer_my_mem_writer1(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log);
    b.bytes = json.len() as u64;

    let mut wr = MyMemWriter1::with_capacity(1024);

    b.iter(|| {
        wr.buf.clear();

        let mut serializer = json::Serializer::new(wr.by_ref());
        log.serialize(&mut serializer).unwrap();
        let _json = serializer.into_inner();
    });
}

#[bench]
fn bench_copy(b: &mut Bencher) {
    let json = JSON_STR.as_bytes().to_vec();
    b.bytes = json.len() as u64;

    b.iter(|| {
        let _json = JSON_STR.as_bytes().to_vec();
    });
}

fn manual_serialize_no_escape<W: Write>(wr: &mut W, log: &Log) {
    wr.write(b"{\"timestamp\":").unwrap();
    (write!(wr, "{}", log.timestamp)).unwrap();
    wr.write(b",\"zone_id\":").unwrap();
    (write!(wr, "{}", log.zone_id)).unwrap();
    wr.write(b",\"zone_plan\":").unwrap();
    (write!(wr, "{}", log.zone_plan as usize)).unwrap();

    wr.write(b",\"http\":{\"protocol\":").unwrap();
    (write!(wr, "{}", log.http.protocol as usize)).unwrap();
    wr.write(b",\"status\":").unwrap();
    (write!(wr, "{}", log.http.status)).unwrap();
    wr.write(b",\"host_status\":").unwrap();
    (write!(wr, "{}", log.http.host_status)).unwrap();
    wr.write(b",\"up_status\":").unwrap();
    (write!(wr, "{}", log.http.up_status)).unwrap();
    wr.write(b",\"method\":").unwrap();
    (write!(wr, "{}", log.http.method as usize)).unwrap();
    wr.write(b",\"content_type\":").unwrap();
    (write!(wr, "\"{}\"", log.http.content_type)).unwrap();
    wr.write(b",\"user_agent\":").unwrap();
    (write!(wr, "\"{}\"", log.http.user_agent)).unwrap();
    wr.write(b",\"referer\":").unwrap();
    (write!(wr, "\"{}\"", log.http.referer)).unwrap();
    wr.write(b",\"request_uri\":").unwrap();
    (write!(wr, "\"{}\"", log.http.request_uri)).unwrap();

    wr.write(b"},\"origin\":{").unwrap();

    wr.write(b"\"ip\":").unwrap();
    (write!(wr, "\"{}\"", log.origin.ip)).unwrap();
    wr.write(b",\"port\":").unwrap();
    (write!(wr, "{}", log.origin.port)).unwrap();
    wr.write(b",\"hostname\":").unwrap();
    (write!(wr, "\"{}\"", log.origin.hostname)).unwrap();

    wr.write(b",\"protocol\":").unwrap();
    (write!(wr, "{}", log.origin.protocol as usize)).unwrap();

    wr.write(b"},\"country\":").unwrap();
    (write!(wr, "{}", log.country as usize)).unwrap();
    wr.write(b",\"cache_status\":").unwrap();
    (write!(wr, "{}", log.cache_status as usize)).unwrap();
    wr.write(b",\"server_ip\":").unwrap();
    (write!(wr, "\"{}\"", log.server_ip)).unwrap();
    wr.write(b",\"server_name\":").unwrap();
    (write!(wr, "\"{}\"", log.server_name)).unwrap();
    wr.write(b",\"remote_ip\":").unwrap();
    (write!(wr, "\"{}\"", log.remote_ip)).unwrap();
    wr.write(b",\"bytes_dlv\":").unwrap();
    (write!(wr, "{}", log.bytes_dlv)).unwrap();

    wr.write(b",\"ray_id\":").unwrap();
    (write!(wr, "\"{}\"", log.ray_id)).unwrap();
    wr.write(b"}").unwrap();
}

fn manual_serialize_escape<W: Write>(wr: &mut W, log: &Log) {
    wr.write_all(b"{").unwrap();
    escape_str(wr, "timestamp").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.timestamp)).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "zone_id").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.zone_id)).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "zone_plan").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.zone_plan as isize)).unwrap();

    wr.write_all(b",").unwrap();
    escape_str(wr, "http").unwrap();
    wr.write_all(b":{").unwrap();
    escape_str(wr, "protocol").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.http.protocol as usize)).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "status").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.http.status)).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "host_status").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.http.host_status)).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "up_status").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.http.up_status)).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "method").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.http.method as usize)).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "content_type").unwrap();
    wr.write_all(b":").unwrap();
    escape_str(wr, &log.http.content_type).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "user_agent").unwrap();
    wr.write_all(b":").unwrap();
    escape_str(wr, &log.http.user_agent).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "referer").unwrap();
    wr.write_all(b":").unwrap();
    escape_str(wr, &log.http.referer).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "request_uri").unwrap();
    wr.write_all(b":").unwrap();
    escape_str(wr, &log.http.request_uri).unwrap();

    wr.write_all(b"},").unwrap();
    escape_str(wr, "origin").unwrap();
    wr.write_all(b":{").unwrap();

    escape_str(wr, "ip").unwrap();
    wr.write_all(b":").unwrap();
    escape_str(wr, &log.origin.ip).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "port").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.origin.port)).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "hostname").unwrap();
    wr.write_all(b":").unwrap();
    escape_str(wr, &log.origin.hostname).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "protocol").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.origin.protocol as usize)).unwrap();

    wr.write_all(b"},").unwrap();
    escape_str(wr, "country").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.country as usize)).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "cache_status").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.cache_status as usize)).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "server_ip").unwrap();
    wr.write_all(b":").unwrap();
    escape_str(wr, &log.server_ip).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "server_name").unwrap();
    wr.write_all(b":").unwrap();
    escape_str(wr, &log.server_name).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "remote_ip").unwrap();
    wr.write_all(b":").unwrap();
    escape_str(wr, &log.remote_ip).unwrap();
    wr.write_all(b",").unwrap();
    escape_str(wr, "bytes_dlv").unwrap();
    wr.write_all(b":").unwrap();
    (write!(wr, "{}", log.bytes_dlv)).unwrap();

    wr.write_all(b",").unwrap();
    escape_str(wr, "ray_id").unwrap();
    wr.write_all(b":").unwrap();
    escape_str(wr, &log.ray_id).unwrap();
    wr.write_all(b"}").unwrap();
}

#[test]
fn test_manual_serialize_vec_no_escape() {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_serialize_no_escape(&mut wr, &log);

    let json = String::from_utf8(wr).unwrap();
    assert_eq!(&JSON_STR, &json);
}

#[bench]
fn bench_manual_serialize_vec_no_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_serialize_no_escape(&mut wr, &log);
    b.bytes = wr.len() as u64;

    b.iter(|| {
        wr.clear();
        manual_serialize_no_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_serialize_vec_escape() {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_serialize_escape(&mut wr, &log);

    let json = String::from_utf8(wr).unwrap();
    assert_eq!(&JSON_STR, &json);
}

#[bench]
fn bench_manual_serialize_vec_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_serialize_escape(&mut wr, &log);
    b.bytes = wr.len() as u64;

    b.iter(|| {
        wr.clear();

        manual_serialize_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_serialize_my_mem_writer0_no_escape() {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1000);
    manual_serialize_no_escape(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(&JSON_STR, &json);
}

#[bench]
fn bench_manual_serialize_my_mem_writer0_no_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);
    manual_serialize_no_escape(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        manual_serialize_no_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_serialize_my_mem_writer0_escape() {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);
    manual_serialize_escape(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(&JSON_STR, &json);
}

#[bench]
fn bench_manual_serialize_my_mem_writer0_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);
    manual_serialize_escape(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        manual_serialize_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_serialize_my_mem_writer1_no_escape() {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_serialize_no_escape(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(&JSON_STR, &json);
}

#[bench]
fn bench_manual_serialize_my_mem_writer1_no_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_serialize_no_escape(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        manual_serialize_no_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_serialize_my_mem_writer1_escape() {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_serialize_escape(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(&JSON_STR, &json);
}

#[bench]
fn bench_manual_serialize_my_mem_writer1_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_serialize_escape(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        manual_serialize_escape(&mut wr, &log);
    });
}

#[bench]
fn bench_decoder(b: &mut Bencher) {
    use rustc_serialize::json::Json;

    b.bytes = JSON_STR.len() as u64;

    b.iter(|| {
        let json = Json::from_str(JSON_STR).unwrap();
        let mut decoder = rustc_serialize::json::Decoder::new(json);
        let _log: Log = rustc_serialize::Decodable::decode(&mut decoder).unwrap();
    });
}

#[bench]
fn bench_deserializer(b: &mut Bencher) {
    b.bytes = JSON_STR.len() as u64;

    b.iter(|| {
        let _log: Log = json::from_str(JSON_STR).unwrap();
    });
}
