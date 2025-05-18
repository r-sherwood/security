use diesel::deserialize;
use diesel::deserialize::FromSqlRow;
use diesel::mysql::Mysql;
use diesel::mysql::MysqlValue;
use diesel::serialize;
use diesel::serialize::IsNull;
use diesel::sql_types::Text;
use hickory_resolver::Name;
use rocket::request::FromParam;
use std::str::FromStr;

use serde::{Deserialize, Deserializer};
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScannerData<'a> {
    pub static_file_name: Option<&'a str>,
    pub funny_name: &'a str,
    pub display_name: &'a str,
    pub value: &'a str,
    pub dns_prefix: Option<&'a str>,
}

pub const STRETCHOID: ScannerData = ScannerData {
    static_file_name: None,
    funny_name: "stretchoid agent",
    display_name: "stretchoid",
    value: "stretchoid",
    dns_prefix: Some("stretchoid.com."),
};
pub const BINARYEDGE: ScannerData = ScannerData {
    static_file_name: None,
    funny_name: "binaryedge ninja",
    display_name: "binaryedge",
    value: "binaryedge",
    dns_prefix: Some("binaryedge.ninja."),
};

pub const SHADOWSERVER: ScannerData = ScannerData {
    static_file_name: None,
    funny_name: "cloudy shadowserver",
    display_name: "shadowserver",
    value: "shadowserver",
    dns_prefix: Some("shadowserver.org."),
};

pub fn get_scanners() -> Vec<ScannerData<'static>> {
    vec![
        STRETCHOID,
        BINARYEDGE,
        SHADOWSERVER,
        ScannerData {
            static_file_name: Some("censys.txt"),
            funny_name: "Censys node",
            display_name: "censys",
            value: "censys",
            dns_prefix: None,
        },
        ScannerData {
            static_file_name: Some("internet-measurement.com.txt"),
            funny_name: "internet measurement probe",
            display_name: "internet-measurement.com",
            value: "internet-measurement.com",
            dns_prefix: None,
        },
        ScannerData {
            static_file_name: Some("anssi.txt"),
            funny_name: "French ANSSI probe",
            display_name: "anssi",
            value: "anssi",
            dns_prefix: None,
        },
    ]
}

pub type ScannerNode = ScannersWrapper<ScannerData<'static>>;

#[derive(Debug, Clone, Copy, FromSqlRow, PartialEq)]
pub struct ScannersWrapper<ScannerData> {
    pub info: ScannerData,
}

pub trait ScannerMethods {
    fn is_static(self: &Self) -> bool;
    fn static_file_name(self: &Self) -> Option<&str>;
    fn funny_name(self: &Self) -> &str;
}

impl ScannerMethods for ScannerNode {
    fn is_static(self: &Self) -> bool {
        self.static_file_name().is_some()
    }

    fn static_file_name(self: &Self) -> Option<&str> {
        self.info.static_file_name
    }

    fn funny_name(self: &Self) -> &str {
        self.info.funny_name
    }
}

impl FromParam<'_> for ScannerNode {
    type Error = String;

    fn from_param(param: &'_ str) -> Result<Self, Self::Error> {
        param.try_into()
    }
}

impl<'de> Deserialize<'de> for ScannerNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <Vec<String>>::deserialize(deserializer)?;
        let k: &str = s[0].as_str();
        match k.try_into() {
            Ok(scanners) => Ok(scanners),
            Err(v) => Err(serde::de::Error::custom(format!("Unknown value: {}", v))),
        }
    }
}

impl ToString for ScannerNode {
    fn to_string(&self) -> String {
        let res: &str = (*self).into();
        res.to_string()
    }
}

impl Into<&str> for ScannerNode {
    fn into(self) -> &'static str {
        self.info.display_name
    }
}

impl serialize::ToSql<Text, Mysql> for ScannerNode {
    fn to_sql(&self, out: &mut serialize::Output<Mysql>) -> serialize::Result {
        let res: &str = (*self).into();
        out.write_all(res.as_bytes())?;

        Ok(IsNull::No)
    }
}

impl deserialize::FromSql<Text, Mysql> for ScannerNode {
    fn from_sql(bytes: MysqlValue) -> deserialize::Result<Self> {
        let value = <String as deserialize::FromSql<Text, Mysql>>::from_sql(bytes)?;
        let value = &value as &str;
        let value: Result<ScannerNode, String> = value.try_into();
        match value {
            Ok(d) => Ok(d),
            Err(err) => Err(err.into()),
        }
    }
}

// Used for FromSql & FromParam & Deserialize
impl TryInto<ScannerNode> for &str {
    type Error = String;

    fn try_into(self) -> Result<ScannerNode, Self::Error> {
        let value: String = self.replace(".txt", "").as_str().to_string();
        match get_scanners()
            .iter()
            .find(|scanner| scanner.value.eq(&value))
        {
            Some(scanner) => Ok(ScannersWrapper { info: *scanner }),
            None => Err(format!("Invalid value: {value}")),
        }
    }
}

// Used by the DNS logic
impl TryInto<ScannerNode> for Name {
    type Error = String;

    fn try_into(self) -> Result<ScannerNode, Self::Error> {
        let short_name = self.trim_to(2);
        match get_scanners()
            .iter()
            .filter(|scanner| scanner.dns_prefix.is_some())
            .find(|scanner| {
                short_name.eq_case(
                    &Name::from_str(scanner.dns_prefix.expect("Should have a DNS prefix"))
                        .expect("Should parse"),
                )
            }) {
            Some(scanner) => Ok(ScannersWrapper { info: *scanner }),
            None => Err(format!("Invalid hostname: {self}")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_detect_scanner_from_name() {
        let ptr = Name::from_str("scan-47e.shadowserver.org.").unwrap();

        let res: Result<ScannerNode, String> = ptr.try_into();

        assert_eq!(res.unwrap().info, SHADOWSERVER);
    }
}
