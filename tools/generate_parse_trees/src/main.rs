//! Generates parsing/parse_tree.rs.
//! The ParseTrees contain a lot of boilerplate code, so we auto generate.
//! To add a new tree, add an entry to the configs() function below.

// class name, field names
// class name must be in CamelCase. Field names must be in lower_case.
type TreeConfig = (&'static str, Vec<&'static str>);

//
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
                "null_treatment_opt",
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
            "CreateRole",
            vec![
                "create",
                "role",
                "name",
                "with_admin_grantor_opt",
            ],
        ),
        (
            "WithAdminGrantor",
            vec![
                "with",
                "admin",
                "grantor",
            ],
        ),
        (
            "UserPrincipal",
            vec![
                "user",
                "identifier",
            ],
        ),
        (
            "RolePrincipal",
            vec![
                "role",
                "identifier",
            ],
        ),
        (
            "UnspecifiedPrincipal",
            vec![
                "identifier",
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
        (
            "RelationOrQuery",
            vec!["open_paren", "query_or_relation", "close_paren"],
        ),
        ("EmptyGroupingSet", vec!["open_paren", "close_paren"]),
        (
            "ExpressionOrQuery",
            vec!["open_paren", "expression_or_query", "close_paren"],
        ),
        (
            "Entrypoint",
            vec!["beginning_of_file", "tree", "end_of_file"],
        ),
        ("NullTreatment", vec!["treatment", "nulls"]),
        // ("Class", vec![]),
    ]
}

const FILE_HEADER: &str = r#"use crate::lexing::token;
use crate::utils::{position, syntax_error::SyntaxError, text_range::TextRange};

/// A syntax tree for the Presto SQL language.
///
/// The syntax tree is mostly concrete: only non-significant whitespace
/// is discarded, and that can be reconstituted using the TextRanges
/// of the contained lexemes.
///
/// Every token consumed from the input will be present in the resulting
/// tree.
///
/// There are 4 kinds of parse trees that are structural: Token, List, Empty,
/// and Error.
///
/// Token parse trees consume a single token in the input. Every token consumed
/// from the input will be present in the output parse tree as a Token tree.
///
/// Empty parse trees consume no input, and are a placeholder to indicate that
/// an optional piece of syntax is not present.
///
/// List parse trees represent a possibly delimited, possibly separated list.
/// If the delimiters are not present in the source, then the start_delimiter and
/// end delimiter fields will be Empty parse trees. Similarly, if seperators
/// are not present then they will also be Empty parse trees.
///
/// Error parse trees represent errors during parsing - that a given construct
/// is malformed. Note that errors found during lexing are often attached to
/// (non-error) tokens, so to accumulate all syntax errors both Token and
/// Error trees must be consulted.
///
/// Each syntax production in the grammar is represented by a separate tree
/// kind. Syntax production trees have a set of named children, each child
/// is also a ParseTree. The children of a parse tree must be in the order
/// that they appear. Iterating over the children of a parse tree iterates
/// in source order.
///
/// Rust's enums are somewhat clumsy when representing class hierarchies.
/// ParseTrees include the methods to ease ParseTree usage:
///
///  is_*() - returns true if the tree's kind matches *.
///  as_*() - returns a ref to the typed tree. Must only be called it is_*() is true.
///  unbox_*() -> consumes a parse tree, destructuring it into its unboxed components.
///  children() -> returns a Vec containing refs to all immediate children of the tree.
///
/// This mod also contains a top level factory function for each kind of ParseTree.
///
/// Parse trees are allocated on the heap(in Boxes or Vecs); however the contained
/// tokens have lifetime scoped to the input string which was parsed.
/// Typically consumers will parse, then process parse trees into another format,
/// then release both the parse tree and the input text.
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

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        self.start_delimiter.get_first_token().or_else(||{
            for (element, separator) in &self.elements_and_separators {
                let result = element.get_first_token().or_else(||separator.get_first_token());
                if result.is_some() {
                    return result
                }
            }
            None
        }).or_else(||self.end_delimiter.get_first_token())
    }

    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        self.end_delimiter.get_last_token().or_else(||{
            for (element, separator) in self.elements_and_separators.iter().rev() {
                let result = element.get_last_token().or_else(||separator.get_last_token());
                if result.is_some() {
                    return result
                }
            }
            None
        }).or_else(||self.start_delimiter.get_last_token())
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

    // Note: Has poor performance O(tree depth)
    pub fn get_start(&self) -> position::Position {
        match self {
            ParseTree::Empty(empty) => empty.range.start,
            ParseTree::Error(error) => error.error.get_range().start,
            _ => match self.get_first_token() {
                Some(token) => token.range.start,
                // All children are empty or errors
                None => self.get_first_child().get_start(),
            }
        }
    }

    // Note: Has poor performance O(tree depth)
    pub fn get_full_start(&self) -> position::Position {
        match self {
            ParseTree::Empty(empty) => empty.range.start,
            ParseTree::Error(error) => error.error.get_range().start,
            _ => match self.get_first_token() {
                Some(token) => token.full_start(),
                // All children are empty or errors
                None => self.get_first_child().get_start(),
            }
        }
    }

    // Note: Has poor performance O(tree depth)
    pub fn get_end(&self) -> position::Position {
        match self {
            ParseTree::Empty(empty) => empty.range.end,
            ParseTree::Error(error) => error.error.get_range().end,
            _ => match self.get_last_token() {
                Some(token) => token.range.end,
                // All children are empty or errors
                None => self.get_last_child().get_end(),
            }
        }
    }

    // Note: Has poor performance O(tree depth)
    pub fn get_full_end(&self) -> position::Position {
        match self {
            ParseTree::Empty(empty) => empty.range.end,
            ParseTree::Error(error) => error.error.get_range().end,
            _ => match self.get_last_token() {
                Some(token) => token.full_end(),
                // All children are empty or errors
                None => self.get_last_child().get_full_end(),
            }
        }
    }

    // Note: Has poor performance O(tree depth)
    pub fn get_range(&self) -> TextRange {
        TextRange::new(self.get_start(), self.get_end())
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
            debug_assert!(ch.is_ascii_lowercase());
            ctor_name.push(ch);
        }
    }
    let field_names = &config.1;
    debug_assert!(field_names.len() > 0);

    (config.0, ctor_name, field_names)
}

