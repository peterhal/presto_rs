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
    QuantifiedComparison(QuantifiedComparison<'a>),
    NullPredicate(NullPredicate<'a>),
    DistinctFrom(DistinctFrom<'a>),
    Between(Between<'a>),
    Like(Like<'a>),
    InSubquery(InSubquery<'a>),
    InList(InList<'a>),
    AtTimeZone(AtTimeZone<'a>),
    Dereference(Dereference<'a>),
    Subscript(Subscript<'a>),
    Lambda(Lambda<'a>),
    Literal(Literal<'a>),
    RowConstructor(RowConstructor<'a>),
    ParenthesizedExpression(ParenthesizedExpression<'a>),
    Identifier(Identifier<'a>),
    FunctionCall(FunctionCall<'a>),
    Filter(Filter<'a>),
    Over(Over<'a>),
    WindowFrame(WindowFrame<'a>),
    UnboundedFrame(UnboundedFrame<'a>),
    CurrentRowBound(CurrentRowBound<'a>),
    BoundedFrame(BoundedFrame<'a>),
    UnicodeString(UnicodeString<'a>),
    ConfigureExpression(ConfigureExpression<'a>),
    SubqueryExpression(SubqueryExpression<'a>),
    Grouping(Grouping<'a>),
    Extract(Extract<'a>),
    CurrentTime(CurrentTime<'a>),
    CurrentTimestamp(CurrentTimestamp<'a>),
    Normalize(Normalize<'a>),
    Localtime(Localtime<'a>),
    Localtimestamp(Localtimestamp<'a>),
    Cast(Cast<'a>),
    WhenClause(WhenClause<'a>),
    Case(Case<'a>),
    Exists(Exists<'a>),
    TypeConstructor(TypeConstructor<'a>),
    Array(Array<'a>),
    Interval(Interval<'a>),
    Row(Row<'a>),
    TryCast(TryCast<'a>),
    Substring(Substring<'a>),
    Position(Position<'a>),
    ArrayTypeSuffix(ArrayTypeSuffix<'a>),
    NamedType(NamedType<'a>),
    ArrayType(ArrayType<'a>),
    MapType(MapType<'a>),
    RowType(RowType<'a>),
    RowTypeElement(RowTypeElement<'a>),
    IntervalType(IntervalType<'a>),
    IfNotExists(IfNotExists<'a>),
    CreateTable(CreateTable<'a>),
    CreateTableAsSelect(CreateTableAsSelect<'a>),
    WithProperties(WithProperties<'a>),
    Property(Property<'a>),
    WithData(WithData<'a>),
    Comment(Comment<'a>),
    ColumnDefinition(ColumnDefinition<'a>),
    NotNull(NotNull<'a>),
    LikeClause(LikeClause<'a>),
    InsertInto(InsertInto<'a>),
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

impl<'a> List<'a> {
    pub fn len(&self) -> usize {
        self.elements_and_separators.len()
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        Vec<(ParseTree<'a>, ParseTree<'a>)>,
        ParseTree<'a>,
    ) {
        (
            *self.start_delimiter,
            self.elements_and_separators,
            *self.end_delimiter,
        )
    }
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
    pub fn unbox_list(
        self,
    ) -> (
        ParseTree<'a>,
        Vec<(ParseTree<'a>, ParseTree<'a>)>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::List(value) => value.unbox(),
            _ => panic!("Expected List"),
        }
    }

    pub fn is_list(&self) -> bool {
        if let ParseTree::List(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_list(&self) -> &List {
        if let ParseTree::List(value) = self {
            value
        } else {
            panic!("Expected List")
        }
    }

    pub fn is_empty(&self) -> bool {
        if let ParseTree::Empty(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_empty(&self) -> &Empty {
        if let ParseTree::Empty(value) = self {
            value
        } else {
            panic!("Expected Empty")
        }
    }

    pub fn is_token(&self) -> bool {
        if let ParseTree::Token(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_token(&self) -> &Token {
        if let ParseTree::Token(value) = self {
            value
        } else {
            panic!("Expected Token")
        }
    }

    pub fn is_error(&self) -> bool {
        if let ParseTree::Error(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_error(&self) -> &Error {
        if let ParseTree::Error(value) = self {
            value
        } else {
            panic!("Expected Error")
        }
    }

    pub fn is_query(&self) -> bool {
        if let ParseTree::Query(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_query(&self) -> &Query {
        if let ParseTree::Query(value) = self {
            value
        } else {
            panic!("Expected Query")
        }
    }

    pub fn unbox_query(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Query(tree) => tree.unbox(),
            _ => panic!("Expected Query"),
        }
    }

    pub fn is_with(&self) -> bool {
        if let ParseTree::With(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_with(&self) -> &With {
        if let ParseTree::With(value) = self {
            value
        } else {
            panic!("Expected With")
        }
    }

    pub fn unbox_with(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::With(tree) => tree.unbox(),
            _ => panic!("Expected With"),
        }
    }

    pub fn is_named_query(&self) -> bool {
        if let ParseTree::NamedQuery(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_named_query(&self) -> &NamedQuery {
        if let ParseTree::NamedQuery(value) = self {
            value
        } else {
            panic!("Expected NamedQuery")
        }
    }

    pub fn unbox_named_query(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::NamedQuery(tree) => tree.unbox(),
            _ => panic!("Expected NamedQuery"),
        }
    }

    pub fn is_query_no_with(&self) -> bool {
        if let ParseTree::QueryNoWith(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_query_no_with(&self) -> &QueryNoWith {
        if let ParseTree::QueryNoWith(value) = self {
            value
        } else {
            panic!("Expected QueryNoWith")
        }
    }

    pub fn unbox_query_no_with(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::QueryNoWith(tree) => tree.unbox(),
            _ => panic!("Expected QueryNoWith"),
        }
    }

    pub fn is_order_by(&self) -> bool {
        if let ParseTree::OrderBy(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_order_by(&self) -> &OrderBy {
        if let ParseTree::OrderBy(value) = self {
            value
        } else {
            panic!("Expected OrderBy")
        }
    }

    pub fn unbox_order_by(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::OrderBy(tree) => tree.unbox(),
            _ => panic!("Expected OrderBy"),
        }
    }

    pub fn is_limit(&self) -> bool {
        if let ParseTree::Limit(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_limit(&self) -> &Limit {
        if let ParseTree::Limit(value) = self {
            value
        } else {
            panic!("Expected Limit")
        }
    }

    pub fn unbox_limit(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Limit(tree) => tree.unbox(),
            _ => panic!("Expected Limit"),
        }
    }

    pub fn is_query_set_operation(&self) -> bool {
        if let ParseTree::QuerySetOperation(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_query_set_operation(&self) -> &QuerySetOperation {
        if let ParseTree::QuerySetOperation(value) = self {
            value
        } else {
            panic!("Expected QuerySetOperation")
        }
    }

    pub fn unbox_query_set_operation(
        self,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::QuerySetOperation(tree) => tree.unbox(),
            _ => panic!("Expected QuerySetOperation"),
        }
    }

    pub fn is_sort_item(&self) -> bool {
        if let ParseTree::SortItem(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_sort_item(&self) -> &SortItem {
        if let ParseTree::SortItem(value) = self {
            value
        } else {
            panic!("Expected SortItem")
        }
    }

    pub fn unbox_sort_item(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::SortItem(tree) => tree.unbox(),
            _ => panic!("Expected SortItem"),
        }
    }

    pub fn is_subquery(&self) -> bool {
        if let ParseTree::Subquery(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_subquery(&self) -> &Subquery {
        if let ParseTree::Subquery(value) = self {
            value
        } else {
            panic!("Expected Subquery")
        }
    }

    pub fn unbox_subquery(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Subquery(tree) => tree.unbox(),
            _ => panic!("Expected Subquery"),
        }
    }

    pub fn is_inline_table(&self) -> bool {
        if let ParseTree::InlineTable(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_inline_table(&self) -> &InlineTable {
        if let ParseTree::InlineTable(value) = self {
            value
        } else {
            panic!("Expected InlineTable")
        }
    }

    pub fn unbox_inline_table(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::InlineTable(tree) => tree.unbox(),
            _ => panic!("Expected InlineTable"),
        }
    }

    pub fn is_table(&self) -> bool {
        if let ParseTree::Table(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_table(&self) -> &Table {
        if let ParseTree::Table(value) = self {
            value
        } else {
            panic!("Expected Table")
        }
    }

    pub fn unbox_table(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Table(tree) => tree.unbox(),
            _ => panic!("Expected Table"),
        }
    }

    pub fn is_query_specification(&self) -> bool {
        if let ParseTree::QuerySpecification(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_query_specification(&self) -> &QuerySpecification {
        if let ParseTree::QuerySpecification(value) = self {
            value
        } else {
            panic!("Expected QuerySpecification")
        }
    }

    pub fn unbox_query_specification(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::QuerySpecification(tree) => tree.unbox(),
            _ => panic!("Expected QuerySpecification"),
        }
    }

    pub fn is_qualified_name(&self) -> bool {
        if let ParseTree::QualifiedName(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_qualified_name(&self) -> &QualifiedName {
        if let ParseTree::QualifiedName(value) = self {
            value
        } else {
            panic!("Expected QualifiedName")
        }
    }

    pub fn unbox_qualified_name(self) -> (ParseTree<'a>,) {
        match self {
            ParseTree::QualifiedName(tree) => tree.unbox(),
            _ => panic!("Expected QualifiedName"),
        }
    }

    pub fn is_select_all(&self) -> bool {
        if let ParseTree::SelectAll(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_select_all(&self) -> &SelectAll {
        if let ParseTree::SelectAll(value) = self {
            value
        } else {
            panic!("Expected SelectAll")
        }
    }

    pub fn unbox_select_all(self) -> (ParseTree<'a>,) {
        match self {
            ParseTree::SelectAll(tree) => tree.unbox(),
            _ => panic!("Expected SelectAll"),
        }
    }

    pub fn is_qualified_select_all(&self) -> bool {
        if let ParseTree::QualifiedSelectAll(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_qualified_select_all(&self) -> &QualifiedSelectAll {
        if let ParseTree::QualifiedSelectAll(value) = self {
            value
        } else {
            panic!("Expected QualifiedSelectAll")
        }
    }

    pub fn unbox_qualified_select_all(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::QualifiedSelectAll(tree) => tree.unbox(),
            _ => panic!("Expected QualifiedSelectAll"),
        }
    }

    pub fn is_select_item(&self) -> bool {
        if let ParseTree::SelectItem(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_select_item(&self) -> &SelectItem {
        if let ParseTree::SelectItem(value) = self {
            value
        } else {
            panic!("Expected SelectItem")
        }
    }

    pub fn unbox_select_item(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::SelectItem(tree) => tree.unbox(),
            _ => panic!("Expected SelectItem"),
        }
    }

    pub fn is_subquery_relation(&self) -> bool {
        if let ParseTree::SubqueryRelation(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_subquery_relation(&self) -> &SubqueryRelation {
        if let ParseTree::SubqueryRelation(value) = self {
            value
        } else {
            panic!("Expected SubqueryRelation")
        }
    }

    pub fn unbox_subquery_relation(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::SubqueryRelation(tree) => tree.unbox(),
            _ => panic!("Expected SubqueryRelation"),
        }
    }

    pub fn is_parenthesized_relation(&self) -> bool {
        if let ParseTree::ParenthesizedRelation(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_parenthesized_relation(&self) -> &ParenthesizedRelation {
        if let ParseTree::ParenthesizedRelation(value) = self {
            value
        } else {
            panic!("Expected ParenthesizedRelation")
        }
    }

    pub fn unbox_parenthesized_relation(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::ParenthesizedRelation(tree) => tree.unbox(),
            _ => panic!("Expected ParenthesizedRelation"),
        }
    }

    pub fn is_table_name(&self) -> bool {
        if let ParseTree::TableName(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_table_name(&self) -> &TableName {
        if let ParseTree::TableName(value) = self {
            value
        } else {
            panic!("Expected TableName")
        }
    }

    pub fn unbox_table_name(self) -> (ParseTree<'a>,) {
        match self {
            ParseTree::TableName(tree) => tree.unbox(),
            _ => panic!("Expected TableName"),
        }
    }

    pub fn is_lateral(&self) -> bool {
        if let ParseTree::Lateral(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_lateral(&self) -> &Lateral {
        if let ParseTree::Lateral(value) = self {
            value
        } else {
            panic!("Expected Lateral")
        }
    }

    pub fn unbox_lateral(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Lateral(tree) => tree.unbox(),
            _ => panic!("Expected Lateral"),
        }
    }

    pub fn is_unnest(&self) -> bool {
        if let ParseTree::Unnest(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_unnest(&self) -> &Unnest {
        if let ParseTree::Unnest(value) = self {
            value
        } else {
            panic!("Expected Unnest")
        }
    }

    pub fn unbox_unnest(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Unnest(tree) => tree.unbox(),
            _ => panic!("Expected Unnest"),
        }
    }

    pub fn is_sampled_relation(&self) -> bool {
        if let ParseTree::SampledRelation(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_sampled_relation(&self) -> &SampledRelation {
        if let ParseTree::SampledRelation(value) = self {
            value
        } else {
            panic!("Expected SampledRelation")
        }
    }

    pub fn unbox_sampled_relation(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::SampledRelation(tree) => tree.unbox(),
            _ => panic!("Expected SampledRelation"),
        }
    }

    pub fn is_aliased_relation(&self) -> bool {
        if let ParseTree::AliasedRelation(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_aliased_relation(&self) -> &AliasedRelation {
        if let ParseTree::AliasedRelation(value) = self {
            value
        } else {
            panic!("Expected AliasedRelation")
        }
    }

    pub fn unbox_aliased_relation(
        self,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::AliasedRelation(tree) => tree.unbox(),
            _ => panic!("Expected AliasedRelation"),
        }
    }

    pub fn is_cross_join(&self) -> bool {
        if let ParseTree::CrossJoin(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_cross_join(&self) -> &CrossJoin {
        if let ParseTree::CrossJoin(value) = self {
            value
        } else {
            panic!("Expected CrossJoin")
        }
    }

    pub fn unbox_cross_join(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::CrossJoin(tree) => tree.unbox(),
            _ => panic!("Expected CrossJoin"),
        }
    }

    pub fn is_join(&self) -> bool {
        if let ParseTree::Join(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_join(&self) -> &Join {
        if let ParseTree::Join(value) = self {
            value
        } else {
            panic!("Expected Join")
        }
    }

    pub fn unbox_join(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Join(tree) => tree.unbox(),
            _ => panic!("Expected Join"),
        }
    }

    pub fn is_natural_join(&self) -> bool {
        if let ParseTree::NaturalJoin(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_natural_join(&self) -> &NaturalJoin {
        if let ParseTree::NaturalJoin(value) = self {
            value
        } else {
            panic!("Expected NaturalJoin")
        }
    }

    pub fn unbox_natural_join(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::NaturalJoin(tree) => tree.unbox(),
            _ => panic!("Expected NaturalJoin"),
        }
    }

    pub fn is_outer_join_kind(&self) -> bool {
        if let ParseTree::OuterJoinKind(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_outer_join_kind(&self) -> &OuterJoinKind {
        if let ParseTree::OuterJoinKind(value) = self {
            value
        } else {
            panic!("Expected OuterJoinKind")
        }
    }

    pub fn unbox_outer_join_kind(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::OuterJoinKind(tree) => tree.unbox(),
            _ => panic!("Expected OuterJoinKind"),
        }
    }

    pub fn is_on_join_criteria(&self) -> bool {
        if let ParseTree::OnJoinCriteria(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_on_join_criteria(&self) -> &OnJoinCriteria {
        if let ParseTree::OnJoinCriteria(value) = self {
            value
        } else {
            panic!("Expected OnJoinCriteria")
        }
    }

    pub fn unbox_on_join_criteria(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::OnJoinCriteria(tree) => tree.unbox(),
            _ => panic!("Expected OnJoinCriteria"),
        }
    }

    pub fn is_using_join_criteria(&self) -> bool {
        if let ParseTree::UsingJoinCriteria(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_using_join_criteria(&self) -> &UsingJoinCriteria {
        if let ParseTree::UsingJoinCriteria(value) = self {
            value
        } else {
            panic!("Expected UsingJoinCriteria")
        }
    }

    pub fn unbox_using_join_criteria(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::UsingJoinCriteria(tree) => tree.unbox(),
            _ => panic!("Expected UsingJoinCriteria"),
        }
    }

    pub fn is_group_by(&self) -> bool {
        if let ParseTree::GroupBy(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_group_by(&self) -> &GroupBy {
        if let ParseTree::GroupBy(value) = self {
            value
        } else {
            panic!("Expected GroupBy")
        }
    }

    pub fn unbox_group_by(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::GroupBy(tree) => tree.unbox(),
            _ => panic!("Expected GroupBy"),
        }
    }

    pub fn is_rollup(&self) -> bool {
        if let ParseTree::Rollup(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_rollup(&self) -> &Rollup {
        if let ParseTree::Rollup(value) = self {
            value
        } else {
            panic!("Expected Rollup")
        }
    }

    pub fn unbox_rollup(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Rollup(tree) => tree.unbox(),
            _ => panic!("Expected Rollup"),
        }
    }

    pub fn is_cube(&self) -> bool {
        if let ParseTree::Cube(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_cube(&self) -> &Cube {
        if let ParseTree::Cube(value) = self {
            value
        } else {
            panic!("Expected Cube")
        }
    }

    pub fn unbox_cube(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Cube(tree) => tree.unbox(),
            _ => panic!("Expected Cube"),
        }
    }

    pub fn is_grouping_sets(&self) -> bool {
        if let ParseTree::GroupingSets(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_grouping_sets(&self) -> &GroupingSets {
        if let ParseTree::GroupingSets(value) = self {
            value
        } else {
            panic!("Expected GroupingSets")
        }
    }

    pub fn unbox_grouping_sets(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::GroupingSets(tree) => tree.unbox(),
            _ => panic!("Expected GroupingSets"),
        }
    }

    pub fn is_binary_expression(&self) -> bool {
        if let ParseTree::BinaryExpression(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_binary_expression(&self) -> &BinaryExpression {
        if let ParseTree::BinaryExpression(value) = self {
            value
        } else {
            panic!("Expected BinaryExpression")
        }
    }

    pub fn unbox_binary_expression(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::BinaryExpression(tree) => tree.unbox(),
            _ => panic!("Expected BinaryExpression"),
        }
    }

    pub fn is_unary_expression(&self) -> bool {
        if let ParseTree::UnaryExpression(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_unary_expression(&self) -> &UnaryExpression {
        if let ParseTree::UnaryExpression(value) = self {
            value
        } else {
            panic!("Expected UnaryExpression")
        }
    }

    pub fn unbox_unary_expression(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::UnaryExpression(tree) => tree.unbox(),
            _ => panic!("Expected UnaryExpression"),
        }
    }

    pub fn is_quantified_comparison(&self) -> bool {
        if let ParseTree::QuantifiedComparison(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_quantified_comparison(&self) -> &QuantifiedComparison {
        if let ParseTree::QuantifiedComparison(value) = self {
            value
        } else {
            panic!("Expected QuantifiedComparison")
        }
    }

    pub fn unbox_quantified_comparison(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::QuantifiedComparison(tree) => tree.unbox(),
            _ => panic!("Expected QuantifiedComparison"),
        }
    }

    pub fn is_null_predicate(&self) -> bool {
        if let ParseTree::NullPredicate(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_null_predicate(&self) -> &NullPredicate {
        if let ParseTree::NullPredicate(value) = self {
            value
        } else {
            panic!("Expected NullPredicate")
        }
    }

    pub fn unbox_null_predicate(
        self,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::NullPredicate(tree) => tree.unbox(),
            _ => panic!("Expected NullPredicate"),
        }
    }

    pub fn is_distinct_from(&self) -> bool {
        if let ParseTree::DistinctFrom(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_distinct_from(&self) -> &DistinctFrom {
        if let ParseTree::DistinctFrom(value) = self {
            value
        } else {
            panic!("Expected DistinctFrom")
        }
    }

    pub fn unbox_distinct_from(
        self,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::DistinctFrom(tree) => tree.unbox(),
            _ => panic!("Expected DistinctFrom"),
        }
    }

    pub fn is_between(&self) -> bool {
        if let ParseTree::Between(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_between(&self) -> &Between {
        if let ParseTree::Between(value) = self {
            value
        } else {
            panic!("Expected Between")
        }
    }

    pub fn unbox_between(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Between(tree) => tree.unbox(),
            _ => panic!("Expected Between"),
        }
    }

    pub fn is_like(&self) -> bool {
        if let ParseTree::Like(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_like(&self) -> &Like {
        if let ParseTree::Like(value) = self {
            value
        } else {
            panic!("Expected Like")
        }
    }

    pub fn unbox_like(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Like(tree) => tree.unbox(),
            _ => panic!("Expected Like"),
        }
    }

    pub fn is_in_subquery(&self) -> bool {
        if let ParseTree::InSubquery(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_in_subquery(&self) -> &InSubquery {
        if let ParseTree::InSubquery(value) = self {
            value
        } else {
            panic!("Expected InSubquery")
        }
    }

    pub fn unbox_in_subquery(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::InSubquery(tree) => tree.unbox(),
            _ => panic!("Expected InSubquery"),
        }
    }

    pub fn is_in_list(&self) -> bool {
        if let ParseTree::InList(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_in_list(&self) -> &InList {
        if let ParseTree::InList(value) = self {
            value
        } else {
            panic!("Expected InList")
        }
    }

    pub fn unbox_in_list(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::InList(tree) => tree.unbox(),
            _ => panic!("Expected InList"),
        }
    }

    pub fn is_at_time_zone(&self) -> bool {
        if let ParseTree::AtTimeZone(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_at_time_zone(&self) -> &AtTimeZone {
        if let ParseTree::AtTimeZone(value) = self {
            value
        } else {
            panic!("Expected AtTimeZone")
        }
    }

    pub fn unbox_at_time_zone(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::AtTimeZone(tree) => tree.unbox(),
            _ => panic!("Expected AtTimeZone"),
        }
    }

    pub fn is_dereference(&self) -> bool {
        if let ParseTree::Dereference(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_dereference(&self) -> &Dereference {
        if let ParseTree::Dereference(value) = self {
            value
        } else {
            panic!("Expected Dereference")
        }
    }

    pub fn unbox_dereference(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Dereference(tree) => tree.unbox(),
            _ => panic!("Expected Dereference"),
        }
    }

    pub fn is_subscript(&self) -> bool {
        if let ParseTree::Subscript(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_subscript(&self) -> &Subscript {
        if let ParseTree::Subscript(value) = self {
            value
        } else {
            panic!("Expected Subscript")
        }
    }

    pub fn unbox_subscript(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Subscript(tree) => tree.unbox(),
            _ => panic!("Expected Subscript"),
        }
    }

    pub fn is_lambda(&self) -> bool {
        if let ParseTree::Lambda(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_lambda(&self) -> &Lambda {
        if let ParseTree::Lambda(value) = self {
            value
        } else {
            panic!("Expected Lambda")
        }
    }

    pub fn unbox_lambda(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Lambda(tree) => tree.unbox(),
            _ => panic!("Expected Lambda"),
        }
    }

    pub fn is_literal(&self) -> bool {
        if let ParseTree::Literal(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_literal(&self) -> &Literal {
        if let ParseTree::Literal(value) = self {
            value
        } else {
            panic!("Expected Literal")
        }
    }

    pub fn unbox_literal(self) -> (ParseTree<'a>,) {
        match self {
            ParseTree::Literal(tree) => tree.unbox(),
            _ => panic!("Expected Literal"),
        }
    }

    pub fn is_row_constructor(&self) -> bool {
        if let ParseTree::RowConstructor(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_row_constructor(&self) -> &RowConstructor {
        if let ParseTree::RowConstructor(value) = self {
            value
        } else {
            panic!("Expected RowConstructor")
        }
    }

    pub fn unbox_row_constructor(self) -> (ParseTree<'a>,) {
        match self {
            ParseTree::RowConstructor(tree) => tree.unbox(),
            _ => panic!("Expected RowConstructor"),
        }
    }

    pub fn is_parenthesized_expression(&self) -> bool {
        if let ParseTree::ParenthesizedExpression(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_parenthesized_expression(&self) -> &ParenthesizedExpression {
        if let ParseTree::ParenthesizedExpression(value) = self {
            value
        } else {
            panic!("Expected ParenthesizedExpression")
        }
    }

    pub fn unbox_parenthesized_expression(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::ParenthesizedExpression(tree) => tree.unbox(),
            _ => panic!("Expected ParenthesizedExpression"),
        }
    }

    pub fn is_identifier(&self) -> bool {
        if let ParseTree::Identifier(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_identifier(&self) -> &Identifier {
        if let ParseTree::Identifier(value) = self {
            value
        } else {
            panic!("Expected Identifier")
        }
    }

    pub fn unbox_identifier(self) -> (ParseTree<'a>,) {
        match self {
            ParseTree::Identifier(tree) => tree.unbox(),
            _ => panic!("Expected Identifier"),
        }
    }

    pub fn is_function_call(&self) -> bool {
        if let ParseTree::FunctionCall(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_function_call(&self) -> &FunctionCall {
        if let ParseTree::FunctionCall(value) = self {
            value
        } else {
            panic!("Expected FunctionCall")
        }
    }

    pub fn unbox_function_call(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::FunctionCall(tree) => tree.unbox(),
            _ => panic!("Expected FunctionCall"),
        }
    }

    pub fn is_filter(&self) -> bool {
        if let ParseTree::Filter(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_filter(&self) -> &Filter {
        if let ParseTree::Filter(value) = self {
            value
        } else {
            panic!("Expected Filter")
        }
    }

    pub fn unbox_filter(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Filter(tree) => tree.unbox(),
            _ => panic!("Expected Filter"),
        }
    }

    pub fn is_over(&self) -> bool {
        if let ParseTree::Over(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_over(&self) -> &Over {
        if let ParseTree::Over(value) = self {
            value
        } else {
            panic!("Expected Over")
        }
    }

    pub fn unbox_over(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Over(tree) => tree.unbox(),
            _ => panic!("Expected Over"),
        }
    }

    pub fn is_window_frame(&self) -> bool {
        if let ParseTree::WindowFrame(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_window_frame(&self) -> &WindowFrame {
        if let ParseTree::WindowFrame(value) = self {
            value
        } else {
            panic!("Expected WindowFrame")
        }
    }

    pub fn unbox_window_frame(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::WindowFrame(tree) => tree.unbox(),
            _ => panic!("Expected WindowFrame"),
        }
    }

    pub fn is_unbounded_frame(&self) -> bool {
        if let ParseTree::UnboundedFrame(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_unbounded_frame(&self) -> &UnboundedFrame {
        if let ParseTree::UnboundedFrame(value) = self {
            value
        } else {
            panic!("Expected UnboundedFrame")
        }
    }

    pub fn unbox_unbounded_frame(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::UnboundedFrame(tree) => tree.unbox(),
            _ => panic!("Expected UnboundedFrame"),
        }
    }

    pub fn is_current_row_bound(&self) -> bool {
        if let ParseTree::CurrentRowBound(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_current_row_bound(&self) -> &CurrentRowBound {
        if let ParseTree::CurrentRowBound(value) = self {
            value
        } else {
            panic!("Expected CurrentRowBound")
        }
    }

    pub fn unbox_current_row_bound(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::CurrentRowBound(tree) => tree.unbox(),
            _ => panic!("Expected CurrentRowBound"),
        }
    }

    pub fn is_bounded_frame(&self) -> bool {
        if let ParseTree::BoundedFrame(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_bounded_frame(&self) -> &BoundedFrame {
        if let ParseTree::BoundedFrame(value) = self {
            value
        } else {
            panic!("Expected BoundedFrame")
        }
    }

    pub fn unbox_bounded_frame(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::BoundedFrame(tree) => tree.unbox(),
            _ => panic!("Expected BoundedFrame"),
        }
    }

    pub fn is_unicode_string(&self) -> bool {
        if let ParseTree::UnicodeString(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_unicode_string(&self) -> &UnicodeString {
        if let ParseTree::UnicodeString(value) = self {
            value
        } else {
            panic!("Expected UnicodeString")
        }
    }

    pub fn unbox_unicode_string(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::UnicodeString(tree) => tree.unbox(),
            _ => panic!("Expected UnicodeString"),
        }
    }

    pub fn is_configure_expression(&self) -> bool {
        if let ParseTree::ConfigureExpression(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_configure_expression(&self) -> &ConfigureExpression {
        if let ParseTree::ConfigureExpression(value) = self {
            value
        } else {
            panic!("Expected ConfigureExpression")
        }
    }

    pub fn unbox_configure_expression(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::ConfigureExpression(tree) => tree.unbox(),
            _ => panic!("Expected ConfigureExpression"),
        }
    }

    pub fn is_subquery_expression(&self) -> bool {
        if let ParseTree::SubqueryExpression(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_subquery_expression(&self) -> &SubqueryExpression {
        if let ParseTree::SubqueryExpression(value) = self {
            value
        } else {
            panic!("Expected SubqueryExpression")
        }
    }

    pub fn unbox_subquery_expression(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::SubqueryExpression(tree) => tree.unbox(),
            _ => panic!("Expected SubqueryExpression"),
        }
    }

    pub fn is_grouping(&self) -> bool {
        if let ParseTree::Grouping(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_grouping(&self) -> &Grouping {
        if let ParseTree::Grouping(value) = self {
            value
        } else {
            panic!("Expected Grouping")
        }
    }

    pub fn unbox_grouping(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Grouping(tree) => tree.unbox(),
            _ => panic!("Expected Grouping"),
        }
    }

    pub fn is_extract(&self) -> bool {
        if let ParseTree::Extract(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_extract(&self) -> &Extract {
        if let ParseTree::Extract(value) = self {
            value
        } else {
            panic!("Expected Extract")
        }
    }

    pub fn unbox_extract(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Extract(tree) => tree.unbox(),
            _ => panic!("Expected Extract"),
        }
    }

    pub fn is_current_time(&self) -> bool {
        if let ParseTree::CurrentTime(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_current_time(&self) -> &CurrentTime {
        if let ParseTree::CurrentTime(value) = self {
            value
        } else {
            panic!("Expected CurrentTime")
        }
    }

    pub fn unbox_current_time(
        self,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::CurrentTime(tree) => tree.unbox(),
            _ => panic!("Expected CurrentTime"),
        }
    }

    pub fn is_current_timestamp(&self) -> bool {
        if let ParseTree::CurrentTimestamp(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_current_timestamp(&self) -> &CurrentTimestamp {
        if let ParseTree::CurrentTimestamp(value) = self {
            value
        } else {
            panic!("Expected CurrentTimestamp")
        }
    }

    pub fn unbox_current_timestamp(
        self,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::CurrentTimestamp(tree) => tree.unbox(),
            _ => panic!("Expected CurrentTimestamp"),
        }
    }

    pub fn is_normalize(&self) -> bool {
        if let ParseTree::Normalize(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_normalize(&self) -> &Normalize {
        if let ParseTree::Normalize(value) = self {
            value
        } else {
            panic!("Expected Normalize")
        }
    }

    pub fn unbox_normalize(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Normalize(tree) => tree.unbox(),
            _ => panic!("Expected Normalize"),
        }
    }

    pub fn is_localtime(&self) -> bool {
        if let ParseTree::Localtime(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_localtime(&self) -> &Localtime {
        if let ParseTree::Localtime(value) = self {
            value
        } else {
            panic!("Expected Localtime")
        }
    }

    pub fn unbox_localtime(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Localtime(tree) => tree.unbox(),
            _ => panic!("Expected Localtime"),
        }
    }

    pub fn is_localtimestamp(&self) -> bool {
        if let ParseTree::Localtimestamp(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_localtimestamp(&self) -> &Localtimestamp {
        if let ParseTree::Localtimestamp(value) = self {
            value
        } else {
            panic!("Expected Localtimestamp")
        }
    }

    pub fn unbox_localtimestamp(
        self,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Localtimestamp(tree) => tree.unbox(),
            _ => panic!("Expected Localtimestamp"),
        }
    }

    pub fn is_cast(&self) -> bool {
        if let ParseTree::Cast(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_cast(&self) -> &Cast {
        if let ParseTree::Cast(value) = self {
            value
        } else {
            panic!("Expected Cast")
        }
    }

    pub fn unbox_cast(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Cast(tree) => tree.unbox(),
            _ => panic!("Expected Cast"),
        }
    }

    pub fn is_when_clause(&self) -> bool {
        if let ParseTree::WhenClause(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_when_clause(&self) -> &WhenClause {
        if let ParseTree::WhenClause(value) = self {
            value
        } else {
            panic!("Expected WhenClause")
        }
    }

    pub fn unbox_when_clause(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::WhenClause(tree) => tree.unbox(),
            _ => panic!("Expected WhenClause"),
        }
    }

    pub fn is_case(&self) -> bool {
        if let ParseTree::Case(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_case(&self) -> &Case {
        if let ParseTree::Case(value) = self {
            value
        } else {
            panic!("Expected Case")
        }
    }

    pub fn unbox_case(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Case(tree) => tree.unbox(),
            _ => panic!("Expected Case"),
        }
    }

    pub fn is_exists(&self) -> bool {
        if let ParseTree::Exists(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_exists(&self) -> &Exists {
        if let ParseTree::Exists(value) = self {
            value
        } else {
            panic!("Expected Exists")
        }
    }

    pub fn unbox_exists(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Exists(tree) => tree.unbox(),
            _ => panic!("Expected Exists"),
        }
    }

    pub fn is_type_constructor(&self) -> bool {
        if let ParseTree::TypeConstructor(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_type_constructor(&self) -> &TypeConstructor {
        if let ParseTree::TypeConstructor(value) = self {
            value
        } else {
            panic!("Expected TypeConstructor")
        }
    }

    pub fn unbox_type_constructor(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::TypeConstructor(tree) => tree.unbox(),
            _ => panic!("Expected TypeConstructor"),
        }
    }

    pub fn is_array(&self) -> bool {
        if let ParseTree::Array(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_array(&self) -> &Array {
        if let ParseTree::Array(value) = self {
            value
        } else {
            panic!("Expected Array")
        }
    }

    pub fn unbox_array(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Array(tree) => tree.unbox(),
            _ => panic!("Expected Array"),
        }
    }

    pub fn is_interval(&self) -> bool {
        if let ParseTree::Interval(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_interval(&self) -> &Interval {
        if let ParseTree::Interval(value) = self {
            value
        } else {
            panic!("Expected Interval")
        }
    }

    pub fn unbox_interval(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Interval(tree) => tree.unbox(),
            _ => panic!("Expected Interval"),
        }
    }

    pub fn is_row(&self) -> bool {
        if let ParseTree::Row(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_row(&self) -> &Row {
        if let ParseTree::Row(value) = self {
            value
        } else {
            panic!("Expected Row")
        }
    }

    pub fn unbox_row(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Row(tree) => tree.unbox(),
            _ => panic!("Expected Row"),
        }
    }

    pub fn is_try_cast(&self) -> bool {
        if let ParseTree::TryCast(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_try_cast(&self) -> &TryCast {
        if let ParseTree::TryCast(value) = self {
            value
        } else {
            panic!("Expected TryCast")
        }
    }

    pub fn unbox_try_cast(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::TryCast(tree) => tree.unbox(),
            _ => panic!("Expected TryCast"),
        }
    }

    pub fn is_substring(&self) -> bool {
        if let ParseTree::Substring(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_substring(&self) -> &Substring {
        if let ParseTree::Substring(value) = self {
            value
        } else {
            panic!("Expected Substring")
        }
    }

    pub fn unbox_substring(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Substring(tree) => tree.unbox(),
            _ => panic!("Expected Substring"),
        }
    }

    pub fn is_position(&self) -> bool {
        if let ParseTree::Position(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_position(&self) -> &Position {
        if let ParseTree::Position(value) = self {
            value
        } else {
            panic!("Expected Position")
        }
    }

    pub fn unbox_position(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Position(tree) => tree.unbox(),
            _ => panic!("Expected Position"),
        }
    }

    pub fn is_array_type_suffix(&self) -> bool {
        if let ParseTree::ArrayTypeSuffix(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_array_type_suffix(&self) -> &ArrayTypeSuffix {
        if let ParseTree::ArrayTypeSuffix(value) = self {
            value
        } else {
            panic!("Expected ArrayTypeSuffix")
        }
    }

    pub fn unbox_array_type_suffix(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::ArrayTypeSuffix(tree) => tree.unbox(),
            _ => panic!("Expected ArrayTypeSuffix"),
        }
    }

    pub fn is_named_type(&self) -> bool {
        if let ParseTree::NamedType(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_named_type(&self) -> &NamedType {
        if let ParseTree::NamedType(value) = self {
            value
        } else {
            panic!("Expected NamedType")
        }
    }

    pub fn unbox_named_type(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::NamedType(tree) => tree.unbox(),
            _ => panic!("Expected NamedType"),
        }
    }

    pub fn is_array_type(&self) -> bool {
        if let ParseTree::ArrayType(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_array_type(&self) -> &ArrayType {
        if let ParseTree::ArrayType(value) = self {
            value
        } else {
            panic!("Expected ArrayType")
        }
    }

    pub fn unbox_array_type(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::ArrayType(tree) => tree.unbox(),
            _ => panic!("Expected ArrayType"),
        }
    }

    pub fn is_map_type(&self) -> bool {
        if let ParseTree::MapType(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_map_type(&self) -> &MapType {
        if let ParseTree::MapType(value) = self {
            value
        } else {
            panic!("Expected MapType")
        }
    }

    pub fn unbox_map_type(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::MapType(tree) => tree.unbox(),
            _ => panic!("Expected MapType"),
        }
    }

    pub fn is_row_type(&self) -> bool {
        if let ParseTree::RowType(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_row_type(&self) -> &RowType {
        if let ParseTree::RowType(value) = self {
            value
        } else {
            panic!("Expected RowType")
        }
    }

    pub fn unbox_row_type(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::RowType(tree) => tree.unbox(),
            _ => panic!("Expected RowType"),
        }
    }

    pub fn is_row_type_element(&self) -> bool {
        if let ParseTree::RowTypeElement(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_row_type_element(&self) -> &RowTypeElement {
        if let ParseTree::RowTypeElement(value) = self {
            value
        } else {
            panic!("Expected RowTypeElement")
        }
    }

    pub fn unbox_row_type_element(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::RowTypeElement(tree) => tree.unbox(),
            _ => panic!("Expected RowTypeElement"),
        }
    }

    pub fn is_interval_type(&self) -> bool {
        if let ParseTree::IntervalType(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_interval_type(&self) -> &IntervalType {
        if let ParseTree::IntervalType(value) = self {
            value
        } else {
            panic!("Expected IntervalType")
        }
    }

    pub fn unbox_interval_type(
        self,
    ) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::IntervalType(tree) => tree.unbox(),
            _ => panic!("Expected IntervalType"),
        }
    }

    pub fn is_if_not_exists(&self) -> bool {
        if let ParseTree::IfNotExists(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_if_not_exists(&self) -> &IfNotExists {
        if let ParseTree::IfNotExists(value) = self {
            value
        } else {
            panic!("Expected IfNotExists")
        }
    }

    pub fn unbox_if_not_exists(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::IfNotExists(tree) => tree.unbox(),
            _ => panic!("Expected IfNotExists"),
        }
    }

    pub fn is_create_table(&self) -> bool {
        if let ParseTree::CreateTable(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_create_table(&self) -> &CreateTable {
        if let ParseTree::CreateTable(value) = self {
            value
        } else {
            panic!("Expected CreateTable")
        }
    }

    pub fn unbox_create_table(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::CreateTable(tree) => tree.unbox(),
            _ => panic!("Expected CreateTable"),
        }
    }

    pub fn is_create_table_as_select(&self) -> bool {
        if let ParseTree::CreateTableAsSelect(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_create_table_as_select(&self) -> &CreateTableAsSelect {
        if let ParseTree::CreateTableAsSelect(value) = self {
            value
        } else {
            panic!("Expected CreateTableAsSelect")
        }
    }

    pub fn unbox_create_table_as_select(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::CreateTableAsSelect(tree) => tree.unbox(),
            _ => panic!("Expected CreateTableAsSelect"),
        }
    }

    pub fn is_with_properties(&self) -> bool {
        if let ParseTree::WithProperties(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_with_properties(&self) -> &WithProperties {
        if let ParseTree::WithProperties(value) = self {
            value
        } else {
            panic!("Expected WithProperties")
        }
    }

    pub fn unbox_with_properties(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::WithProperties(tree) => tree.unbox(),
            _ => panic!("Expected WithProperties"),
        }
    }

    pub fn is_property(&self) -> bool {
        if let ParseTree::Property(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_property(&self) -> &Property {
        if let ParseTree::Property(value) = self {
            value
        } else {
            panic!("Expected Property")
        }
    }

    pub fn unbox_property(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Property(tree) => tree.unbox(),
            _ => panic!("Expected Property"),
        }
    }

    pub fn is_with_data(&self) -> bool {
        if let ParseTree::WithData(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_with_data(&self) -> &WithData {
        if let ParseTree::WithData(value) = self {
            value
        } else {
            panic!("Expected WithData")
        }
    }

    pub fn unbox_with_data(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::WithData(tree) => tree.unbox(),
            _ => panic!("Expected WithData"),
        }
    }

    pub fn is_comment(&self) -> bool {
        if let ParseTree::Comment(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_comment(&self) -> &Comment {
        if let ParseTree::Comment(value) = self {
            value
        } else {
            panic!("Expected Comment")
        }
    }

    pub fn unbox_comment(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Comment(tree) => tree.unbox(),
            _ => panic!("Expected Comment"),
        }
    }

    pub fn is_column_definition(&self) -> bool {
        if let ParseTree::ColumnDefinition(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_column_definition(&self) -> &ColumnDefinition {
        if let ParseTree::ColumnDefinition(value) = self {
            value
        } else {
            panic!("Expected ColumnDefinition")
        }
    }

    pub fn unbox_column_definition(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::ColumnDefinition(tree) => tree.unbox(),
            _ => panic!("Expected ColumnDefinition"),
        }
    }

    pub fn is_not_null(&self) -> bool {
        if let ParseTree::NotNull(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_not_null(&self) -> &NotNull {
        if let ParseTree::NotNull(value) = self {
            value
        } else {
            panic!("Expected NotNull")
        }
    }

    pub fn unbox_not_null(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::NotNull(tree) => tree.unbox(),
            _ => panic!("Expected NotNull"),
        }
    }

    pub fn is_like_clause(&self) -> bool {
        if let ParseTree::LikeClause(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_like_clause(&self) -> &LikeClause {
        if let ParseTree::LikeClause(value) = self {
            value
        } else {
            panic!("Expected LikeClause")
        }
    }

    pub fn unbox_like_clause(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::LikeClause(tree) => tree.unbox(),
            _ => panic!("Expected LikeClause"),
        }
    }

    pub fn is_insert_into(&self) -> bool {
        if let ParseTree::InsertInto(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_insert_into(&self) -> &InsertInto {
        if let ParseTree::InsertInto(value) = self {
            value
        } else {
            panic!("Expected InsertInto")
        }
    }

    pub fn unbox_insert_into(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::InsertInto(tree) => tree.unbox(),
            _ => panic!("Expected InsertInto"),
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

impl<'a> Query<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Query(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.with, *self.query_no_with)
    }
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

impl<'a> With<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::With(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.with, *self.recursive, *self.named_queries)
    }
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

impl<'a> NamedQuery<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::NamedQuery(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.name,
            *self.column_aliases_opt,
            *self.as_,
            *self.open_paren,
            *self.query,
            *self.close_paren,
        )
    }
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

impl<'a> QueryNoWith<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::QueryNoWith(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.query_term, *self.order_by_opt, *self.limit_opt)
    }
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

impl<'a> OrderBy<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::OrderBy(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.order, *self.by, *self.sort_items)
    }
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

impl<'a> Limit<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Limit(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.limit, *self.value)
    }
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

impl<'a> QuerySetOperation<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::QuerySetOperation(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.left,
            *self.operator,
            *self.set_quantifier_opt,
            *self.right,
        )
    }
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

impl<'a> SortItem<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::SortItem(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.expression,
            *self.ordering_opt,
            *self.nulls,
            *self.null_ordering_opt,
        )
    }
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

impl<'a> Subquery<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Subquery(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.open_paren, *self.query_no_with, *self.close_paren)
    }
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

impl<'a> InlineTable<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::InlineTable(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.values, *self.expressions)
    }
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

impl<'a> Table<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Table(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.table, *self.qualified_name)
    }
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

impl<'a> QuerySpecification<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::QuerySpecification(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.select,
            *self.set_quantifier_opt,
            *self.select_items,
            *self.from,
            *self.relations,
            *self.where_,
            *self.where_predicate,
            *self.group,
            *self.by,
            *self.group_by,
            *self.having,
            *self.having_predicate,
        )
    }
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

impl<'a> QualifiedName<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::QualifiedName(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.names,)
    }
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

impl<'a> SelectAll<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::SelectAll(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.asterisk,)
    }
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

impl<'a> QualifiedSelectAll<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::QualifiedSelectAll(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.qualifier, *self.period, *self.asterisk)
    }
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

impl<'a> SelectItem<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::SelectItem(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.expression, *self.as_, *self.identifier)
    }
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

impl<'a> SubqueryRelation<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::SubqueryRelation(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.open_paren, *self.query, *self.close_paren)
    }
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

impl<'a> ParenthesizedRelation<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::ParenthesizedRelation(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.open_paren, *self.relation, *self.close_paren)
    }
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

impl<'a> TableName<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::TableName(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.name,)
    }
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

impl<'a> Lateral<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Lateral(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.lateral,
            *self.open_paren,
            *self.query,
            *self.close_paren,
        )
    }
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

impl<'a> Unnest<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Unnest(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.unnest,
            *self.expressions,
            *self.with,
            *self.ordinality,
        )
    }
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

impl<'a> SampledRelation<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::SampledRelation(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.aliased_relation,
            *self.tablesample,
            *self.sample_type,
            *self.open_paren,
            *self.expression,
            *self.close_paren,
        )
    }
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

impl<'a> AliasedRelation<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::AliasedRelation(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.relation_primary,
            *self.as_opt,
            *self.identifier,
            *self.column_aliases_opt,
        )
    }
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

impl<'a> CrossJoin<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::CrossJoin(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.left, *self.cross, *self.join, *self.right)
    }
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

impl<'a> Join<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Join(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.left,
            *self.join_type,
            *self.join,
            *self.right,
            *self.join_criteria,
        )
    }
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

impl<'a> NaturalJoin<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::NaturalJoin(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.left,
            *self.natural,
            *self.join_type,
            *self.join,
            *self.right,
        )
    }
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

impl<'a> OuterJoinKind<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::OuterJoinKind(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.kind, *self.outer_opt)
    }
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

impl<'a> OnJoinCriteria<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::OnJoinCriteria(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.on, *self.predicate)
    }
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

impl<'a> UsingJoinCriteria<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::UsingJoinCriteria(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.using, *self.names)
    }
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

impl<'a> GroupBy<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::GroupBy(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.set_quantifier_opt, *self.grouping_elements)
    }
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

impl<'a> Rollup<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Rollup(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.rollup, *self.expressions)
    }
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

impl<'a> Cube<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Cube(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.cube, *self.expressions)
    }
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

impl<'a> GroupingSets<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::GroupingSets(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.grouping, *self.sets, *self.grouping_sets)
    }
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

impl<'a> BinaryExpression<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::BinaryExpression(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.left, *self.operator, *self.right)
    }
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

impl<'a> UnaryExpression<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::UnaryExpression(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.operator, *self.operand)
    }
}

#[derive(Clone, Debug)]
pub struct QuantifiedComparison<'a> {
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
    ParseTree::QuantifiedComparison(QuantifiedComparison {
        operand: Box::new(operand),
        operator: Box::new(operator),
        comparison_quantifier: Box::new(comparison_quantifier),
        open_paren: Box::new(open_paren),
        query: Box::new(query),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> QuantifiedComparison<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::QuantifiedComparison(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.operand,
            *self.operator,
            *self.comparison_quantifier,
            *self.open_paren,
            *self.query,
            *self.close_paren,
        )
    }
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

impl<'a> NullPredicate<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::NullPredicate(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.value, *self.is, *self.not_opt, *self.null)
    }
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

impl<'a> DistinctFrom<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::DistinctFrom(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.left, *self.distinct, *self.from, *self.right)
    }
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

impl<'a> Between<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Between(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.value,
            *self.not_opt,
            *self.between,
            *self.lower,
            *self.and,
            *self.upper,
        )
    }
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

impl<'a> Like<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Like(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.value,
            *self.not_opt,
            *self.like,
            *self.patrern,
            *self.escape_opt,
            *self.escape_value_opt,
        )
    }
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

impl<'a> InSubquery<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::InSubquery(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.value,
            *self.not_opt,
            *self.in_,
            *self.open_paren,
            *self.query,
            *self.close_paren,
        )
    }
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

impl<'a> InList<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::InList(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.value, *self.not_opt, *self.in_, *self.expressions)
    }
}

#[derive(Clone, Debug)]
pub struct AtTimeZone<'a> {
    pub value: Box<ParseTree<'a>>,
    pub at: Box<ParseTree<'a>>,
    pub time: Box<ParseTree<'a>>,
    pub zone: Box<ParseTree<'a>>,
    pub specifier: Box<ParseTree<'a>>,
}

pub fn at_time_zone<'a>(
    value: ParseTree<'a>,
    at: ParseTree<'a>,
    time: ParseTree<'a>,
    zone: ParseTree<'a>,
    specifier: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::AtTimeZone(AtTimeZone {
        value: Box::new(value),
        at: Box::new(at),
        time: Box::new(time),
        zone: Box::new(zone),
        specifier: Box::new(specifier),
    })
}

impl<'a> AtTimeZone<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::AtTimeZone(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.value,
            *self.at,
            *self.time,
            *self.zone,
            *self.specifier,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Dereference<'a> {
    pub object: Box<ParseTree<'a>>,
    pub period: Box<ParseTree<'a>>,
    pub field_name: Box<ParseTree<'a>>,
}

pub fn dereference<'a>(
    object: ParseTree<'a>,
    period: ParseTree<'a>,
    field_name: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Dereference(Dereference {
        object: Box::new(object),
        period: Box::new(period),
        field_name: Box::new(field_name),
    })
}

impl<'a> Dereference<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Dereference(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.object, *self.period, *self.field_name)
    }
}

#[derive(Clone, Debug)]
pub struct Subscript<'a> {
    pub operand: Box<ParseTree<'a>>,
    pub open_square: Box<ParseTree<'a>>,
    pub index: Box<ParseTree<'a>>,
    pub close_square: Box<ParseTree<'a>>,
}

pub fn subscript<'a>(
    operand: ParseTree<'a>,
    open_square: ParseTree<'a>,
    index: ParseTree<'a>,
    close_square: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Subscript(Subscript {
        operand: Box::new(operand),
        open_square: Box::new(open_square),
        index: Box::new(index),
        close_square: Box::new(close_square),
    })
}

impl<'a> Subscript<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Subscript(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.operand,
            *self.open_square,
            *self.index,
            *self.close_square,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Lambda<'a> {
    pub parameters: Box<ParseTree<'a>>,
    pub array: Box<ParseTree<'a>>,
    pub body: Box<ParseTree<'a>>,
}

pub fn lambda<'a>(
    parameters: ParseTree<'a>,
    array: ParseTree<'a>,
    body: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Lambda(Lambda {
        parameters: Box::new(parameters),
        array: Box::new(array),
        body: Box::new(body),
    })
}

impl<'a> Lambda<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Lambda(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.parameters, *self.array, *self.body)
    }
}

#[derive(Clone, Debug)]
pub struct Literal<'a> {
    pub value: Box<ParseTree<'a>>,
}

pub fn literal<'a>(value: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Literal(Literal {
        value: Box::new(value),
    })
}

impl<'a> Literal<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Literal(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.value,)
    }
}

#[derive(Clone, Debug)]
pub struct RowConstructor<'a> {
    pub elements: Box<ParseTree<'a>>,
}

pub fn row_constructor<'a>(elements: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::RowConstructor(RowConstructor {
        elements: Box::new(elements),
    })
}

impl<'a> RowConstructor<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::RowConstructor(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.elements,)
    }
}

#[derive(Clone, Debug)]
pub struct ParenthesizedExpression<'a> {
    pub open_paren: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn parenthesized_expression<'a>(
    open_paren: ParseTree<'a>,
    value: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::ParenthesizedExpression(ParenthesizedExpression {
        open_paren: Box::new(open_paren),
        value: Box::new(value),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> ParenthesizedExpression<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::ParenthesizedExpression(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.open_paren, *self.value, *self.close_paren)
    }
}

#[derive(Clone, Debug)]
pub struct Identifier<'a> {
    pub value: Box<ParseTree<'a>>,
}

pub fn identifier<'a>(value: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Identifier(Identifier {
        value: Box::new(value),
    })
}

impl<'a> Identifier<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Identifier(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.value,)
    }
}

#[derive(Clone, Debug)]
pub struct FunctionCall<'a> {
    pub name: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub set_quantifier_opt: Box<ParseTree<'a>>,
    pub arguments: Box<ParseTree<'a>>,
    pub order_by_opt: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
    pub filter_opt: Box<ParseTree<'a>>,
    pub over_opt: Box<ParseTree<'a>>,
}

pub fn function_call<'a>(
    name: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    set_quantifier_opt: ParseTree<'a>,
    arguments: ParseTree<'a>,
    order_by_opt: ParseTree<'a>,
    close_paren: ParseTree<'a>,
    filter_opt: ParseTree<'a>,
    over_opt: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::FunctionCall(FunctionCall {
        name: Box::new(name),
        open_paren: Box::new(open_paren),
        set_quantifier_opt: Box::new(set_quantifier_opt),
        arguments: Box::new(arguments),
        order_by_opt: Box::new(order_by_opt),
        close_paren: Box::new(close_paren),
        filter_opt: Box::new(filter_opt),
        over_opt: Box::new(over_opt),
    })
}

impl<'a> FunctionCall<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::FunctionCall(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.name,
            *self.open_paren,
            *self.set_quantifier_opt,
            *self.arguments,
            *self.order_by_opt,
            *self.close_paren,
            *self.filter_opt,
            *self.over_opt,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Filter<'a> {
    pub filter: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub where_: Box<ParseTree<'a>>,
    pub predicate: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn filter<'a>(
    filter: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    where_: ParseTree<'a>,
    predicate: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Filter(Filter {
        filter: Box::new(filter),
        open_paren: Box::new(open_paren),
        where_: Box::new(where_),
        predicate: Box::new(predicate),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> Filter<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Filter(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.filter,
            *self.open_paren,
            *self.where_,
            *self.predicate,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Over<'a> {
    pub over: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub partition_opt: Box<ParseTree<'a>>,
    pub by: Box<ParseTree<'a>>,
    pub partitions: Box<ParseTree<'a>>,
    pub order_by_opt: Box<ParseTree<'a>>,
    pub window_frame: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn over<'a>(
    over: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    partition_opt: ParseTree<'a>,
    by: ParseTree<'a>,
    partitions: ParseTree<'a>,
    order_by_opt: ParseTree<'a>,
    window_frame: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Over(Over {
        over: Box::new(over),
        open_paren: Box::new(open_paren),
        partition_opt: Box::new(partition_opt),
        by: Box::new(by),
        partitions: Box::new(partitions),
        order_by_opt: Box::new(order_by_opt),
        window_frame: Box::new(window_frame),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> Over<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Over(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.over,
            *self.open_paren,
            *self.partition_opt,
            *self.by,
            *self.partitions,
            *self.order_by_opt,
            *self.window_frame,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct WindowFrame<'a> {
    pub frame_type: Box<ParseTree<'a>>,
    pub between_opt: Box<ParseTree<'a>>,
    pub start: Box<ParseTree<'a>>,
    pub and: Box<ParseTree<'a>>,
    pub end: Box<ParseTree<'a>>,
}

pub fn window_frame<'a>(
    frame_type: ParseTree<'a>,
    between_opt: ParseTree<'a>,
    start: ParseTree<'a>,
    and: ParseTree<'a>,
    end: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::WindowFrame(WindowFrame {
        frame_type: Box::new(frame_type),
        between_opt: Box::new(between_opt),
        start: Box::new(start),
        and: Box::new(and),
        end: Box::new(end),
    })
}

impl<'a> WindowFrame<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::WindowFrame(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.frame_type,
            *self.between_opt,
            *self.start,
            *self.and,
            *self.end,
        )
    }
}

#[derive(Clone, Debug)]
pub struct UnboundedFrame<'a> {
    pub unbounded: Box<ParseTree<'a>>,
    pub bound_type: Box<ParseTree<'a>>,
}

pub fn unbounded_frame<'a>(unbounded: ParseTree<'a>, bound_type: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::UnboundedFrame(UnboundedFrame {
        unbounded: Box::new(unbounded),
        bound_type: Box::new(bound_type),
    })
}

impl<'a> UnboundedFrame<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::UnboundedFrame(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.unbounded, *self.bound_type)
    }
}

#[derive(Clone, Debug)]
pub struct CurrentRowBound<'a> {
    pub current: Box<ParseTree<'a>>,
    pub row: Box<ParseTree<'a>>,
}

pub fn current_row_bound<'a>(current: ParseTree<'a>, row: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::CurrentRowBound(CurrentRowBound {
        current: Box::new(current),
        row: Box::new(row),
    })
}

impl<'a> CurrentRowBound<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::CurrentRowBound(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.current, *self.row)
    }
}

#[derive(Clone, Debug)]
pub struct BoundedFrame<'a> {
    pub bound: Box<ParseTree<'a>>,
    pub bound_type: Box<ParseTree<'a>>,
}

pub fn bounded_frame<'a>(bound: ParseTree<'a>, bound_type: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::BoundedFrame(BoundedFrame {
        bound: Box::new(bound),
        bound_type: Box::new(bound_type),
    })
}

impl<'a> BoundedFrame<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::BoundedFrame(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.bound, *self.bound_type)
    }
}

#[derive(Clone, Debug)]
pub struct UnicodeString<'a> {
    pub string: Box<ParseTree<'a>>,
    pub uescape_opt: Box<ParseTree<'a>>,
    pub escape: Box<ParseTree<'a>>,
}

pub fn unicode_string<'a>(
    string: ParseTree<'a>,
    uescape_opt: ParseTree<'a>,
    escape: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::UnicodeString(UnicodeString {
        string: Box::new(string),
        uescape_opt: Box::new(uescape_opt),
        escape: Box::new(escape),
    })
}

impl<'a> UnicodeString<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::UnicodeString(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.string, *self.uescape_opt, *self.escape)
    }
}

#[derive(Clone, Debug)]
pub struct ConfigureExpression<'a> {
    pub configure: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub identifier: Box<ParseTree<'a>>,
    pub comma: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn configure_expression<'a>(
    configure: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    identifier: ParseTree<'a>,
    comma: ParseTree<'a>,
    value: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::ConfigureExpression(ConfigureExpression {
        configure: Box::new(configure),
        open_paren: Box::new(open_paren),
        identifier: Box::new(identifier),
        comma: Box::new(comma),
        value: Box::new(value),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> ConfigureExpression<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::ConfigureExpression(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.configure,
            *self.open_paren,
            *self.identifier,
            *self.comma,
            *self.value,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct SubqueryExpression<'a> {
    pub open_paren: Box<ParseTree<'a>>,
    pub query: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn subquery_expression<'a>(
    open_paren: ParseTree<'a>,
    query: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::SubqueryExpression(SubqueryExpression {
        open_paren: Box::new(open_paren),
        query: Box::new(query),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> SubqueryExpression<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::SubqueryExpression(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.open_paren, *self.query, *self.close_paren)
    }
}

#[derive(Clone, Debug)]
pub struct Grouping<'a> {
    pub grouping: Box<ParseTree<'a>>,
    pub groups: Box<ParseTree<'a>>,
}

pub fn grouping<'a>(grouping: ParseTree<'a>, groups: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Grouping(Grouping {
        grouping: Box::new(grouping),
        groups: Box::new(groups),
    })
}

impl<'a> Grouping<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Grouping(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.grouping, *self.groups)
    }
}

#[derive(Clone, Debug)]
pub struct Extract<'a> {
    pub extract: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub identifier: Box<ParseTree<'a>>,
    pub from: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn extract<'a>(
    extract: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    identifier: ParseTree<'a>,
    from: ParseTree<'a>,
    value: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Extract(Extract {
        extract: Box::new(extract),
        open_paren: Box::new(open_paren),
        identifier: Box::new(identifier),
        from: Box::new(from),
        value: Box::new(value),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> Extract<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Extract(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.extract,
            *self.open_paren,
            *self.identifier,
            *self.from,
            *self.value,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct CurrentTime<'a> {
    pub current_time: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub precision: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn current_time<'a>(
    current_time: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    precision: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::CurrentTime(CurrentTime {
        current_time: Box::new(current_time),
        open_paren: Box::new(open_paren),
        precision: Box::new(precision),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> CurrentTime<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::CurrentTime(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.current_time,
            *self.open_paren,
            *self.precision,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct CurrentTimestamp<'a> {
    pub current_timestamp: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub precision: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn current_timestamp<'a>(
    current_timestamp: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    precision: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::CurrentTimestamp(CurrentTimestamp {
        current_timestamp: Box::new(current_timestamp),
        open_paren: Box::new(open_paren),
        precision: Box::new(precision),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> CurrentTimestamp<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::CurrentTimestamp(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.current_timestamp,
            *self.open_paren,
            *self.precision,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Normalize<'a> {
    pub normalize: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
    pub comma_opt: Box<ParseTree<'a>>,
    pub normal_form: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn normalize<'a>(
    normalize: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    value: ParseTree<'a>,
    comma_opt: ParseTree<'a>,
    normal_form: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Normalize(Normalize {
        normalize: Box::new(normalize),
        open_paren: Box::new(open_paren),
        value: Box::new(value),
        comma_opt: Box::new(comma_opt),
        normal_form: Box::new(normal_form),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> Normalize<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Normalize(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.normalize,
            *self.open_paren,
            *self.value,
            *self.comma_opt,
            *self.normal_form,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Localtime<'a> {
    pub localtime: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub precision: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn localtime<'a>(
    localtime: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    precision: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Localtime(Localtime {
        localtime: Box::new(localtime),
        open_paren: Box::new(open_paren),
        precision: Box::new(precision),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> Localtime<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Localtime(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.localtime,
            *self.open_paren,
            *self.precision,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Localtimestamp<'a> {
    pub localtimestamp: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub precision: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn localtimestamp<'a>(
    localtimestamp: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    precision: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Localtimestamp(Localtimestamp {
        localtimestamp: Box::new(localtimestamp),
        open_paren: Box::new(open_paren),
        precision: Box::new(precision),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> Localtimestamp<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Localtimestamp(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.localtimestamp,
            *self.open_paren,
            *self.precision,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Cast<'a> {
    pub cast: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
    pub as_: Box<ParseTree<'a>>,
    pub type_: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn cast<'a>(
    cast: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    value: ParseTree<'a>,
    as_: ParseTree<'a>,
    type_: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Cast(Cast {
        cast: Box::new(cast),
        open_paren: Box::new(open_paren),
        value: Box::new(value),
        as_: Box::new(as_),
        type_: Box::new(type_),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> Cast<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Cast(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.cast,
            *self.open_paren,
            *self.value,
            *self.as_,
            *self.type_,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct WhenClause<'a> {
    pub when: Box<ParseTree<'a>>,
    pub condition: Box<ParseTree<'a>>,
    pub then: Box<ParseTree<'a>>,
    pub result: Box<ParseTree<'a>>,
}

pub fn when_clause<'a>(
    when: ParseTree<'a>,
    condition: ParseTree<'a>,
    then: ParseTree<'a>,
    result: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::WhenClause(WhenClause {
        when: Box::new(when),
        condition: Box::new(condition),
        then: Box::new(then),
        result: Box::new(result),
    })
}

impl<'a> WhenClause<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::WhenClause(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.when, *self.condition, *self.then, *self.result)
    }
}

#[derive(Clone, Debug)]
pub struct Case<'a> {
    pub case: Box<ParseTree<'a>>,
    pub value_opt: Box<ParseTree<'a>>,
    pub when_clauses: Box<ParseTree<'a>>,
    pub else_opt: Box<ParseTree<'a>>,
    pub default: Box<ParseTree<'a>>,
    pub end: Box<ParseTree<'a>>,
}

pub fn case<'a>(
    case: ParseTree<'a>,
    value_opt: ParseTree<'a>,
    when_clauses: ParseTree<'a>,
    else_opt: ParseTree<'a>,
    default: ParseTree<'a>,
    end: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Case(Case {
        case: Box::new(case),
        value_opt: Box::new(value_opt),
        when_clauses: Box::new(when_clauses),
        else_opt: Box::new(else_opt),
        default: Box::new(default),
        end: Box::new(end),
    })
}

impl<'a> Case<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Case(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.case,
            *self.value_opt,
            *self.when_clauses,
            *self.else_opt,
            *self.default,
            *self.end,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Exists<'a> {
    pub exists: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub query: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn exists<'a>(
    exists: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    query: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Exists(Exists {
        exists: Box::new(exists),
        open_paren: Box::new(open_paren),
        query: Box::new(query),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> Exists<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Exists(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.exists,
            *self.open_paren,
            *self.query,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct TypeConstructor<'a> {
    pub type_: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
}

pub fn type_constructor<'a>(type_: ParseTree<'a>, value: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::TypeConstructor(TypeConstructor {
        type_: Box::new(type_),
        value: Box::new(value),
    })
}

impl<'a> TypeConstructor<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::TypeConstructor(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.type_, *self.value)
    }
}

#[derive(Clone, Debug)]
pub struct Array<'a> {
    pub array: Box<ParseTree<'a>>,
    pub elements: Box<ParseTree<'a>>,
}

pub fn array<'a>(array: ParseTree<'a>, elements: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Array(Array {
        array: Box::new(array),
        elements: Box::new(elements),
    })
}

impl<'a> Array<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Array(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.array, *self.elements)
    }
}

#[derive(Clone, Debug)]
pub struct Interval<'a> {
    pub interval: Box<ParseTree<'a>>,
    pub sign_opt: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
    pub from: Box<ParseTree<'a>>,
    pub to_kw_opt: Box<ParseTree<'a>>,
    pub to: Box<ParseTree<'a>>,
}

pub fn interval<'a>(
    interval: ParseTree<'a>,
    sign_opt: ParseTree<'a>,
    value: ParseTree<'a>,
    from: ParseTree<'a>,
    to_kw_opt: ParseTree<'a>,
    to: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Interval(Interval {
        interval: Box::new(interval),
        sign_opt: Box::new(sign_opt),
        value: Box::new(value),
        from: Box::new(from),
        to_kw_opt: Box::new(to_kw_opt),
        to: Box::new(to),
    })
}

impl<'a> Interval<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Interval(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.interval,
            *self.sign_opt,
            *self.value,
            *self.from,
            *self.to_kw_opt,
            *self.to,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Row<'a> {
    pub row: Box<ParseTree<'a>>,
    pub elements: Box<ParseTree<'a>>,
}

pub fn row<'a>(row: ParseTree<'a>, elements: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Row(Row {
        row: Box::new(row),
        elements: Box::new(elements),
    })
}

impl<'a> Row<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Row(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.row, *self.elements)
    }
}

#[derive(Clone, Debug)]
pub struct TryCast<'a> {
    pub try_cast: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
    pub as_: Box<ParseTree<'a>>,
    pub type_: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn try_cast<'a>(
    try_cast: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    value: ParseTree<'a>,
    as_: ParseTree<'a>,
    type_: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::TryCast(TryCast {
        try_cast: Box::new(try_cast),
        open_paren: Box::new(open_paren),
        value: Box::new(value),
        as_: Box::new(as_),
        type_: Box::new(type_),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> TryCast<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::TryCast(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.try_cast,
            *self.open_paren,
            *self.value,
            *self.as_,
            *self.type_,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Substring<'a> {
    pub substring: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
    pub from: Box<ParseTree<'a>>,
    pub from_value: Box<ParseTree<'a>>,
    pub for_opt: Box<ParseTree<'a>>,
    pub for_value: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn substring<'a>(
    substring: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    value: ParseTree<'a>,
    from: ParseTree<'a>,
    from_value: ParseTree<'a>,
    for_opt: ParseTree<'a>,
    for_value: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Substring(Substring {
        substring: Box::new(substring),
        open_paren: Box::new(open_paren),
        value: Box::new(value),
        from: Box::new(from),
        from_value: Box::new(from_value),
        for_opt: Box::new(for_opt),
        for_value: Box::new(for_value),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> Substring<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Substring(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.substring,
            *self.open_paren,
            *self.value,
            *self.from,
            *self.from_value,
            *self.for_opt,
            *self.for_value,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Position<'a> {
    pub position: Box<ParseTree<'a>>,
    pub open_paren: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
    pub in_: Box<ParseTree<'a>>,
    pub target: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn position<'a>(
    position: ParseTree<'a>,
    open_paren: ParseTree<'a>,
    value: ParseTree<'a>,
    in_: ParseTree<'a>,
    target: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Position(Position {
        position: Box::new(position),
        open_paren: Box::new(open_paren),
        value: Box::new(value),
        in_: Box::new(in_),
        target: Box::new(target),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> Position<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Position(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.position,
            *self.open_paren,
            *self.value,
            *self.in_,
            *self.target,
            *self.close_paren,
        )
    }
}

#[derive(Clone, Debug)]
pub struct ArrayTypeSuffix<'a> {
    pub base_type: Box<ParseTree<'a>>,
    pub array: Box<ParseTree<'a>>,
}

pub fn array_type_suffix<'a>(base_type: ParseTree<'a>, array: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::ArrayTypeSuffix(ArrayTypeSuffix {
        base_type: Box::new(base_type),
        array: Box::new(array),
    })
}

impl<'a> ArrayTypeSuffix<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::ArrayTypeSuffix(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.base_type, *self.array)
    }
}

#[derive(Clone, Debug)]
pub struct NamedType<'a> {
    pub name: Box<ParseTree<'a>>,
    pub type_parameters: Box<ParseTree<'a>>,
}

pub fn named_type<'a>(name: ParseTree<'a>, type_parameters: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::NamedType(NamedType {
        name: Box::new(name),
        type_parameters: Box::new(type_parameters),
    })
}

impl<'a> NamedType<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::NamedType(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.name, *self.type_parameters)
    }
}

#[derive(Clone, Debug)]
pub struct ArrayType<'a> {
    pub array: Box<ParseTree<'a>>,
    pub open_angle: Box<ParseTree<'a>>,
    pub element_type: Box<ParseTree<'a>>,
    pub close_angle: Box<ParseTree<'a>>,
}

pub fn array_type<'a>(
    array: ParseTree<'a>,
    open_angle: ParseTree<'a>,
    element_type: ParseTree<'a>,
    close_angle: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::ArrayType(ArrayType {
        array: Box::new(array),
        open_angle: Box::new(open_angle),
        element_type: Box::new(element_type),
        close_angle: Box::new(close_angle),
    })
}

impl<'a> ArrayType<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::ArrayType(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.array,
            *self.open_angle,
            *self.element_type,
            *self.close_angle,
        )
    }
}

#[derive(Clone, Debug)]
pub struct MapType<'a> {
    pub map: Box<ParseTree<'a>>,
    pub open_angle: Box<ParseTree<'a>>,
    pub key_type: Box<ParseTree<'a>>,
    pub comma: Box<ParseTree<'a>>,
    pub value_type: Box<ParseTree<'a>>,
    pub close_angle: Box<ParseTree<'a>>,
}

pub fn map_type<'a>(
    map: ParseTree<'a>,
    open_angle: ParseTree<'a>,
    key_type: ParseTree<'a>,
    comma: ParseTree<'a>,
    value_type: ParseTree<'a>,
    close_angle: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::MapType(MapType {
        map: Box::new(map),
        open_angle: Box::new(open_angle),
        key_type: Box::new(key_type),
        comma: Box::new(comma),
        value_type: Box::new(value_type),
        close_angle: Box::new(close_angle),
    })
}

impl<'a> MapType<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::MapType(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.map,
            *self.open_angle,
            *self.key_type,
            *self.comma,
            *self.value_type,
            *self.close_angle,
        )
    }
}

#[derive(Clone, Debug)]
pub struct RowType<'a> {
    pub row: Box<ParseTree<'a>>,
    pub element_types: Box<ParseTree<'a>>,
}

pub fn row_type<'a>(row: ParseTree<'a>, element_types: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::RowType(RowType {
        row: Box::new(row),
        element_types: Box::new(element_types),
    })
}

impl<'a> RowType<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::RowType(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.row, *self.element_types)
    }
}

#[derive(Clone, Debug)]
pub struct RowTypeElement<'a> {
    pub identifier: Box<ParseTree<'a>>,
    pub type_: Box<ParseTree<'a>>,
}

pub fn row_type_element<'a>(identifier: ParseTree<'a>, type_: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::RowTypeElement(RowTypeElement {
        identifier: Box::new(identifier),
        type_: Box::new(type_),
    })
}

impl<'a> RowTypeElement<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::RowTypeElement(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.identifier, *self.type_)
    }
}

#[derive(Clone, Debug)]
pub struct IntervalType<'a> {
    pub interval: Box<ParseTree<'a>>,
    pub from: Box<ParseTree<'a>>,
    pub to_kw: Box<ParseTree<'a>>,
    pub to: Box<ParseTree<'a>>,
}

pub fn interval_type<'a>(
    interval: ParseTree<'a>,
    from: ParseTree<'a>,
    to_kw: ParseTree<'a>,
    to: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::IntervalType(IntervalType {
        interval: Box::new(interval),
        from: Box::new(from),
        to_kw: Box::new(to_kw),
        to: Box::new(to),
    })
}

impl<'a> IntervalType<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::IntervalType(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.interval, *self.from, *self.to_kw, *self.to)
    }
}

#[derive(Clone, Debug)]
pub struct IfNotExists<'a> {
    pub if_: Box<ParseTree<'a>>,
    pub not: Box<ParseTree<'a>>,
    pub exists: Box<ParseTree<'a>>,
}

pub fn if_not_exists<'a>(
    if_: ParseTree<'a>,
    not: ParseTree<'a>,
    exists: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::IfNotExists(IfNotExists {
        if_: Box::new(if_),
        not: Box::new(not),
        exists: Box::new(exists),
    })
}

impl<'a> IfNotExists<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::IfNotExists(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.if_, *self.not, *self.exists)
    }
}

#[derive(Clone, Debug)]
pub struct CreateTable<'a> {
    pub create: Box<ParseTree<'a>>,
    pub table: Box<ParseTree<'a>>,
    pub if_not_exists_opt: Box<ParseTree<'a>>,
    pub table_name: Box<ParseTree<'a>>,
    pub table_elements: Box<ParseTree<'a>>,
    pub comment_opt: Box<ParseTree<'a>>,
    pub with_properties_opt: Box<ParseTree<'a>>,
}

pub fn create_table<'a>(
    create: ParseTree<'a>,
    table: ParseTree<'a>,
    if_not_exists_opt: ParseTree<'a>,
    table_name: ParseTree<'a>,
    table_elements: ParseTree<'a>,
    comment_opt: ParseTree<'a>,
    with_properties_opt: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::CreateTable(CreateTable {
        create: Box::new(create),
        table: Box::new(table),
        if_not_exists_opt: Box::new(if_not_exists_opt),
        table_name: Box::new(table_name),
        table_elements: Box::new(table_elements),
        comment_opt: Box::new(comment_opt),
        with_properties_opt: Box::new(with_properties_opt),
    })
}

impl<'a> CreateTable<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::CreateTable(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.create,
            *self.table,
            *self.if_not_exists_opt,
            *self.table_name,
            *self.table_elements,
            *self.comment_opt,
            *self.with_properties_opt,
        )
    }
}

#[derive(Clone, Debug)]
pub struct CreateTableAsSelect<'a> {
    pub create: Box<ParseTree<'a>>,
    pub table: Box<ParseTree<'a>>,
    pub if_not_exists_opt: Box<ParseTree<'a>>,
    pub table_name: Box<ParseTree<'a>>,
    pub column_aliases_opt: Box<ParseTree<'a>>,
    pub comment_opt: Box<ParseTree<'a>>,
    pub with_properties_opt: Box<ParseTree<'a>>,
    pub as_: Box<ParseTree<'a>>,
    pub open_paren_opt: Box<ParseTree<'a>>,
    pub query: Box<ParseTree<'a>>,
    pub close_paren_opt: Box<ParseTree<'a>>,
    pub with_data_opt: Box<ParseTree<'a>>,
}

pub fn create_table_as_select<'a>(
    create: ParseTree<'a>,
    table: ParseTree<'a>,
    if_not_exists_opt: ParseTree<'a>,
    table_name: ParseTree<'a>,
    column_aliases_opt: ParseTree<'a>,
    comment_opt: ParseTree<'a>,
    with_properties_opt: ParseTree<'a>,
    as_: ParseTree<'a>,
    open_paren_opt: ParseTree<'a>,
    query: ParseTree<'a>,
    close_paren_opt: ParseTree<'a>,
    with_data_opt: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::CreateTableAsSelect(CreateTableAsSelect {
        create: Box::new(create),
        table: Box::new(table),
        if_not_exists_opt: Box::new(if_not_exists_opt),
        table_name: Box::new(table_name),
        column_aliases_opt: Box::new(column_aliases_opt),
        comment_opt: Box::new(comment_opt),
        with_properties_opt: Box::new(with_properties_opt),
        as_: Box::new(as_),
        open_paren_opt: Box::new(open_paren_opt),
        query: Box::new(query),
        close_paren_opt: Box::new(close_paren_opt),
        with_data_opt: Box::new(with_data_opt),
    })
}

impl<'a> CreateTableAsSelect<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::CreateTableAsSelect(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.create,
            *self.table,
            *self.if_not_exists_opt,
            *self.table_name,
            *self.column_aliases_opt,
            *self.comment_opt,
            *self.with_properties_opt,
            *self.as_,
            *self.open_paren_opt,
            *self.query,
            *self.close_paren_opt,
            *self.with_data_opt,
        )
    }
}

#[derive(Clone, Debug)]
pub struct WithProperties<'a> {
    pub with: Box<ParseTree<'a>>,
    pub properties: Box<ParseTree<'a>>,
}

pub fn with_properties<'a>(with: ParseTree<'a>, properties: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::WithProperties(WithProperties {
        with: Box::new(with),
        properties: Box::new(properties),
    })
}

impl<'a> WithProperties<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::WithProperties(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.with, *self.properties)
    }
}

#[derive(Clone, Debug)]
pub struct Property<'a> {
    pub identifier: Box<ParseTree<'a>>,
    pub eq: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
}

pub fn property<'a>(
    identifier: ParseTree<'a>,
    eq: ParseTree<'a>,
    value: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Property(Property {
        identifier: Box::new(identifier),
        eq: Box::new(eq),
        value: Box::new(value),
    })
}

impl<'a> Property<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Property(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.identifier, *self.eq, *self.value)
    }
}

#[derive(Clone, Debug)]
pub struct WithData<'a> {
    pub with: Box<ParseTree<'a>>,
    pub no_opt: Box<ParseTree<'a>>,
    pub data: Box<ParseTree<'a>>,
}

pub fn with_data<'a>(
    with: ParseTree<'a>,
    no_opt: ParseTree<'a>,
    data: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::WithData(WithData {
        with: Box::new(with),
        no_opt: Box::new(no_opt),
        data: Box::new(data),
    })
}

impl<'a> WithData<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::WithData(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.with, *self.no_opt, *self.data)
    }
}

#[derive(Clone, Debug)]
pub struct Comment<'a> {
    pub comment: Box<ParseTree<'a>>,
    pub value: Box<ParseTree<'a>>,
}

pub fn comment<'a>(comment: ParseTree<'a>, value: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::Comment(Comment {
        comment: Box::new(comment),
        value: Box::new(value),
    })
}

impl<'a> Comment<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Comment(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.comment, *self.value)
    }
}

#[derive(Clone, Debug)]
pub struct ColumnDefinition<'a> {
    pub identifier: Box<ParseTree<'a>>,
    pub type_: Box<ParseTree<'a>>,
    pub not_null_opt: Box<ParseTree<'a>>,
    pub comment_opt: Box<ParseTree<'a>>,
    pub with_properties_opt: Box<ParseTree<'a>>,
}

pub fn column_definition<'a>(
    identifier: ParseTree<'a>,
    type_: ParseTree<'a>,
    not_null_opt: ParseTree<'a>,
    comment_opt: ParseTree<'a>,
    with_properties_opt: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::ColumnDefinition(ColumnDefinition {
        identifier: Box::new(identifier),
        type_: Box::new(type_),
        not_null_opt: Box::new(not_null_opt),
        comment_opt: Box::new(comment_opt),
        with_properties_opt: Box::new(with_properties_opt),
    })
}

impl<'a> ColumnDefinition<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::ColumnDefinition(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.identifier,
            *self.type_,
            *self.not_null_opt,
            *self.comment_opt,
            *self.with_properties_opt,
        )
    }
}

#[derive(Clone, Debug)]
pub struct NotNull<'a> {
    pub not: Box<ParseTree<'a>>,
    pub null: Box<ParseTree<'a>>,
}

pub fn not_null<'a>(not: ParseTree<'a>, null: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::NotNull(NotNull {
        not: Box::new(not),
        null: Box::new(null),
    })
}

impl<'a> NotNull<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::NotNull(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.not, *self.null)
    }
}

#[derive(Clone, Debug)]
pub struct LikeClause<'a> {
    pub like: Box<ParseTree<'a>>,
    pub name: Box<ParseTree<'a>>,
    pub option_type_opt: Box<ParseTree<'a>>,
    pub properties: Box<ParseTree<'a>>,
}

pub fn like_clause<'a>(
    like: ParseTree<'a>,
    name: ParseTree<'a>,
    option_type_opt: ParseTree<'a>,
    properties: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::LikeClause(LikeClause {
        like: Box::new(like),
        name: Box::new(name),
        option_type_opt: Box::new(option_type_opt),
        properties: Box::new(properties),
    })
}

impl<'a> LikeClause<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::LikeClause(self)
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.like,
            *self.name,
            *self.option_type_opt,
            *self.properties,
        )
    }
}

#[derive(Clone, Debug)]
pub struct InsertInto<'a> {
    pub insert: Box<ParseTree<'a>>,
    pub into: Box<ParseTree<'a>>,
    pub table_name: Box<ParseTree<'a>>,
    pub column_aliases_opt: Box<ParseTree<'a>>,
    pub query: Box<ParseTree<'a>>,
}

pub fn insert_into<'a>(
    insert: ParseTree<'a>,
    into: ParseTree<'a>,
    table_name: ParseTree<'a>,
    column_aliases_opt: ParseTree<'a>,
    query: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::InsertInto(InsertInto {
        insert: Box::new(insert),
        into: Box::new(into),
        table_name: Box::new(table_name),
        column_aliases_opt: Box::new(column_aliases_opt),
        query: Box::new(query),
    })
}

impl<'a> InsertInto<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::InsertInto(self)
    }

    pub fn unbox(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        (
            *self.insert,
            *self.into,
            *self.table_name,
            *self.column_aliases_opt,
            *self.query,
        )
    }
}
