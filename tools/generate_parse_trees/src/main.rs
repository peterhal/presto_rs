// class name, ctor name, field names
type TreeConfig = (&'static str, Vec<&'static str>);

fn configs() -> Vec<TreeConfig> {
    vec![
        // ("Class", vec!["fields"]),
        ("Query", vec!["with", "query_no_with"]),
        ("With", vec!["with", "recursive", "named_queries"]),
        (
            "NamedQuery",
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
            vec!["query_term", "order_by_opt", "limit_opt"],
        ),
        ("OrderBy", vec!["order", "by", "sort_items"]),
        ("Limit", vec!["limit", "value"]),
        (
            "QuerySetOperation",
            vec!["left", "operator", "set_quantifier_opt", "right"],
        ),
        (
            "SortItem",
            vec!["expression", "ordering_opt", "nulls", "null_ordering_opt"],
        ),
        (
            "Subquery",
            vec!["open_paren", "query_no_with", "close_paren"],
        ),
        ("InlineTable", vec!["values", "expressions"]),
        ("Table", vec!["table", "qualified_name"]),
        (
            "QuerySpecification",
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
        ("QualifiedName", vec!["names"]),
        ("SelectAll", vec!["asterisk"]),
        (
            "QualifiedSelectAll",
            vec!["qualifier", "period", "asterisk"],
        ),
        ("SelectItem", vec!["expression", "as_", "identifier"]),
        (
            "SubqueryRelation",
            vec!["open_paren", "query", "close_paren"],
        ),
        (
            "ParenthesizedRelation",
            vec!["open_paren", "relation", "close_paren"],
        ),
        ("TableName", vec!["name"]),
        (
            "Lateral",
            vec!["lateral", "open_paren", "query", "close_paren"],
        ),
        (
            "Unnest",
            vec!["unnest", "expressions", "with", "ordinality"],
        ),
        (
            "SampledRelation",
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
            vec![
                "relation_primary",
                "as_opt",
                "identifier",
                "column_aliases_opt",
            ],
        ),
        ("CrossJoin", vec!["left", "cross", "join", "right"]),
        (
            "Join",
            vec!["left", "join_type", "join", "right", "join_criteria"],
        ),
        (
            "NaturalJoin",
            vec!["left", "natural", "join_type", "join", "right"],
        ),
        ("OuterJoinKind", vec!["kind", "outer_opt"]),
        ("OnJoinCriteria", vec!["on", "predicate"]),
        ("UsingJoinCriteria", vec!["using", "names"]),
        ("GroupBy", vec!["set_quantifier_opt", "grouping_elements"]),
        ("Rollup", vec!["rollup", "expressions"]),
        ("Cube", vec!["cube", "expressions"]),
        ("GroupingSets", vec!["grouping", "sets", "grouping_sets"]),
        ("BinaryExpression", vec!["left", "operator", "right"]),
        ("UnaryExpression", vec!["operator", "operand"]),
        (
            "QuantifiedComparison",
            vec![
                "operand",
                "operator",
                "comparison_quantifier",
                "open_paren",
                "query",
                "close_paren",
            ],
        ),
        ("NullPredicate", vec!["value", "is", "not_opt", "null"]),
        ("DistinctFrom", vec!["left", "distinct", "from", "right"]),
        (
            "Between",
            vec!["value", "not_opt", "between", "lower", "and", "upper"],
        ),
        (
            "Like",
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
            vec![
                "value",
                "not_opt",
                "in_",
                "open_paren",
                "query",
                "close_paren",
            ],
        ),
        ("InList", vec!["value", "not_opt", "in_", "expressions"]),
        (
            "AtTimeZone",
            vec!["value", "at", "time", "zone", "specifier"],
        ),
        ("Dereference", vec!["object", "period", "field_name"]),
        (
            "Subscript",
            vec!["operand", "open_square", "index", "close_square"],
        ),
        ("Lambda", vec!["parameters", "array", "body"]),
        ("Literal", vec!["value"]),
        ("RowConstructor", vec!["elements"]),
        (
            "ParenthesizedExpression",
            vec!["open_paren", "value", "close_paren"],
        ),
        ("Identifier", vec!["value"]),
        (
            "FunctionCall",
            vec![
                "name",
                "open_paren",
                "set_quantifier_opt",
                "arguments",
                "order_by_opt",
                "close_paren",
                "filter_opt",
                "over_opt",
            ],
        ),
        (
            "Filter",
            vec!["filter", "open_paren", "where_", "predicate", "close_paren"],
        ),
        (
            "Over",
            vec![
                "over",
                "open_paren",
                "partition_opt",
                "by",
                "partitions",
                "order_by_opt",
                "window_frame",
                "close_paren",
            ],
        ),
        (
            "WindowFrame",
            vec!["frame_type", "between_opt", "start", "and", "end"],
        ),
        ("UnboundedFrame", vec!["unbounded", "bound_type"]),
        ("CurrentRowBound", vec!["current", "row"]),
        ("BoundedFrame", vec!["bound", "bound_type"]),
        ("UnicodeString", vec!["string", "uescape_opt", "escape"]),
        (
            "ConfigureExpression",
            vec![
                "configure",
                "open_paren",
                "identifier",
                "comma",
                "value",
                "close_paren",
            ],
        ),
        (
            "SubqueryExpression",
            vec!["open_paren", "query", "close_paren"],
        ),
        ("Grouping", vec!["grouping", "groups"]),
        (
            "Extract",
            vec![
                "extract",
                "open_paren",
                "identifier",
                "from",
                "value",
                "close_paren",
            ],
        ),
        (
            "CurrentTime",
            vec!["current_time", "open_paren", "precision", "close_paren"],
        ),
        (
            "CurrentTimestamp",
            vec![
                "current_timestamp",
                "open_paren",
                "precision",
                "close_paren",
            ],
        ),
        (
            "Normalize",
            vec![
                "normalize",
                "open_paren",
                "value",
                "comma_opt",
                "normal_form",
                "close_paren",
            ],
        ),
        (
            "Localtime",
            vec!["localtime", "open_paren", "precision", "close_paren"],
        ),
        (
            "Localtimestamp",
            vec!["localtimestamp", "open_paren", "precision", "close_paren"],
        ),
        (
            "Cast",
            vec!["cast", "open_paren", "value", "as_", "type_", "close_paren"],
        ),
        ("WhenClause", vec!["when", "condition", "then", "result"]),
        (
            "Case",
            vec![
                "case",
                "value_opt",
                "when_clauses",
                "else_opt",
                "default",
                "end",
            ],
        ),
        (
            "Exists",
            vec!["exists", "open_paren", "query", "close_paren"],
        ),
        ("TypeConstructor", vec!["type_", "value"]),
        ("Array", vec!["array", "elements"]),
        (
            "Interval",
            vec!["interval", "sign_opt", "value", "from", "to_kw_opt", "to"],
        ),
        ("Row", vec!["row", "elements"]),
        (
            "TryCast",
            vec![
                "try_cast",
                "open_paren",
                "value",
                "as_",
                "type_",
                "close_paren",
            ],
        ),
        (
            "Substring",
            vec![
                "substring",
                "open_paren",
                "value",
                "from",
                "from_value",
                "for_opt",
                "for_value",
                "close_paren",
            ],
        ),
        (
            "Position",
            vec![
                "position",
                "open_paren",
                "value",
                "in_",
                "target",
                "close_paren",
            ],
        ),
        ("ArrayTypeSuffix", vec!["base_type", "array"]),
        ("NamedType", vec!["name", "type_parameters"]),
        (
            "ArrayType",
            vec!["array", "open_angle", "element_type", "close_angle"],
        ),
        (
            "MapType",
            vec![
                "map",
                "open_angle",
                "key_type",
                "comma",
                "value_type",
                "close_angle",
            ],
        ),
        ("RowType", vec!["row", "element_types"]),
        ("RowTypeElement", vec!["identifier", "type_"]),
        ("IntervalType", vec!["interval", "from", "to_kw", "to"]),
        ("IfNotExists", vec!["if_", "not", "exists"]),
        (
            "CreateTable",
            vec![
                "create",
                "table",
                "if_not_exists_opt",
                "table_name",
                "table_elements",
                "comment_opt",
                "with_properties_opt",
            ],
        ),
        (
            "CreateTableAsSelect",
            vec![
                "create",
                "table",
                "if_not_exists_opt",
                "table_name",
                "column_aliases_opt",
                "comment_opt",
                "with_properties_opt",
                "as_",
                "open_paren_opt",
                "query",
                "close_paren_opt",
                "with_data_opt",
            ],
        ),
        ("WithProperties", vec!["with", "properties"]),
        ("Property", vec!["identifier", "eq", "value"]),
        ("WithData", vec!["with", "no_opt", "data"]),
        ("Comment", vec!["comment", "value"]),
        (
            "ColumnDefinition",
            vec![
                "identifier",
                "type_",
                "not_null_opt",
                "comment_opt",
                "with_properties_opt",
            ],
        ),
        ("NotNull", vec!["not", "null"]),
        (
            "LikeClause",
            vec!["like", "name", "option_type_opt", "properties"],
        ),
        (
            "InsertInto",
            vec![
                "insert",
                "into",
                "table_name",
                "column_aliases_opt",
                "query",
            ],
        ),
        (
            "Delete",
            vec!["delete", "from", "table_name", "where_opt", "predicate"],
        ),
        ("GroupingSet", vec!["elements"]),
        // ("Class", vec![]),
    ]
}

const FILE_HEADER: &str = r#"use crate::lexing::{syntax_error::SyntaxError, text_range::TextRange, token};

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

impl Empty {
    pub fn children(&self) -> Vec<&'static ParseTree<'static>> {
        Vec::new()
    }
}

"#;

const TOKEN_DEFINITION: &str = r#"Token<'a> {
    pub token: token::Token<'a>,
}

