// class name, ctor name, field names
type TreeConfig = (&'static str, &'static str, Vec<&'static str>);

fn configs() -> Vec<TreeConfig> {
    vec![
        // ("Class", "ctor", vec!["fields"]),
        ("Query", "query", vec!["with", "query_no_with"]),
        ("With", "with", vec!["with", "recursive", "named_queries"]),
        (
            "NamedQuery",
            "named_query",
            vec![
                "name",
                "column_aliases_opt",
                "as_",
                "open_paren",
                "query",
                "close_paren",
            ],
        ),
        (
            "QueryNoWith",
            "query_no_with",
            vec!["query_term", "order_by_opt", "limit_opt"],
        ),
        ("OrderBy", "order_by", vec!["order", "by", "sort_items"]),
        ("Limit", "limit", vec!["limit", "value"]),
        (
            "QuerySetOperation",
            "query_set_operation",
            vec!["left", "operator", "set_quantifier_opt", "right"],
        ),
        ("SortItem", "sort_item", vec!["expression", "ordering_opt", "nulls", "null_ordering_opt"]),
        ("Subquery", "subquery", vec!["open_paren", "query_no_with", "close_paren"]),
        ("InlineTable", "inline_table", vec!["values", "expressions"]),
        ("Table", "table", vec!["table", "qualified_name"]),
        ("QuerySpecification", "query_specification", vec![
            "select", 
            "set_quantifier_opt",
            "select_items",
            "from",
            "relations",
            "where_",
            "where_predicate",
            "group",
            "by",
            "group_by",
            "having",
            "having_predicate",
        ]),
        ("QualifiedName", "qualified_name", vec!["names"]),
        ("SelectAll", "select_all", vec!["asterisk"]),
        ("QualifiedSelectAll", "qualified_select_all", vec!["qualifier", "period", "asterisk"]),
        ("SelectItem", "select_item", vec!["expression", "as_", "identifier"]),
        // ("Class", "ctor", vec!["fields"]),
    ]
}

const FILE_HEADER: &str = r#"use crate::lexing::{text_range::TextRange, token};

#[derive(Clone, Debug)]
pub enum ParseTree<'a> {
    // The core trees
    Empty(Empty),
    Token(Token<'a>),
    List(List<'a>),
    Error(Error),

    // The language specific trees
"#;

const END: &str = "}\n\n";

const STRUCT_HEADER: &str = r#"#[derive(Clone, Debug)]
pub struct "#;

const EMPTY_DEFINITION: &str = r#"Empty {
    pub range: TextRange,
}

pub fn empty<'a>(range: TextRange) -> ParseTree<'a> {
    ParseTree::Empty(Empty { range })
}

"#;

const TOKEN_DEFINITION: &str = r#"Token<'a> {
    pub token: token::Token<'a>,
}

pub fn token<'a>(token: token::Token<'a>) -> ParseTree<'a> {
    ParseTree::Token(Token { token })
}

"#;

const LIST_DEFINITION: &str = r#"List<'a> {
    pub start_delimiter: Box<ParseTree<'a>>,
    pub elements_and_separators: Vec<(ParseTree<'a>, ParseTree<'a>)>,
    pub end_delimiter: Box<ParseTree<'a>>,
}

pub fn list<'a>(
    start_delimiter: ParseTree<'a>,
    elements_and_separators: Vec<(ParseTree<'a>, ParseTree<'a>)>,
    end_delimiter: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::List(List {
        start_delimiter: Box::new(start_delimiter),
        elements_and_separators,
        end_delimiter: Box::new(end_delimiter),
    })
}

"#;

const ERROR_DEFINITION: &str = r#"Error {
    pub range: TextRange,
    pub message: String,
}

pub fn error<'a>(range: TextRange, message: String) -> ParseTree<'a> {
    ParseTree::Error(Error {
        range,
        message: message,
    })
}

"#;

const CORE_IMPL: &str = r#"// core impl
impl<'a> ParseTree<'a> {
    pub fn is_empty(&self) -> bool {
        if let ParseTree::Empty(_) = self { true } else { false }
    }
}

"#;

fn main() {
    let cs = configs();

    // enum ParseTree
    print!("{}", FILE_HEADER);
    for config in &cs {
        let class_name = config.0;
        print!("    {}({}<'a>),\n", class_name, class_name);
    }
    print!("{}", END);

    print!("{}", "// The core trees\n");
    print!("{}{}", STRUCT_HEADER, EMPTY_DEFINITION);
    print!("{}{}", STRUCT_HEADER, TOKEN_DEFINITION);
    print!("{}{}", STRUCT_HEADER, LIST_DEFINITION);
    print!("{}{}", STRUCT_HEADER, ERROR_DEFINITION);
    print!("{}", CORE_IMPL);

    print!("{}", "// The language specific trees\n");
    for config in &cs {
        let (class_name, ctor_name, fields) = &config;

        // struct definition
        print!("{}{}<'a> {{\n", STRUCT_HEADER, class_name);
        for field_name in fields {
            print!("    pub {}: Box<ParseTree<'a>>,\n", field_name);
        }
        print!("{}", END);

        // constructor definition
        print!("pub fn {}<'a>(\n", ctor_name);
        for field_name in fields {
            print!("    {}: ParseTree<'a>,\n", field_name);
        }
        print!(") -> ParseTree<'a> {{\n");
        print!("    ParseTree::{}({} {{\n", class_name, class_name);
        for field_name in fields {
            print!("        {}: Box::new({}),\n", field_name, field_name);
        }
        print!("    }})\n");
        print!("{}", END);
    }
}
