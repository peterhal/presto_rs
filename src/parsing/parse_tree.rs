use crate::lexing::{text_range::TextRange, token};

#[derive(Clone, Debug)]
pub enum ParseTree<'a> {
    // The core trees
    Empty(Empty),
    Token(Token<'a>),
    List(List<'a>),
    Error(Error),

    // The language specific trees
    Query(Query<'a>),
    With(With<'a>),
    NamedQuery(NamedQuery<'a>),
    QueryNoWith(QueryNoWith<'a>),
    OrderBy(OrderBy<'a>),
    Limit(Limit<'a>),
    QuerySetOperation(QuerySetOperation<'a>),
    SortItem(SortItem<'a>),
    Subquery(Subquery<'a>),
    InlineTable(InlineTable<'a>),
    Table(Table<'a>),
    QuerySpecification(QuerySpecification<'a>),
    QualifiedName(QualifiedName<'a>),
    SelectAll(SelectAll<'a>),
    QualifiedSelectAll(QualifiedSelectAll<'a>),
    SelectItem(SelectItem<'a>),
    SubqueryRelation(SubqueryRelation<'a>),
    ParenthesizedRelation(ParenthesizedRelation<'a>),
    TableName(TableName<'a>),
    Lateral(Lateral<'a>),
    Unnest(Unnest<'a>),
    SampledRelation(SampledRelation<'a>),
    AliasedRelation(AliasedRelation<'a>),
}

// The core trees
#[derive(Clone, Debug)]
pub struct Empty {
    pub range: TextRange,
}

pub fn empty<'a>(range: TextRange) -> ParseTree<'a> {
    ParseTree::Empty(Empty { range })
}

#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub token: token::Token<'a>,
}

pub fn token<'a>(token: token::Token<'a>) -> ParseTree<'a> {
    ParseTree::Token(Token { token })
}

#[derive(Clone, Debug)]
pub struct List<'a> {
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

#[derive(Clone, Debug)]
pub struct Error {
    pub range: TextRange,
    pub message: String,
}

pub fn error<'a>(range: TextRange, message: String) -> ParseTree<'a> {
    ParseTree::Error(Error {
        range,
        message: message,
    })
}

// core impl
impl<'a> ParseTree<'a> {
    pub fn is_empty(&self) -> bool {
        if let ParseTree::Empty(_) = self {
            true
        } else {
            false
        }
    }
}

// The language specific trees
#[derive(Clone, Debug)]
pub struct Query<'a> {
    pub with: Box<ParseTree<'a>>,
    pub query_no_with: Box<ParseTree<'a>>,
}

pub fn query<'a>(with: ParseTree<'a>, query_no_with: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Query(Query {
        with: Box::new(with),
        query_no_with: Box::new(query_no_with),
    })
}

#[derive(Clone, Debug)]
pub struct With<'a> {
    pub with: Box<ParseTree<'a>>,
    pub recursive: Box<ParseTree<'a>>,
    pub named_queries: Box<ParseTree<'a>>,
}

pub fn with<'a>(
    with: ParseTree<'a>,
    recursive: ParseTree<'a>,
    named_queries: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::With(With {
        with: Box::new(with),
        recursive: Box::new(recursive),
        named_queries: Box::new(named_queries),
    })
}

#[derive(Clone, Debug)]
pub struct NamedQuery<'a> {
    pub name: Box<ParseTree<'a>>,
    pub column_aliases_opt: Box<ParseTree<'a>>,
    pub as_: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub query: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn named_query<'a>(
    name: ParseTree<'a>,
    column_aliases_opt: ParseTree<'a>,
    as_: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    query: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::NamedQuery(NamedQuery {
        name: Box::new(name),
        column_aliases_opt: Box::new(column_aliases_opt),
        as_: Box::new(as_),
        open_paren: Box::new(open_paren),
        query: Box::new(query),
        close_paren: Box::new(close_paren),
    })
}

#[derive(Clone, Debug)]
pub struct QueryNoWith<'a> {
    pub query_term: Box<ParseTree<'a>>,
    pub order_by_opt: Box<ParseTree<'a>>,
    pub limit_opt: Box<ParseTree<'a>>,
}

pub fn query_no_with<'a>(
    query_term: ParseTree<'a>,
    order_by_opt: ParseTree<'a>,
    limit_opt: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::QueryNoWith(QueryNoWith {
        query_term: Box::new(query_term),
        order_by_opt: Box::new(order_by_opt),
        limit_opt: Box::new(limit_opt),
    })
}

#[derive(Clone, Debug)]
pub struct OrderBy<'a> {
    pub order: Box<ParseTree<'a>>,
    pub by: Box<ParseTree<'a>>,
    pub sort_items: Box<ParseTree<'a>>,
}

pub fn order_by<'a>(
    order: ParseTree<'a>,
    by: ParseTree<'a>,
    sort_items: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::OrderBy(OrderBy {
        order: Box::new(order),
        by: Box::new(by),
        sort_items: Box::new(sort_items),
    })
}

#[derive(Clone, Debug)]
pub struct Limit<'a> {
    pub limit: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
}

pub fn limit<'a>(limit: ParseTree<'a>, value: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Limit(Limit {
        limit: Box::new(limit),
        value: Box::new(value),
    })
}

#[derive(Clone, Debug)]
pub struct QuerySetOperation<'a> {
    pub left: Box<ParseTree<'a>>,
    pub operator: Box<ParseTree<'a>>,
    pub set_quantifier_opt: Box<ParseTree<'a>>,
    pub right: Box<ParseTree<'a>>,
}

pub fn query_set_operation<'a>(
    left: ParseTree<'a>,
    operator: ParseTree<'a>,
    set_quantifier_opt: ParseTree<'a>,
    right: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::QuerySetOperation(QuerySetOperation {
        left: Box::new(left),
        operator: Box::new(operator),
        set_quantifier_opt: Box::new(set_quantifier_opt),
        right: Box::new(right),
    })
}

#[derive(Clone, Debug)]
pub struct SortItem<'a> {
    pub expression: Box<ParseTree<'a>>,
    pub ordering_opt: Box<ParseTree<'a>>,
    pub nulls: Box<ParseTree<'a>>,
    pub null_ordering_opt: Box<ParseTree<'a>>,
}

pub fn sort_item<'a>(
    expression: ParseTree<'a>,
    ordering_opt: ParseTree<'a>,
    nulls: ParseTree<'a>,
    null_ordering_opt: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::SortItem(SortItem {
        expression: Box::new(expression),
        ordering_opt: Box::new(ordering_opt),
        nulls: Box::new(nulls),
        null_ordering_opt: Box::new(null_ordering_opt),
    })
}

#[derive(Clone, Debug)]
pub struct Subquery<'a> {
    pub open_paren: Box<ParseTree<'a>>,
    pub query_no_with: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn subquery<'a>(
    open_paren: ParseTree<'a>,
    query_no_with: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Subquery(Subquery {
        open_paren: Box::new(open_paren),
        query_no_with: Box::new(query_no_with),
        close_paren: Box::new(close_paren),
    })
}

#[derive(Clone, Debug)]
pub struct InlineTable<'a> {
    pub values: Box<ParseTree<'a>>,
    pub expressions: Box<ParseTree<'a>>,
}

pub fn inline_table<'a>(values: ParseTree<'a>, expressions: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::InlineTable(InlineTable {
        values: Box::new(values),
        expressions: Box::new(expressions),
    })
}

#[derive(Clone, Debug)]
pub struct Table<'a> {
    pub table: Box<ParseTree<'a>>,
    pub qualified_name: Box<ParseTree<'a>>,
}

pub fn table<'a>(table: ParseTree<'a>, qualified_name: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Table(Table {
        table: Box::new(table),
        qualified_name: Box::new(qualified_name),
    })
}

#[derive(Clone, Debug)]
pub struct QuerySpecification<'a> {
    pub select: Box<ParseTree<'a>>,
    pub set_quantifier_opt: Box<ParseTree<'a>>,
    pub select_items: Box<ParseTree<'a>>,
    pub from: Box<ParseTree<'a>>,
    pub relations: Box<ParseTree<'a>>,
    pub where_: Box<ParseTree<'a>>,
    pub where_predicate: Box<ParseTree<'a>>,
    pub group: Box<ParseTree<'a>>,
    pub by: Box<ParseTree<'a>>,
    pub group_by: Box<ParseTree<'a>>,
    pub having: Box<ParseTree<'a>>,
    pub having_predicate: Box<ParseTree<'a>>,
}

