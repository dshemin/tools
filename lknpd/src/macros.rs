/// Создаёт новый доменный тип.
#[macro_export]
macro_rules! newtype {
    ($tname:ident, String, "String", $validate_fn:ident) => {
        #[derive(
            std::fmt::Debug,
            serde::Serialize,
            serde::Deserialize,
            std::cmp::PartialEq,
            std::cmp::Eq,
            std::default::Default,
        )]
        #[serde(try_from = "String")]
        pub struct $tname(String);

        impl $tname {
            pub fn new(value: &str) -> anyhow::Result<Self> {
                Self::validate(&value)?;
                Ok(Self(value.into()))
            }

            fn validate(value: &str) -> anyhow::Result<()> {
                $validate_fn(value)
            }
        }

        #[allow(clippy::from_over_into)]
        impl std::convert::Into<String> for $tname {
            fn into(self) -> String {
                self.0
            }
        }

        impl std::convert::TryFrom<String> for $tname {
            type Error = anyhow::Error;

            fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
                $tname::new(&value)
            }
        }
    };

    ($tname:ident, $type:ty, $try_from:literal, $validate_fn:ident) => {
        #[derive(
            std::fmt::Debug,
            serde::Serialize,
            serde::Deserialize,
            std::cmp::PartialEq,
            std::cmp::Eq,
            std::default::Default,
        )]
        #[serde(try_from = $try_from)]
        pub struct $tname($type);

        impl $tname {
            pub fn new(value: $type) -> anyhow::Result<Self> {
                Self::validate(&value)?;
                Ok(Self(value.into()))
            }

            fn validate(value: &$type) -> anyhow::Result<()> {
                $validate_fn(value)
            }
        }

        #[allow(clippy::from_over_into)]
        impl std::convert::Into<$type> for $tname {
            fn into(self) -> $type {
                self.0
            }
        }

        impl std::convert::TryFrom<$type> for $tname {
            type Error = anyhow::Error;

            fn try_from(value: $type) -> std::result::Result<Self, Self::Error> {
                $tname::new(value)
            }
        }
    };

    ($tname:ident, $type:ty, $try_from:literal) => {
        #[derive(
            std::fmt::Debug,
            serde::Serialize,
            serde::Deserialize,
            std::cmp::PartialEq,
            std::cmp::Eq,
            std::default::Default,
        )]
        #[serde(try_from = $try_from)]
        pub struct $tname($type);

        impl $tname {
            pub fn new(value: $type) -> Self {
                Self(value.into())
            }
        }

        #[allow(clippy::from_over_into)]
        impl std::convert::Into<$type> for $tname {
            fn into(self) -> $type {
                self.0
            }
        }

        impl std::convert::From<$type> for $tname {
            fn from(value: $type) -> Self {
                $tname::new(value)
            }
        }

        impl std::str::FromStr for $tname {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok($tname::new(<$type>::from_str(s)?))
            }
        }
    };
}