pub fn token<'a>(token: token::Token<'a>) -> ParseTree<'a> {
    ParseTree::Token(Token { token })
}

impl<'a> Token<'a> {
    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        Vec::new()
    }
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

    pub fn unbox(self) -> (ParseTree<'a>, Vec<(ParseTree<'a>, ParseTree<'a>)>, ParseTree<'a>) {
        (
            *self.start_delimiter,
            self.elements_and_separators,
            *self.end_delimiter,
        )
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2 + self.elements_and_separators.len() * 2);
        result.push(&*self.start_delimiter);
        for (element, separator) in &self.elements_and_separators {
            result.push(&element);
            result.push(&separator);
        }
        result.push(&*self.end_delimiter);
        result
    }
}

"#;

const ERROR_DEFINITION: &str = r#"Error {
    pub error: SyntaxError,
}

pub fn error<'a>(error: SyntaxError) -> ParseTree<'a> {
    ParseTree::Error(Error {
        error,
    })
}

impl Error {
    pub fn children(&self) -> Vec<&'static ParseTree<'static>> {
        Vec::new()
    }
}

"#;

const CORE_IMPL: &str = r#"// core impl

impl<'a> ParseTree<'a> {
    pub fn unbox_list(self) -> (ParseTree<'a>, Vec<(ParseTree<'a>, ParseTree<'a>)>, ParseTree<'a>) {
        match self {
            ParseTree::List(value) => value.unbox(),
            _ => panic!("Expected List"),
        }
    }

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

fn print_unbox(ctor_name: &str, class_name: &str, fields: &Vec<&str>) {
    // unbox
    print!("    pub fn unbox_{}(self) -> (\n", ctor_name);
    for _ in 0..fields.len() {
        print!("        ParseTree<'a>,\n");
    }
    print!(") {{\n");
    print!(
        r#"        match self {{
            ParseTree::{0}(tree) => tree.unbox(),
            _ => panic!("Expected {0}"),
        }}
    }}