pub fn query_specification<'a>(
    select: ParseTree<'a>,
    set_quantifier_opt: ParseTree<'a>,
    select_items: ParseTree<'a>,
    from: ParseTree<'a>,
    relations: ParseTree<'a>,
    where_: ParseTree<'a>,
    where_predicate: ParseTree<'a>,
    group: ParseTree<'a>,
    by: ParseTree<'a>,
    group_by: ParseTree<'a>,
    having: ParseTree<'a>,
    having_predicate: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::QuerySpecification(QuerySpecification {
        select: Box::new(select),
        set_quantifier_opt: Box::new(set_quantifier_opt),
        select_items: Box::new(select_items),
        from: Box::new(from),
        relations: Box::new(relations),
        where_: Box::new(where_),
        where_predicate: Box::new(where_predicate),
        group: Box::new(group),
        by: Box::new(by),
        group_by: Box::new(group_by),
        having: Box::new(having),
        having_predicate: Box::new(having_predicate),
    })
}

#[derive(Clone, Debug)]
pub struct QualifiedName<'a> {
    pub names: Box<ParseTree<'a>>,
}

pub fn qualified_name<'a>(names: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::QualifiedName(QualifiedName {
        names: Box::new(names),
    })
}

#[derive(Clone, Debug)]
pub struct SelectAll<'a> {
    pub asterisk: Box<ParseTree<'a>>,
}

