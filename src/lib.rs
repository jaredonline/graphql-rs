#[macro_use]
extern crate log;
extern crate env_logger;

pub mod language;

pub enum AnyType {
    Func,
    String,
    None,
    Integer,
}

pub struct Any {
    _type:   AnyType,
    _func:   Option<Box<Fn(Any) -> Any>>,
    _string: Option<String>,
    _int:    Option<i32>,
}

impl Any {
    pub fn string(s: String) -> Any {
        Any {
            _type: AnyType::String,
            _func: None,
            _string: Some(s),
            _int: None,
        }
    }

    pub fn str(s: &str) -> Any {
        Any {
            _type: AnyType::String,
            _func: None,
            _string: Some(s.to_string()),
            _int: None,
        }
    }

    pub fn func(s: Box<Fn(Any) -> Any>) -> Any {
        Any {
            _type: AnyType::Func,
            _func: Some(s),
            _string: None,
            _int: None,
        }
    }

    pub fn int(s: i32) -> Any {
        Any {
            _type: AnyType::Integer,
            _func: None,
            _string: None,
            _int: Some(s)
        }
    }

    pub fn none() -> Any {
        Any {
            _type: AnyType::None,
            _func: None,
            _string: None,
            _int: None,
        }
    }

    pub fn coerce_string(&self) -> Option<String> {
        match self._type {
            AnyType::String => {
                self._string.clone()
            },
            _ => None
        }
    }

    pub fn coerce_int(&self) -> Option<i32> {
        match self._type {
            AnyType::Integer => {
                self._int
            },
            _ => None,
        }
    }
}

pub struct Scalar<T> {
    name: String,
    description: String,
    coerce: Box<Fn(Any) -> Option<T>>
}

impl <T> Scalar<T> {
    pub fn coerce(&self, value: Any) -> Option<T> {
        let ref fun = self.coerce;
        fun(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn build_a_scalar() {
        use Scalar;

        let scalar = Scalar {
            name: "Int".to_string(),
            description: "Integer scalar type.".to_string(),
            coerce: Box::new(|val: Any| -> Option<i32> {
                val.coerce_int()
            }),
            //coerce_literal: None,
        };

        assert_eq!(scalar.coerce(Any::str("Foo")), None);
        assert_eq!(scalar.coerce(Any::int(1)), Some(1));

        //assert_eq!(config.name, "Int".to_string());
        //assert_eq!(config.description.unwrap(), "Integer scalar type.".to_string());
        //let coerce = config.coerce.unwrap();
        //let val    = coerce(Any::str("Foo"));
        //let st     = val.coerce_string().unwrap();
        //assert_eq!(st, "Foo".to_string());
    }
}