"#,
        class_name
    );
}

fn get_config<'a>(config: &'a TreeConfig) -> (&'static str, String, &'a Vec<&'static str>) {
    let mut ctor_name = String::new();
    for ch in config.0.chars() {
        if ch.is_ascii_uppercase() {
            if ctor_name.len() > 0 {
                ctor_name.push('_');
            }
            ctor_name.push(ch.to_ascii_lowercase());
        } else {
            assert!(ch.is_ascii_lowercase());
            ctor_name.push(ch);
        }
    }
    (config.0, ctor_name, &config.1)
}

fn main() {
    let cs = configs();

    // enum ParseTree
    print!("{}", FILE_HEADER);
    for config in &cs {
        let class_name = get_config(config).0;
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
        let (class_name, ctor_name, fields) = get_config(config);
        print_is_as_impl(ctor_name.as_str(), class_name);
        print_unbox(ctor_name.as_str(), class_name, fields);
    }
    // children
    print!("    pub fn children(&self) -> Vec<&ParseTree<'a>> {{\n");
    print!("        match self {{\n");
    print!("            ParseTree::Token(token) => token.children(),\n");
    print!("            ParseTree::List(list) => list.children(),\n");
    print!("            ParseTree::Error(error) => error.children(),\n");
    print!("            ParseTree::Empty(empty) => empty.children(),\n");
    for config in &cs {
        let (class_name, ctor_name, _) = get_config(config);
        print!(
            "            ParseTree::{0}({1}) => {1}.children(),",
            class_name, ctor_name
        );
    }
    print!("        }}\n");

    print!("{}", END);

    print!("{}", END);

    print!("{}", "// The language specific trees\n");
    for config in &cs {
        let (class_name, ctor_name, fields) = get_config(config);

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
        // to_tree
        print!("impl<'a> {}<'a> {{\n", class_name);
        print!(
            r#"    pub fn to_tree(self) -> ParseTree<'a> {{
        ParseTree::{}(self)
    }}

"#,
            class_name
        );

        // children
        print!("    pub fn children(&self) -> Vec<&ParseTree<'a>> {{\n");
        print!(
            "        let mut result = Vec::with_capacity({});",
            fields.len()
        );
        for field_name in fields {
            print!("        result.push(&*self.{0});\n", field_name);
        }
        print!("        result");
        print!("{}", END);

        // unbox
        print!("    pub fn unbox(self) -> (\n");
        for _ in 0..fields.len() {
            print!("        ParseTree<'a>,\n");
        }
        print!(") {{\n");
        print!("        (\n");
        for field_name in fields {
            print!("        *self.{0},\n", field_name);
        }
        print!("    )\n");
        print!("{}", END);

        // end impl
        print!("{}", END);
    }
}
