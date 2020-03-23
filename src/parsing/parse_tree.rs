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
    CrossJoin(CrossJoin<'a>),
    Join(Join<'a>),
    NaturalJoin(NaturalJoin<'a>),
    OuterJoinKind(OuterJoinKind<'a>),
    OnJoinCriteria(OnJoinCriteria<'a>),
    UsingJoinCriteria(UsingJoinCriteria<'a>),
    GroupBy(GroupBy<'a>),
    Rollup(Rollup<'a>),
    Cube(Cube<'a>),
    GroupingSets(GroupingSets<'a>),
    BinaryExpression(BinaryExpression<'a>),
    UnaryExpression(UnaryExpression<'a>),
    QuanitifiedComparison(QuanitifiedComparison<'a>),
    NullPredicate(NullPredicate<'a>),
    DistinctFrom(DistinctFrom<'a>),
    Between(Between<'a>),
    Like(Like<'a>),
    InSubquery(InSubquery<'a>),
    InList(InList<'a>),
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

#[derive(Clone, Debug)]
pub struct CrossJoin<'a> {
    pub left: Box<ParseTree<'a>>,
    pub cross: Box<ParseTree<'a>>,
    pub join: Box<ParseTree<'a>>,
    pub right: Box<ParseTree<'a>>,
}

pub fn cross_join<'a>(
    left: ParseTree<'a>,
    cross: ParseTree<'a>,
    join: ParseTree<'a>,
    right: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::CrossJoin(CrossJoin {
        left: Box::new(left),
        cross: Box::new(cross),
        join: Box::new(join),
        right: Box::new(right),
    })
}

#[derive(Clone, Debug)]
pub struct Join<'a> {
    pub left: Box<ParseTree<'a>>,
    pub join_type: Box<ParseTree<'a>>,
    pub join: Box<ParseTree<'a>>,
    pub right: Box<ParseTree<'a>>,
    pub join_criteria: Box<ParseTree<'a>>,
}

pub fn join<'a>(
    left: ParseTree<'a>,
    join_type: ParseTree<'a>,
    join: ParseTree<'a>,
    right: ParseTree<'a>,
    join_criteria: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Join(Join {
        left: Box::new(left),
        join_type: Box::new(join_type),
        join: Box::new(join),
        right: Box::new(right),
        join_criteria: Box::new(join_criteria),
    })
}

#[derive(Clone, Debug)]
pub struct NaturalJoin<'a> {
    pub left: Box<ParseTree<'a>>,
    pub natural: Box<ParseTree<'a>>,
    pub join_type: Box<ParseTree<'a>>,
    pub join: Box<ParseTree<'a>>,
    pub right: Box<ParseTree<'a>>,
}

pub fn natural_join<'a>(
    left: ParseTree<'a>,
    natural: ParseTree<'a>,
    join_type: ParseTree<'a>,
    join: ParseTree<'a>,
    right: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::NaturalJoin(NaturalJoin {
        left: Box::new(left),
        natural: Box::new(natural),
        join_type: Box::new(join_type),
        join: Box::new(join),
        right: Box::new(right),
    })
}

#[derive(Clone, Debug)]
pub struct OuterJoinKind<'a> {
    pub kind: Box<ParseTree<'a>>,
    pub outer_opt: Box<ParseTree<'a>>,
}

pub fn outer_join_kind<'a>(kind: ParseTree<'a>, outer_opt: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::OuterJoinKind(OuterJoinKind {
        kind: Box::new(kind),
        outer_opt: Box::new(outer_opt),
    })
}

#[derive(Clone, Debug)]
pub struct OnJoinCriteria<'a> {
    pub on: Box<ParseTree<'a>>,
    pub predicate: Box<ParseTree<'a>>,
}

pub fn on_join_criteria<'a>(on: ParseTree<'a>, predicate: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::OnJoinCriteria(OnJoinCriteria {
        on: Box::new(on),
        predicate: Box::new(predicate),
    })
}

#[derive(Clone, Debug)]
pub struct UsingJoinCriteria<'a> {
    pub using: Box<ParseTree<'a>>,
    pub names: Box<ParseTree<'a>>,
}

pub fn using_join_criteria<'a>(using: ParseTree<'a>, names: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::UsingJoinCriteria(UsingJoinCriteria {
        using: Box::new(using),
        names: Box::new(names),
    })
}

#[derive(Clone, Debug)]
pub struct GroupBy<'a> {
    pub set_quantifier_opt: Box<ParseTree<'a>>,
    pub grouping_elements: Box<ParseTree<'a>>,
}

pub fn group_by<'a>(
    set_quantifier_opt: ParseTree<'a>,
    grouping_elements: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::GroupBy(GroupBy {
        set_quantifier_opt: Box::new(set_quantifier_opt),
        grouping_elements: Box::new(grouping_elements),
    })
}

#[derive(Clone, Debug)]
pub struct Rollup<'a> {
    pub rollup: Box<ParseTree<'a>>,
    pub expressions: Box<ParseTree<'a>>,
}

pub fn rollup<'a>(rollup: ParseTree<'a>, expressions: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Rollup(Rollup {
        rollup: Box::new(rollup),
        expressions: Box::new(expressions),
    })
}

#[derive(Clone, Debug)]
pub struct Cube<'a> {
    pub cube: Box<ParseTree<'a>>,
    pub expressions: Box<ParseTree<'a>>,
}

pub fn cube<'a>(cube: ParseTree<'a>, expressions: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Cube(Cube {
        cube: Box::new(cube),
        expressions: Box::new(expressions),
    })
}

#[derive(Clone, Debug)]
pub struct GroupingSets<'a> {
    pub grouping: Box<ParseTree<'a>>,
    pub sets: Box<ParseTree<'a>>,
    pub grouping_sets: Box<ParseTree<'a>>,
}