fn print_switch_body(cs: &Vec<TreeConfig>, method_name: &str) {
    for config in cs {
        let (class_name, ctor_name, _) = get_config(config);
        println!(
            "            ParseTree::{0}({1}) => {1}.{2}(),",
            class_name, ctor_name, method_name
        );
    }
    print!("        }}\n");
    print!("{}", END);
}

fn print_switch_body_rev(cs: &Vec<TreeConfig>, method_name: &str) {
    for config in cs.iter().rev() {
        let (class_name, ctor_name, _) = get_config(config);
        println!(
            "            ParseTree::{0}({1}) => {1}.{2}(),",
            class_name, ctor_name, method_name
        );
    }
    print!("        }}\n");
    print!("{}", END);
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

    // impl ParseTree
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
    print_switch_body(&cs, "children");
    // get_first_child
    print!("    pub fn get_first_child(&self) -> &ParseTree<'a> {{\n");
    print!("        match self {{\n");
    print!("            ParseTree::Token(_) => self,\n");
    print!("            ParseTree::List(list) => &list.start_delimiter,\n");
    print!("            ParseTree::Error(_) => self,\n");
    print!("            ParseTree::Empty(_) => self,\n");
    print_switch_body(&cs, "get_first_child");
    // get_last_child
    print!("    pub fn get_last_child(&self) -> &ParseTree<'a> {{\n");
    print!("        match self {{\n");
    print!("            ParseTree::Token(_) => self,\n");
    print!("            ParseTree::List(list) => &list.end_delimiter,\n");
    print!("            ParseTree::Error(_) => self,\n");
    print!("            ParseTree::Empty(_) => self,\n");
    print_switch_body(&cs, "get_last_child");
    // get_first_token
    print!("    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {{\n");
    print!("        match self {{\n");
    print!("            ParseTree::Token(token) => Some(&token.token),\n");
    print!("            ParseTree::List(list) => list.get_first_token(),\n");
    print!("            ParseTree::Error(_) => None,\n");
    print!("            ParseTree::Empty(_) => None,\n");
    print_switch_body(&cs, "get_first_token");
    // get_last_token
    print!("    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {{\n");
    print!("        match self {{\n");
    print!("            ParseTree::Token(token) => Some(&token.token),\n");
    print!("            ParseTree::List(list) => list.get_last_token(),\n");
    print!("            ParseTree::Error(_) => None,\n");
    print!("            ParseTree::Empty(_) => None,\n");
    print_switch_body_rev(&cs, "get_last_token");
    // end impl
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

        // get_first_child
        println!("    pub fn get_first_child(&self) -> &ParseTree<'a> {{");
        println!("        &self.{}", fields[0]);
        print!("{}", END);

        // get_last_child
        println!("    pub fn get_last_child(&self) -> &ParseTree<'a> {{");
        println!("        &self.{}", fields[fields.len() - 1]);
        print!("{}", END);

        // get_first_token
        println!("    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {{");
        for field_name in fields {
            println!(
                "        if let Some(token) = self.{}.get_first_token() {{",
                field_name
            );
            println!("            return Some(token);");
            println!("        }}");
        }
        println!("        None");
        println!("    }}");

        // get_last_token
        println!("    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {{");
        for field_name in fields.iter().rev() {
            println!(
                "        if let Some(token) = self.{}.get_last_token() {{",
                field_name
            );
            println!("            return Some(token);");
            println!("        }}");
        }
        println!("        None");
        println!("    }}");

        // end impl
        print!("{}", END);
    }
}
