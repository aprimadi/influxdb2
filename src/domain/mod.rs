enum DialectAnnotations {
    Datatype,
    Default,
    Group,
}

impl fmt::Display for DialectAnnotations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            self::Datatype  => write!(f, "{}", "datatype"),
            self::Default   => write!(f, "{}", "default"),
            self::Group     => write!(f, "{}", "group"),
        }
    }
}

struct Dialect {
    annotations:    Vec<DialectAnnotations>,
    delimiter:      String,
    header:         bool,
}