pub fn grouping_sets<'a>(
    grouping: ParseTree<'a>,
    sets: ParseTree<'a>,
    grouping_sets: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::GroupingSets(GroupingSets {
        grouping: Box::new(grouping),
        sets: Box::new(sets),
        grouping_sets: Box::new(grouping_sets),
    })
}

#[derive(Clone, Debug)]
pub struct BinaryExpression<'a> {
    pub left: Box<ParseTree<'a>>,
    pub operator: Box<ParseTree<'a>>,
    pub right: Box<ParseTree<'a>>,
}

pub fn binary_expression<'a>(
    left: ParseTree<'a>,
    operator: ParseTree<'a>,
    right: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        operator: Box::new(operator),
        right: Box::new(right),
    })
}

#[derive(Clone, Debug)]
pub struct UnaryExpression<'a> {
    pub operator: Box<ParseTree<'a>>,
    pub operand: Box<ParseTree<'a>>,
}

pub fn unary_expression<'a>(operator: ParseTree<'a>, operand: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::UnaryExpression(UnaryExpression {
        operator: Box::new(operator),
        operand: Box::new(operand),
    })
}

#[derive(Clone, Debug)]
pub struct QuanitifiedComparison<'a> {
    pub operand: Box<ParseTree<'a>>,
    pub operator: Box<ParseTree<'a>>,
    pub comparison_quantifier: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub query: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn quantified_comparison<'a>(
    operand: ParseTree<'a>,
    operator: ParseTree<'a>,
    comparison_quantifier: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    query: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::QuanitifiedComparison(QuanitifiedComparison {
        operand: Box::new(operand),
        operator: Box::new(operator),
        comparison_quantifier: Box::new(comparison_quantifier),
        open_paren: Box::new(open_paren),
        query: Box::new(query),
        close_paren: Box::new(close_paren),
    })
}

#[derive(Clone, Debug)]
pub struct NullPredicate<'a> {
    pub value: Box<ParseTree<'a>>,
    pub is: Box<ParseTree<'a>>,
    pub not_opt: Box<ParseTree<'a>>,
    pub null: Box<ParseTree<'a>>,
}

