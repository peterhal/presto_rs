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
        (
            "SortItem",
            "sort_item",
            vec!["expression", "ordering_opt", "nulls", "null_ordering_opt"],
        ),
        (
            "Subquery",
            "subquery",
            vec!["open_paren", "query_no_with", "close_paren"],
        ),
        ("InlineTable", "inline_table", vec!["values", "expressions"]),
        ("Table", "table", vec!["table", "qualified_name"]),
        (
            "QuerySpecification",
            "query_specification",
            vec![
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
            ],
        ),
        ("QualifiedName", "qualified_name", vec!["names"]),
        ("SelectAll", "select_all", vec!["asterisk"]),
        (
            "QualifiedSelectAll",
            "qualified_select_all",
            vec!["qualifier", "period", "asterisk"],
        ),
        (
            "SelectItem",
            "select_item",
            vec!["expression", "as_", "identifier"],
        ),
        (
            "SubqueryRelation",
            "subquery_relation",
            vec!["open_paren", "query", "close_paren"],
        ),
        (
            "ParenthesizedRelation",
            "parenthesized_relation",
            vec!["open_paren", "relation", "close_paren"],
        ),
        ("TableName", "table_name", vec!["name"]),
        (
            "Lateral",
            "lateral",
            vec!["lateral", "open_paren", "query", "close_paren"],
        ),
        (
            "Unnest",
            "unnest",
            vec!["unnest", "expressions", "with", "ordinality"],
        ),
        (
            "SampledRelation",
            "sampled_relation",
            vec![
                "aliased_relation",
                "tablesample",
                "sample_type",
                "open_paren",
                "expression",
                "close_paren",
            ],
        ),
        (
            "AliasedRelation",
            "aliased_relation",
            vec![
                "relation_primary",
                "as_opt",
                "identifier",
                "column_aliases_opt",
            ],
        ),
        (
            "CrossJoin",
            "cross_join",
            vec!["left", "cross", "join", "right"],
        ),
        (
            "Join",
            "join",
            vec!["left", "join_type", "join", "right", "join_criteria"],
        ),
        (
            "NaturalJoin",
            "natural_join",
            vec!["left", "natural", "join_type", "join", "right"],
        ),
        (
            "OuterJoinKind",
            "outer_join_kind",
            vec!["kind", "outer_opt"],
        ),
        (
            "OnJoinCriteria",
            "on_join_criteria",
            vec!["on", "predicate"],
        ),
        (
            "UsingJoinCriteria",
            "using_join_criteria",
            vec!["using", "names"],
        ),
        (
            "GroupBy",
            "group_by",
            vec!["set_quantifier_opt", "grouping_elements"],
        ),
        ("Rollup", "rollup", vec!["rollup", "expressions"]),
        ("Cube", "cube", vec!["cube", "expressions"]),
        (
            "GroupingSets",
            "grouping_sets",
            vec!["grouping", "sets", "grouping_sets"],
        ),
        (
            "BinaryExpression",
            "binary_expression",
            vec!["left", "operator", "right"],
        ),
        (
            "UnaryExpression",
            "unary_expression",
            vec!["operator", "operand"],
        ),
        (
            "QuanitifiedComparison",
            "quantified_comparison",
            vec![
                "operand",
                "operator",
                "comparison_quantifier",
                "open_paren",
                "query",
                "close_paren",
            ],
        ),
        (
            "NullPredicate",
            "null_predicate",
            vec!["value", "is", "not_opt", "null"],
        ),
        (
            "DistinctFrom",
            "distinct_from",
            vec!["left", "distinct", "from", "right"],
        ),
        (
            "Between",
            "between",
            vec!["value", "not_opt", "between", "lower", "and", "upper"],
        ),
        (
            "Like",
            "like",
            vec![
                "value",
                "not_opt",
                "like",
                "patrern",
                "escape_opt",
                "escape_value_opt",
            ],
        ),
        (
            "InSubquery",
            "in_subquery",
            vec![
                "value",
                "not_opt",
                "in_",
                "open_paren",
                "query",
                "close_paren",
            ],
        ),
        (
            "InList",
            "in_list",
            vec!["value", "not_opt", "in_", "expressions"],
        ),
        (
            "AtTimeZone",
            "at_time_zone",
            vec!["value", "at", "time", "zone", "specifier"],
        ),
        (
            "Dereference",
            "dereference",
            vec!["object", "period", "field_name"],
        ),
        (
            "Subscript",
            "subscript",
            vec!["operand", "open_square", "index", "close_square"],
        ),
        ("Lambda", "lambda", vec!["parameters", "array", "body"]),
        ("Literal", "literal", vec!["value"]),
        ("RowConstructor", "row_constructor", vec!["elements"]),
        (
            "ParenthesizedExpression",
            "parenthesized_expression",
            vec!["open_paren", "value", "close_paren"],
        ),
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

impl<'a> List<'a> {
    pub fn len(&self) -> usize {
        self.elements_and_separators.len()
    }
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
"#;

fn print_is_as_impl(ctor_name: &str, class_name: &str) {
    // is_*
    print!(
        r#"    pub fn is_{}(&self) -> bool {{
        if let ParseTree::{}(_) = self {{ true }} else {{ false }}
    }}

    "#,
        ctor_name, class_name
    );

    // as_*
    print!(
        r#"    pub fn as_{0}(&self) -> &{1} {{
        if let ParseTree::{1}(value) = self {{
            value
        }} else {{
            panic!("Expected {1}")
        }}
    }}

    "#,
        ctor_name, class_name
    );
}

fn main() {
    let cs = configs();

    // enum ParseTree
    print!("{}", FILE_HEADER);
    for config in &cs {
        let class_name = config.0;
        print!("    {0}({0}<'a>),\n", class_name);
    }
    print!("{}", END);

    print!("{}", "// The core trees\n");
    print!("{}{}", STRUCT_HEADER, EMPTY_DEFINITION);
    print!("{}{}", STRUCT_HEADER, TOKEN_DEFINITION);
    print!("{}{}", STRUCT_HEADER, LIST_DEFINITION);
    print!("{}{}", STRUCT_HEADER, ERROR_DEFINITION);

    // fn is_*()
    print!("{}", CORE_IMPL);
    print_is_as_impl("list", "List");
    print_is_as_impl("empty", "Empty");
    print_is_as_impl("token", "Token");
    print_is_as_impl("error", "Error");
    for config in &cs {
        let (class_name, ctor_name, _) = &config;
        print_is_as_impl(ctor_name, class_name);
    }
    print!("{}", END);

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
        print!("    ParseTree::{0}({0} {{\n", class_name);
        for field_name in fields {
            print!("        {0}: Box::new({0}),\n", field_name);
        }
        print!("    }})\n");
        print!("{}", END);

        // tree impl
        print!(
            r#"impl<'a> {0}<'a> {{
    pub fn to_tree(self) -> ParseTree<'a> {{
        ParseTree::{0}(self)
    }}
}}

"#,
            class_name
        );
    }
}