pub fn select_all<'a>(asterisk: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::SelectAll(SelectAll {
        asterisk: Box::new(asterisk),
    })
}

#[derive(Clone, Debug)]
pub struct QualifiedSelectAll<'a> {
    pub qualifier: Box<ParseTree<'a>>,
    pub period: Box<ParseTree<'a>>,
    pub asterisk: Box<ParseTree<'a>>,
}

pub fn qualified_select_all<'a>(
    qualifier: ParseTree<'a>,
    period: ParseTree<'a>,
    asterisk: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::QualifiedSelectAll(QualifiedSelectAll {
        qualifier: Box::new(qualifier),
        period: Box::new(period),
        asterisk: Box::new(asterisk),
    })
}

#[derive(Clone, Debug)]
pub struct SelectItem<'a> {
    pub expression: Box<ParseTree<'a>>,
    pub as_: Box<ParseTree<'a>>,
    pub identifier: Box<ParseTree<'a>>,
}

pub fn select_item<'a>(
    expression: ParseTree<'a>,
    as_: ParseTree<'a>,
    identifier: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::SelectItem(SelectItem {
        expression: Box::new(expression),
        as_: Box::new(as_),
        identifier: Box::new(identifier),
    })
}

#[derive(Clone, Debug)]
pub struct SubqueryRelation<'a> {
    pub open_paren: Box<ParseTree<'a>>,
    pub query: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn subquery_relation<'a>(
    open_paren: ParseTree<'a>,
    query: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::SubqueryRelation(SubqueryRelation {
        open_paren: Box::new(open_paren),
        query: Box::new(query),
        close_paren: Box::new(close_paren),
    })
}

#[derive(Clone, Debug)]
pub struct ParenthesizedRelation<'a> {
    pub open_paren: Box<ParseTree<'a>>,
    pub relation: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn parenthesized_relation<'a>(
    open_paren: ParseTree<'a>,
    relation: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::ParenthesizedRelation(ParenthesizedRelation {
        open_paren: Box::new(open_paren),
        relation: Box::new(relation),
        close_paren: Box::new(close_paren),
    })
}

#[derive(Clone, Debug)]
pub struct TableName<'a> {
    pub name: Box<ParseTree<'a>>,
}

pub fn table_name<'a>(name: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::TableName(TableName {
        name: Box::new(name),
    })
}

#[derive(Clone, Debug)]
pub struct Lateral<'a> {
    pub lateral: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub query: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn lateral<'a>(
    lateral: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    query: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Lateral(Lateral {
        lateral: Box::new(lateral),
        open_paren: Box::new(open_paren),
        query: Box::new(query),
        close_paren: Box::new(close_paren),
    })
}

#[derive(Clone, Debug)]
pub struct Unnest<'a> {
    pub unnest: Box<ParseTree<'a>>,
    pub expressions: Box<ParseTree<'a>>,
    pub with: Box<ParseTree<'a>>,
    pub ordinality: Box<ParseTree<'a>>,
}

pub fn unnest<'a>(
    unnest: ParseTree<'a>,
    expressions: ParseTree<'a>,
    with: ParseTree<'a>,
    ordinality: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Unnest(Unnest {
        unnest: Box::new(unnest),
        expressions: Box::new(expressions),
        with: Box::new(with),
        ordinality: Box::new(ordinality),
    })
}

#[derive(Clone, Debug)]
pub struct SampledRelation<'a> {
    pub aliased_relation: Box<ParseTree<'a>>,
    pub tablesample: Box<ParseTree<'a>>,
    pub sample_type: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub expression: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn sampled_relation<'a>(
    aliased_relation: ParseTree<'a>,
    tablesample: ParseTree<'a>,
    sample_type: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    expression: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::SampledRelation(SampledRelation {
        aliased_relation: Box::new(aliased_relation),
        tablesample: Box::new(tablesample),
        sample_type: Box::new(sample_type),
        open_paren: Box::new(open_paren),
        expression: Box::new(expression),
        close_paren: Box::new(close_paren),
    })
}

#[derive(Clone, Debug)]
pub struct AliasedRelation<'a> {
    pub relation_primary: Box<ParseTree<'a>>,
    pub as_opt: Box<ParseTree<'a>>,
    pub identifier: Box<ParseTree<'a>>,
    pub column_aliases_opt: Box<ParseTree<'a>>,
}

pub fn aliased_relation<'a>(
    relation_primary: ParseTree<'a>,
    as_opt: ParseTree<'a>,
    identifier: ParseTree<'a>,
    column_aliases_opt: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::AliasedRelation(AliasedRelation {
        relation_primary: Box::new(relation_primary),
        as_opt: Box::new(as_opt),
        identifier: Box::new(identifier),
        column_aliases_opt: Box::new(column_aliases_opt),
    })
}