pub fn null_predicate<'a>(
    value: ParseTree<'a>,
    is: ParseTree<'a>,
    not_opt: ParseTree<'a>,
    null: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::NullPredicate(NullPredicate {
        value: Box::new(value),
        is: Box::new(is),
        not_opt: Box::new(not_opt),
        null: Box::new(null),
    })
}

#[derive(Clone, Debug)]
pub struct DistinctFrom<'a> {
    pub left: Box<ParseTree<'a>>,
    pub distinct: Box<ParseTree<'a>>,
    pub from: Box<ParseTree<'a>>,
    pub right: Box<ParseTree<'a>>,
}

pub fn distinct_from<'a>(
    left: ParseTree<'a>,
    distinct: ParseTree<'a>,
    from: ParseTree<'a>,
    right: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::DistinctFrom(DistinctFrom {
        left: Box::new(left),
        distinct: Box::new(distinct),
        from: Box::new(from),
        right: Box::new(right),
    })
}

#[derive(Clone, Debug)]
pub struct Between<'a> {
    pub value: Box<ParseTree<'a>>,
    pub not_opt: Box<ParseTree<'a>>,
    pub between: Box<ParseTree<'a>>,
    pub lower: Box<ParseTree<'a>>,
    pub and: Box<ParseTree<'a>>,
    pub upper: Box<ParseTree<'a>>,
}

pub fn between<'a>(
    value: ParseTree<'a>,
    not_opt: ParseTree<'a>,
    between: ParseTree<'a>,
    lower: ParseTree<'a>,
    and: ParseTree<'a>,
    upper: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Between(Between {
        value: Box::new(value),
        not_opt: Box::new(not_opt),
        between: Box::new(between),
        lower: Box::new(lower),
        and: Box::new(and),
        upper: Box::new(upper),
    })
}

#[derive(Clone, Debug)]
pub struct Like<'a> {
    pub value: Box<ParseTree<'a>>,
    pub not_opt: Box<ParseTree<'a>>,
    pub like: Box<ParseTree<'a>>,
    pub patrern: Box<ParseTree<'a>>,
    pub escape_opt: Box<ParseTree<'a>>,
    pub escape_value_opt: Box<ParseTree<'a>>,
}

pub fn like<'a>(
    value: ParseTree<'a>,
    not_opt: ParseTree<'a>,
    like: ParseTree<'a>,
    patrern: ParseTree<'a>,
    escape_opt: ParseTree<'a>,
    escape_value_opt: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Like(Like {
        value: Box::new(value),
        not_opt: Box::new(not_opt),
        like: Box::new(like),
        patrern: Box::new(patrern),
        escape_opt: Box::new(escape_opt),
        escape_value_opt: Box::new(escape_value_opt),
    })
}

#[derive(Clone, Debug)]
pub struct InSubquery<'a> {
    pub value: Box<ParseTree<'a>>,
    pub not_opt: Box<ParseTree<'a>>,
    pub in_: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub query: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn in_subquery<'a>(
    value: ParseTree<'a>,
    not_opt: ParseTree<'a>,
    in_: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    query: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::InSubquery(InSubquery {
        value: Box::new(value),
        not_opt: Box::new(not_opt),
        in_: Box::new(in_),
        open_paren: Box::new(open_paren),
        query: Box::new(query),
        close_paren: Box::new(close_paren),
    })
}

#[derive(Clone, Debug)]
pub struct InList<'a> {
    pub value: Box<ParseTree<'a>>,
    pub not_opt: Box<ParseTree<'a>>,
    pub in_: Box<ParseTree<'a>>,
    pub expressions: Box<ParseTree<'a>>,
}

pub fn in_list<'a>(
    value: ParseTree<'a>,
    not_opt: ParseTree<'a>,
    in_: ParseTree<'a>,
    expressions: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::InList(InList {
        value: Box::new(value),
        not_opt: Box::new(not_opt),
        in_: Box::new(in_),
        expressions: Box::new(expressions),
    })
}
