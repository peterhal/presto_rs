use crate::lexing::token;
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
    CreateView(CreateView<'a>),
    CreateRole(CreateRole<'a>),
    WithAdminGrantor(WithAdminGrantor<'a>),
    UserPrincipal(UserPrincipal<'a>),
    RolePrincipal(RolePrincipal<'a>),
    UnspecifiedPrincipal(UnspecifiedPrincipal<'a>),
    CreateTableAsSelect(CreateTableAsSelect<'a>),
    WithProperties(WithProperties<'a>),
    Property(Property<'a>),
    WithData(WithData<'a>),
    Comment(Comment<'a>),
    ColumnDefinition(ColumnDefinition<'a>),
    NotNull(NotNull<'a>),
    LikeClause(LikeClause<'a>),
    InsertInto(InsertInto<'a>),
    Delete(Delete<'a>),
    GroupingSet(GroupingSet<'a>),
    RelationOrQuery(RelationOrQuery<'a>),
    EmptyGroupingSet(EmptyGroupingSet<'a>),
    ExpressionOrQuery(ExpressionOrQuery<'a>),
    Entrypoint(Entrypoint<'a>),
    NullTreatment(NullTreatment<'a>),
}

// The core trees
#[derive(Clone, Debug)]
pub struct Empty {
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

#[derive(Clone, Debug)]
pub struct Token<'a> {
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
        self.start_delimiter
            .get_first_token()
            .or_else(|| {
                for (element, separator) in &self.elements_and_separators {
                    let result = element
                        .get_first_token()
                        .or_else(|| separator.get_first_token());
                    if result.is_some() {
                        return result;
                    }
                }
                None
            })
            .or_else(|| self.end_delimiter.get_first_token())
    }

    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        self.end_delimiter
            .get_last_token()
            .or_else(|| {
                for (element, separator) in self.elements_and_separators.iter().rev() {
                    let result = element
                        .get_last_token()
                        .or_else(|| separator.get_last_token());
                    if result.is_some() {
                        return result;
                    }
                }
                None
            })
            .or_else(|| self.start_delimiter.get_last_token())
    }
}

#[derive(Clone, Debug)]
pub struct Error {
    pub error: SyntaxError,
}

pub fn error<'a>(error: SyntaxError) -> ParseTree<'a> {
    ParseTree::Error(Error { error })
}

impl Error {
    pub fn children(&self) -> Vec<&'static ParseTree<'static>> {
        Vec::new()
    }
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

    // Note: Has poor performance O(tree depth)
    pub fn get_start(&self) -> position::Position {
        match self {
            ParseTree::Empty(empty) => empty.range.start,
            ParseTree::Error(error) => error.error.get_range().start,
            _ => match self.get_first_token() {
                Some(token) => token.range.start,
                // All children are empty or errors
                None => self.get_first_child().get_start(),
            },
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
            },
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
            },
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
            },
        }
    }

    // Note: Has poor performance O(tree depth)
    pub fn get_range(&self) -> TextRange {
        TextRange::new(self.get_start(), self.get_end())
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

    pub fn is_create_view(&self) -> bool {
        if let ParseTree::CreateView(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_create_view(&self) -> &CreateView {
        if let ParseTree::CreateView(value) = self {
            value
        } else {
            panic!("Expected CreateView")
        }
    }

    pub fn unbox_create_view(
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
            ParseTree::CreateView(tree) => tree.unbox(),
            _ => panic!("Expected CreateView"),
        }
    }

    pub fn is_create_role(&self) -> bool {
        if let ParseTree::CreateRole(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_create_role(&self) -> &CreateRole {
        if let ParseTree::CreateRole(value) = self {
            value
        } else {
            panic!("Expected CreateRole")
        }
    }

    pub fn unbox_create_role(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::CreateRole(tree) => tree.unbox(),
            _ => panic!("Expected CreateRole"),
        }
    }

    pub fn is_with_admin_grantor(&self) -> bool {
        if let ParseTree::WithAdminGrantor(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_with_admin_grantor(&self) -> &WithAdminGrantor {
        if let ParseTree::WithAdminGrantor(value) = self {
            value
        } else {
            panic!("Expected WithAdminGrantor")
        }
    }

    pub fn unbox_with_admin_grantor(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::WithAdminGrantor(tree) => tree.unbox(),
            _ => panic!("Expected WithAdminGrantor"),
        }
    }

    pub fn is_user_principal(&self) -> bool {
        if let ParseTree::UserPrincipal(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_user_principal(&self) -> &UserPrincipal {
        if let ParseTree::UserPrincipal(value) = self {
            value
        } else {
            panic!("Expected UserPrincipal")
        }
    }

    pub fn unbox_user_principal(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::UserPrincipal(tree) => tree.unbox(),
            _ => panic!("Expected UserPrincipal"),
        }
    }

    pub fn is_role_principal(&self) -> bool {
        if let ParseTree::RolePrincipal(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_role_principal(&self) -> &RolePrincipal {
        if let ParseTree::RolePrincipal(value) = self {
            value
        } else {
            panic!("Expected RolePrincipal")
        }
    }

    pub fn unbox_role_principal(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::RolePrincipal(tree) => tree.unbox(),
            _ => panic!("Expected RolePrincipal"),
        }
    }

    pub fn is_unspecified_principal(&self) -> bool {
        if let ParseTree::UnspecifiedPrincipal(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_unspecified_principal(&self) -> &UnspecifiedPrincipal {
        if let ParseTree::UnspecifiedPrincipal(value) = self {
            value
        } else {
            panic!("Expected UnspecifiedPrincipal")
        }
    }

    pub fn unbox_unspecified_principal(self) -> (ParseTree<'a>,) {
        match self {
            ParseTree::UnspecifiedPrincipal(tree) => tree.unbox(),
            _ => panic!("Expected UnspecifiedPrincipal"),
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

    pub fn is_delete(&self) -> bool {
        if let ParseTree::Delete(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_delete(&self) -> &Delete {
        if let ParseTree::Delete(value) = self {
            value
        } else {
            panic!("Expected Delete")
        }
    }

    pub fn unbox_delete(
        self,
    ) -> (
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
        ParseTree<'a>,
    ) {
        match self {
            ParseTree::Delete(tree) => tree.unbox(),
            _ => panic!("Expected Delete"),
        }
    }

    pub fn is_grouping_set(&self) -> bool {
        if let ParseTree::GroupingSet(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_grouping_set(&self) -> &GroupingSet {
        if let ParseTree::GroupingSet(value) = self {
            value
        } else {
            panic!("Expected GroupingSet")
        }
    }

    pub fn unbox_grouping_set(self) -> (ParseTree<'a>,) {
        match self {
            ParseTree::GroupingSet(tree) => tree.unbox(),
            _ => panic!("Expected GroupingSet"),
        }
    }

    pub fn is_relation_or_query(&self) -> bool {
        if let ParseTree::RelationOrQuery(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_relation_or_query(&self) -> &RelationOrQuery {
        if let ParseTree::RelationOrQuery(value) = self {
            value
        } else {
            panic!("Expected RelationOrQuery")
        }
    }

    pub fn unbox_relation_or_query(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::RelationOrQuery(tree) => tree.unbox(),
            _ => panic!("Expected RelationOrQuery"),
        }
    }

    pub fn is_empty_grouping_set(&self) -> bool {
        if let ParseTree::EmptyGroupingSet(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_empty_grouping_set(&self) -> &EmptyGroupingSet {
        if let ParseTree::EmptyGroupingSet(value) = self {
            value
        } else {
            panic!("Expected EmptyGroupingSet")
        }
    }

    pub fn unbox_empty_grouping_set(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::EmptyGroupingSet(tree) => tree.unbox(),
            _ => panic!("Expected EmptyGroupingSet"),
        }
    }

    pub fn is_expression_or_query(&self) -> bool {
        if let ParseTree::ExpressionOrQuery(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_expression_or_query(&self) -> &ExpressionOrQuery {
        if let ParseTree::ExpressionOrQuery(value) = self {
            value
        } else {
            panic!("Expected ExpressionOrQuery")
        }
    }

    pub fn unbox_expression_or_query(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::ExpressionOrQuery(tree) => tree.unbox(),
            _ => panic!("Expected ExpressionOrQuery"),
        }
    }

    pub fn is_entrypoint(&self) -> bool {
        if let ParseTree::Entrypoint(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_entrypoint(&self) -> &Entrypoint {
        if let ParseTree::Entrypoint(value) = self {
            value
        } else {
            panic!("Expected Entrypoint")
        }
    }

    pub fn unbox_entrypoint(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::Entrypoint(tree) => tree.unbox(),
            _ => panic!("Expected Entrypoint"),
        }
    }

    pub fn is_null_treatment(&self) -> bool {
        if let ParseTree::NullTreatment(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_null_treatment(&self) -> &NullTreatment {
        if let ParseTree::NullTreatment(value) = self {
            value
        } else {
            panic!("Expected NullTreatment")
        }
    }

    pub fn unbox_null_treatment(self) -> (ParseTree<'a>, ParseTree<'a>) {
        match self {
            ParseTree::NullTreatment(tree) => tree.unbox(),
            _ => panic!("Expected NullTreatment"),
        }
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        match self {
            ParseTree::Token(token) => token.children(),
            ParseTree::List(list) => list.children(),
            ParseTree::Error(error) => error.children(),
            ParseTree::Empty(empty) => empty.children(),
            ParseTree::Query(query) => query.children(),
            ParseTree::With(with) => with.children(),
            ParseTree::NamedQuery(named_query) => named_query.children(),
            ParseTree::QueryNoWith(query_no_with) => query_no_with.children(),
            ParseTree::OrderBy(order_by) => order_by.children(),
            ParseTree::Limit(limit) => limit.children(),
            ParseTree::QuerySetOperation(query_set_operation) => query_set_operation.children(),
            ParseTree::SortItem(sort_item) => sort_item.children(),
            ParseTree::Subquery(subquery) => subquery.children(),
            ParseTree::InlineTable(inline_table) => inline_table.children(),
            ParseTree::Table(table) => table.children(),
            ParseTree::QuerySpecification(query_specification) => query_specification.children(),
            ParseTree::QualifiedName(qualified_name) => qualified_name.children(),
            ParseTree::SelectAll(select_all) => select_all.children(),
            ParseTree::QualifiedSelectAll(qualified_select_all) => qualified_select_all.children(),
            ParseTree::SelectItem(select_item) => select_item.children(),
            ParseTree::SubqueryRelation(subquery_relation) => subquery_relation.children(),
            ParseTree::ParenthesizedRelation(parenthesized_relation) => {
                parenthesized_relation.children()
            }
            ParseTree::TableName(table_name) => table_name.children(),
            ParseTree::Lateral(lateral) => lateral.children(),
            ParseTree::Unnest(unnest) => unnest.children(),
            ParseTree::SampledRelation(sampled_relation) => sampled_relation.children(),
            ParseTree::AliasedRelation(aliased_relation) => aliased_relation.children(),
            ParseTree::CrossJoin(cross_join) => cross_join.children(),
            ParseTree::Join(join) => join.children(),
            ParseTree::NaturalJoin(natural_join) => natural_join.children(),
            ParseTree::OuterJoinKind(outer_join_kind) => outer_join_kind.children(),
            ParseTree::OnJoinCriteria(on_join_criteria) => on_join_criteria.children(),
            ParseTree::UsingJoinCriteria(using_join_criteria) => using_join_criteria.children(),
            ParseTree::GroupBy(group_by) => group_by.children(),
            ParseTree::Rollup(rollup) => rollup.children(),
            ParseTree::Cube(cube) => cube.children(),
            ParseTree::GroupingSets(grouping_sets) => grouping_sets.children(),
            ParseTree::BinaryExpression(binary_expression) => binary_expression.children(),
            ParseTree::UnaryExpression(unary_expression) => unary_expression.children(),
            ParseTree::QuantifiedComparison(quantified_comparison) => {
                quantified_comparison.children()
            }
            ParseTree::NullPredicate(null_predicate) => null_predicate.children(),
            ParseTree::DistinctFrom(distinct_from) => distinct_from.children(),
            ParseTree::Between(between) => between.children(),
            ParseTree::Like(like) => like.children(),
            ParseTree::InSubquery(in_subquery) => in_subquery.children(),
            ParseTree::InList(in_list) => in_list.children(),
            ParseTree::AtTimeZone(at_time_zone) => at_time_zone.children(),
            ParseTree::Dereference(dereference) => dereference.children(),
            ParseTree::Subscript(subscript) => subscript.children(),
            ParseTree::Lambda(lambda) => lambda.children(),
            ParseTree::Literal(literal) => literal.children(),
            ParseTree::RowConstructor(row_constructor) => row_constructor.children(),
            ParseTree::ParenthesizedExpression(parenthesized_expression) => {
                parenthesized_expression.children()
            }
            ParseTree::Identifier(identifier) => identifier.children(),
            ParseTree::FunctionCall(function_call) => function_call.children(),
            ParseTree::Filter(filter) => filter.children(),
            ParseTree::Over(over) => over.children(),
            ParseTree::WindowFrame(window_frame) => window_frame.children(),
            ParseTree::UnboundedFrame(unbounded_frame) => unbounded_frame.children(),
            ParseTree::CurrentRowBound(current_row_bound) => current_row_bound.children(),
            ParseTree::BoundedFrame(bounded_frame) => bounded_frame.children(),
            ParseTree::UnicodeString(unicode_string) => unicode_string.children(),
            ParseTree::ConfigureExpression(configure_expression) => configure_expression.children(),
            ParseTree::SubqueryExpression(subquery_expression) => subquery_expression.children(),
            ParseTree::Grouping(grouping) => grouping.children(),
            ParseTree::Extract(extract) => extract.children(),
            ParseTree::CurrentTime(current_time) => current_time.children(),
            ParseTree::CurrentTimestamp(current_timestamp) => current_timestamp.children(),
            ParseTree::Normalize(normalize) => normalize.children(),
            ParseTree::Localtime(localtime) => localtime.children(),
            ParseTree::Localtimestamp(localtimestamp) => localtimestamp.children(),
            ParseTree::Cast(cast) => cast.children(),
            ParseTree::WhenClause(when_clause) => when_clause.children(),
            ParseTree::Case(case) => case.children(),
            ParseTree::Exists(exists) => exists.children(),
            ParseTree::TypeConstructor(type_constructor) => type_constructor.children(),
            ParseTree::Array(array) => array.children(),
            ParseTree::Interval(interval) => interval.children(),
            ParseTree::Row(row) => row.children(),
            ParseTree::TryCast(try_cast) => try_cast.children(),
            ParseTree::Substring(substring) => substring.children(),
            ParseTree::Position(position) => position.children(),
            ParseTree::ArrayTypeSuffix(array_type_suffix) => array_type_suffix.children(),
            ParseTree::NamedType(named_type) => named_type.children(),
            ParseTree::ArrayType(array_type) => array_type.children(),
            ParseTree::MapType(map_type) => map_type.children(),
            ParseTree::RowType(row_type) => row_type.children(),
            ParseTree::RowTypeElement(row_type_element) => row_type_element.children(),
            ParseTree::IntervalType(interval_type) => interval_type.children(),
            ParseTree::IfNotExists(if_not_exists) => if_not_exists.children(),
            ParseTree::CreateTable(create_table) => create_table.children(),
            ParseTree::CreateView(create_view) => create_view.children(),
            ParseTree::CreateRole(create_role) => create_role.children(),
            ParseTree::WithAdminGrantor(with_admin_grantor) => with_admin_grantor.children(),
            ParseTree::UserPrincipal(user_principal) => user_principal.children(),
            ParseTree::RolePrincipal(role_principal) => role_principal.children(),
            ParseTree::UnspecifiedPrincipal(unspecified_principal) => {
                unspecified_principal.children()
            }
            ParseTree::CreateTableAsSelect(create_table_as_select) => {
                create_table_as_select.children()
            }
            ParseTree::WithProperties(with_properties) => with_properties.children(),
            ParseTree::Property(property) => property.children(),
            ParseTree::WithData(with_data) => with_data.children(),
            ParseTree::Comment(comment) => comment.children(),
            ParseTree::ColumnDefinition(column_definition) => column_definition.children(),
            ParseTree::NotNull(not_null) => not_null.children(),
            ParseTree::LikeClause(like_clause) => like_clause.children(),
            ParseTree::InsertInto(insert_into) => insert_into.children(),
            ParseTree::Delete(delete) => delete.children(),
            ParseTree::GroupingSet(grouping_set) => grouping_set.children(),
            ParseTree::RelationOrQuery(relation_or_query) => relation_or_query.children(),
            ParseTree::EmptyGroupingSet(empty_grouping_set) => empty_grouping_set.children(),
            ParseTree::ExpressionOrQuery(expression_or_query) => expression_or_query.children(),
            ParseTree::Entrypoint(entrypoint) => entrypoint.children(),
            ParseTree::NullTreatment(null_treatment) => null_treatment.children(),
        }
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        match self {
            ParseTree::Token(_) => self,
            ParseTree::List(list) => &list.start_delimiter,
            ParseTree::Error(_) => self,
            ParseTree::Empty(_) => self,
            ParseTree::Query(query) => query.get_first_child(),
            ParseTree::With(with) => with.get_first_child(),
            ParseTree::NamedQuery(named_query) => named_query.get_first_child(),
            ParseTree::QueryNoWith(query_no_with) => query_no_with.get_first_child(),
            ParseTree::OrderBy(order_by) => order_by.get_first_child(),
            ParseTree::Limit(limit) => limit.get_first_child(),
            ParseTree::QuerySetOperation(query_set_operation) => {
                query_set_operation.get_first_child()
            }
            ParseTree::SortItem(sort_item) => sort_item.get_first_child(),
            ParseTree::Subquery(subquery) => subquery.get_first_child(),
            ParseTree::InlineTable(inline_table) => inline_table.get_first_child(),
            ParseTree::Table(table) => table.get_first_child(),
            ParseTree::QuerySpecification(query_specification) => {
                query_specification.get_first_child()
            }
            ParseTree::QualifiedName(qualified_name) => qualified_name.get_first_child(),
            ParseTree::SelectAll(select_all) => select_all.get_first_child(),
            ParseTree::QualifiedSelectAll(qualified_select_all) => {
                qualified_select_all.get_first_child()
            }
            ParseTree::SelectItem(select_item) => select_item.get_first_child(),
            ParseTree::SubqueryRelation(subquery_relation) => subquery_relation.get_first_child(),
            ParseTree::ParenthesizedRelation(parenthesized_relation) => {
                parenthesized_relation.get_first_child()
            }
            ParseTree::TableName(table_name) => table_name.get_first_child(),
            ParseTree::Lateral(lateral) => lateral.get_first_child(),
            ParseTree::Unnest(unnest) => unnest.get_first_child(),
            ParseTree::SampledRelation(sampled_relation) => sampled_relation.get_first_child(),
            ParseTree::AliasedRelation(aliased_relation) => aliased_relation.get_first_child(),
            ParseTree::CrossJoin(cross_join) => cross_join.get_first_child(),
            ParseTree::Join(join) => join.get_first_child(),
            ParseTree::NaturalJoin(natural_join) => natural_join.get_first_child(),
            ParseTree::OuterJoinKind(outer_join_kind) => outer_join_kind.get_first_child(),
            ParseTree::OnJoinCriteria(on_join_criteria) => on_join_criteria.get_first_child(),
            ParseTree::UsingJoinCriteria(using_join_criteria) => {
                using_join_criteria.get_first_child()
            }
            ParseTree::GroupBy(group_by) => group_by.get_first_child(),
            ParseTree::Rollup(rollup) => rollup.get_first_child(),
            ParseTree::Cube(cube) => cube.get_first_child(),
            ParseTree::GroupingSets(grouping_sets) => grouping_sets.get_first_child(),
            ParseTree::BinaryExpression(binary_expression) => binary_expression.get_first_child(),
            ParseTree::UnaryExpression(unary_expression) => unary_expression.get_first_child(),
            ParseTree::QuantifiedComparison(quantified_comparison) => {
                quantified_comparison.get_first_child()
            }
            ParseTree::NullPredicate(null_predicate) => null_predicate.get_first_child(),
            ParseTree::DistinctFrom(distinct_from) => distinct_from.get_first_child(),
            ParseTree::Between(between) => between.get_first_child(),
            ParseTree::Like(like) => like.get_first_child(),
            ParseTree::InSubquery(in_subquery) => in_subquery.get_first_child(),
            ParseTree::InList(in_list) => in_list.get_first_child(),
            ParseTree::AtTimeZone(at_time_zone) => at_time_zone.get_first_child(),
            ParseTree::Dereference(dereference) => dereference.get_first_child(),
            ParseTree::Subscript(subscript) => subscript.get_first_child(),
            ParseTree::Lambda(lambda) => lambda.get_first_child(),
            ParseTree::Literal(literal) => literal.get_first_child(),
            ParseTree::RowConstructor(row_constructor) => row_constructor.get_first_child(),
            ParseTree::ParenthesizedExpression(parenthesized_expression) => {
                parenthesized_expression.get_first_child()
            }
            ParseTree::Identifier(identifier) => identifier.get_first_child(),
            ParseTree::FunctionCall(function_call) => function_call.get_first_child(),
            ParseTree::Filter(filter) => filter.get_first_child(),
            ParseTree::Over(over) => over.get_first_child(),
            ParseTree::WindowFrame(window_frame) => window_frame.get_first_child(),
            ParseTree::UnboundedFrame(unbounded_frame) => unbounded_frame.get_first_child(),
            ParseTree::CurrentRowBound(current_row_bound) => current_row_bound.get_first_child(),
            ParseTree::BoundedFrame(bounded_frame) => bounded_frame.get_first_child(),
            ParseTree::UnicodeString(unicode_string) => unicode_string.get_first_child(),
            ParseTree::ConfigureExpression(configure_expression) => {
                configure_expression.get_first_child()
            }
            ParseTree::SubqueryExpression(subquery_expression) => {
                subquery_expression.get_first_child()
            }
            ParseTree::Grouping(grouping) => grouping.get_first_child(),
            ParseTree::Extract(extract) => extract.get_first_child(),
            ParseTree::CurrentTime(current_time) => current_time.get_first_child(),
            ParseTree::CurrentTimestamp(current_timestamp) => current_timestamp.get_first_child(),
            ParseTree::Normalize(normalize) => normalize.get_first_child(),
            ParseTree::Localtime(localtime) => localtime.get_first_child(),
            ParseTree::Localtimestamp(localtimestamp) => localtimestamp.get_first_child(),
            ParseTree::Cast(cast) => cast.get_first_child(),
            ParseTree::WhenClause(when_clause) => when_clause.get_first_child(),
            ParseTree::Case(case) => case.get_first_child(),
            ParseTree::Exists(exists) => exists.get_first_child(),
            ParseTree::TypeConstructor(type_constructor) => type_constructor.get_first_child(),
            ParseTree::Array(array) => array.get_first_child(),
            ParseTree::Interval(interval) => interval.get_first_child(),
            ParseTree::Row(row) => row.get_first_child(),
            ParseTree::TryCast(try_cast) => try_cast.get_first_child(),
            ParseTree::Substring(substring) => substring.get_first_child(),
            ParseTree::Position(position) => position.get_first_child(),
            ParseTree::ArrayTypeSuffix(array_type_suffix) => array_type_suffix.get_first_child(),
            ParseTree::NamedType(named_type) => named_type.get_first_child(),
            ParseTree::ArrayType(array_type) => array_type.get_first_child(),
            ParseTree::MapType(map_type) => map_type.get_first_child(),
            ParseTree::RowType(row_type) => row_type.get_first_child(),
            ParseTree::RowTypeElement(row_type_element) => row_type_element.get_first_child(),
            ParseTree::IntervalType(interval_type) => interval_type.get_first_child(),
            ParseTree::IfNotExists(if_not_exists) => if_not_exists.get_first_child(),
            ParseTree::CreateTable(create_table) => create_table.get_first_child(),
            ParseTree::CreateView(create_view) => create_view.get_first_child(),
            ParseTree::CreateRole(create_role) => create_role.get_first_child(),
            ParseTree::WithAdminGrantor(with_admin_grantor) => with_admin_grantor.get_first_child(),
            ParseTree::UserPrincipal(user_principal) => user_principal.get_first_child(),
            ParseTree::RolePrincipal(role_principal) => role_principal.get_first_child(),
            ParseTree::UnspecifiedPrincipal(unspecified_principal) => {
                unspecified_principal.get_first_child()
            }
            ParseTree::CreateTableAsSelect(create_table_as_select) => {
                create_table_as_select.get_first_child()
            }
            ParseTree::WithProperties(with_properties) => with_properties.get_first_child(),
            ParseTree::Property(property) => property.get_first_child(),
            ParseTree::WithData(with_data) => with_data.get_first_child(),
            ParseTree::Comment(comment) => comment.get_first_child(),
            ParseTree::ColumnDefinition(column_definition) => column_definition.get_first_child(),
            ParseTree::NotNull(not_null) => not_null.get_first_child(),
            ParseTree::LikeClause(like_clause) => like_clause.get_first_child(),
            ParseTree::InsertInto(insert_into) => insert_into.get_first_child(),
            ParseTree::Delete(delete) => delete.get_first_child(),
            ParseTree::GroupingSet(grouping_set) => grouping_set.get_first_child(),
            ParseTree::RelationOrQuery(relation_or_query) => relation_or_query.get_first_child(),
            ParseTree::EmptyGroupingSet(empty_grouping_set) => empty_grouping_set.get_first_child(),
            ParseTree::ExpressionOrQuery(expression_or_query) => {
                expression_or_query.get_first_child()
            }
            ParseTree::Entrypoint(entrypoint) => entrypoint.get_first_child(),
            ParseTree::NullTreatment(null_treatment) => null_treatment.get_first_child(),
        }
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        match self {
            ParseTree::Token(_) => self,
            ParseTree::List(list) => &list.end_delimiter,
            ParseTree::Error(_) => self,
            ParseTree::Empty(_) => self,
            ParseTree::Query(query) => query.get_last_child(),
            ParseTree::With(with) => with.get_last_child(),
            ParseTree::NamedQuery(named_query) => named_query.get_last_child(),
            ParseTree::QueryNoWith(query_no_with) => query_no_with.get_last_child(),
            ParseTree::OrderBy(order_by) => order_by.get_last_child(),
            ParseTree::Limit(limit) => limit.get_last_child(),
            ParseTree::QuerySetOperation(query_set_operation) => {
                query_set_operation.get_last_child()
            }
            ParseTree::SortItem(sort_item) => sort_item.get_last_child(),
            ParseTree::Subquery(subquery) => subquery.get_last_child(),
            ParseTree::InlineTable(inline_table) => inline_table.get_last_child(),
            ParseTree::Table(table) => table.get_last_child(),
            ParseTree::QuerySpecification(query_specification) => {
                query_specification.get_last_child()
            }
            ParseTree::QualifiedName(qualified_name) => qualified_name.get_last_child(),
            ParseTree::SelectAll(select_all) => select_all.get_last_child(),
            ParseTree::QualifiedSelectAll(qualified_select_all) => {
                qualified_select_all.get_last_child()
            }
            ParseTree::SelectItem(select_item) => select_item.get_last_child(),
            ParseTree::SubqueryRelation(subquery_relation) => subquery_relation.get_last_child(),
            ParseTree::ParenthesizedRelation(parenthesized_relation) => {
                parenthesized_relation.get_last_child()
            }
            ParseTree::TableName(table_name) => table_name.get_last_child(),
            ParseTree::Lateral(lateral) => lateral.get_last_child(),
            ParseTree::Unnest(unnest) => unnest.get_last_child(),
            ParseTree::SampledRelation(sampled_relation) => sampled_relation.get_last_child(),
            ParseTree::AliasedRelation(aliased_relation) => aliased_relation.get_last_child(),
            ParseTree::CrossJoin(cross_join) => cross_join.get_last_child(),
            ParseTree::Join(join) => join.get_last_child(),
            ParseTree::NaturalJoin(natural_join) => natural_join.get_last_child(),
            ParseTree::OuterJoinKind(outer_join_kind) => outer_join_kind.get_last_child(),
            ParseTree::OnJoinCriteria(on_join_criteria) => on_join_criteria.get_last_child(),
            ParseTree::UsingJoinCriteria(using_join_criteria) => {
                using_join_criteria.get_last_child()
            }
            ParseTree::GroupBy(group_by) => group_by.get_last_child(),
            ParseTree::Rollup(rollup) => rollup.get_last_child(),
            ParseTree::Cube(cube) => cube.get_last_child(),
            ParseTree::GroupingSets(grouping_sets) => grouping_sets.get_last_child(),
            ParseTree::BinaryExpression(binary_expression) => binary_expression.get_last_child(),
            ParseTree::UnaryExpression(unary_expression) => unary_expression.get_last_child(),
            ParseTree::QuantifiedComparison(quantified_comparison) => {
                quantified_comparison.get_last_child()
            }
            ParseTree::NullPredicate(null_predicate) => null_predicate.get_last_child(),
            ParseTree::DistinctFrom(distinct_from) => distinct_from.get_last_child(),
            ParseTree::Between(between) => between.get_last_child(),
            ParseTree::Like(like) => like.get_last_child(),
            ParseTree::InSubquery(in_subquery) => in_subquery.get_last_child(),
            ParseTree::InList(in_list) => in_list.get_last_child(),
            ParseTree::AtTimeZone(at_time_zone) => at_time_zone.get_last_child(),
            ParseTree::Dereference(dereference) => dereference.get_last_child(),
            ParseTree::Subscript(subscript) => subscript.get_last_child(),
            ParseTree::Lambda(lambda) => lambda.get_last_child(),
            ParseTree::Literal(literal) => literal.get_last_child(),
            ParseTree::RowConstructor(row_constructor) => row_constructor.get_last_child(),
            ParseTree::ParenthesizedExpression(parenthesized_expression) => {
                parenthesized_expression.get_last_child()
            }
            ParseTree::Identifier(identifier) => identifier.get_last_child(),
            ParseTree::FunctionCall(function_call) => function_call.get_last_child(),
            ParseTree::Filter(filter) => filter.get_last_child(),
            ParseTree::Over(over) => over.get_last_child(),
            ParseTree::WindowFrame(window_frame) => window_frame.get_last_child(),
            ParseTree::UnboundedFrame(unbounded_frame) => unbounded_frame.get_last_child(),
            ParseTree::CurrentRowBound(current_row_bound) => current_row_bound.get_last_child(),
            ParseTree::BoundedFrame(bounded_frame) => bounded_frame.get_last_child(),
            ParseTree::UnicodeString(unicode_string) => unicode_string.get_last_child(),
            ParseTree::ConfigureExpression(configure_expression) => {
                configure_expression.get_last_child()
            }
            ParseTree::SubqueryExpression(subquery_expression) => {
                subquery_expression.get_last_child()
            }
            ParseTree::Grouping(grouping) => grouping.get_last_child(),
            ParseTree::Extract(extract) => extract.get_last_child(),
            ParseTree::CurrentTime(current_time) => current_time.get_last_child(),
            ParseTree::CurrentTimestamp(current_timestamp) => current_timestamp.get_last_child(),
            ParseTree::Normalize(normalize) => normalize.get_last_child(),
            ParseTree::Localtime(localtime) => localtime.get_last_child(),
            ParseTree::Localtimestamp(localtimestamp) => localtimestamp.get_last_child(),
            ParseTree::Cast(cast) => cast.get_last_child(),
            ParseTree::WhenClause(when_clause) => when_clause.get_last_child(),
            ParseTree::Case(case) => case.get_last_child(),
            ParseTree::Exists(exists) => exists.get_last_child(),
            ParseTree::TypeConstructor(type_constructor) => type_constructor.get_last_child(),
            ParseTree::Array(array) => array.get_last_child(),
            ParseTree::Interval(interval) => interval.get_last_child(),
            ParseTree::Row(row) => row.get_last_child(),
            ParseTree::TryCast(try_cast) => try_cast.get_last_child(),
            ParseTree::Substring(substring) => substring.get_last_child(),
            ParseTree::Position(position) => position.get_last_child(),
            ParseTree::ArrayTypeSuffix(array_type_suffix) => array_type_suffix.get_last_child(),
            ParseTree::NamedType(named_type) => named_type.get_last_child(),
            ParseTree::ArrayType(array_type) => array_type.get_last_child(),
            ParseTree::MapType(map_type) => map_type.get_last_child(),
            ParseTree::RowType(row_type) => row_type.get_last_child(),
            ParseTree::RowTypeElement(row_type_element) => row_type_element.get_last_child(),
            ParseTree::IntervalType(interval_type) => interval_type.get_last_child(),
            ParseTree::IfNotExists(if_not_exists) => if_not_exists.get_last_child(),
            ParseTree::CreateTable(create_table) => create_table.get_last_child(),
            ParseTree::CreateView(create_view) => create_view.get_last_child(),
            ParseTree::CreateRole(create_role) => create_role.get_last_child(),
            ParseTree::WithAdminGrantor(with_admin_grantor) => with_admin_grantor.get_last_child(),
            ParseTree::UserPrincipal(user_principal) => user_principal.get_last_child(),
            ParseTree::RolePrincipal(role_principal) => role_principal.get_last_child(),
            ParseTree::UnspecifiedPrincipal(unspecified_principal) => {
                unspecified_principal.get_last_child()
            }
            ParseTree::CreateTableAsSelect(create_table_as_select) => {
                create_table_as_select.get_last_child()
            }
            ParseTree::WithProperties(with_properties) => with_properties.get_last_child(),
            ParseTree::Property(property) => property.get_last_child(),
            ParseTree::WithData(with_data) => with_data.get_last_child(),
            ParseTree::Comment(comment) => comment.get_last_child(),
            ParseTree::ColumnDefinition(column_definition) => column_definition.get_last_child(),
            ParseTree::NotNull(not_null) => not_null.get_last_child(),
            ParseTree::LikeClause(like_clause) => like_clause.get_last_child(),
            ParseTree::InsertInto(insert_into) => insert_into.get_last_child(),
            ParseTree::Delete(delete) => delete.get_last_child(),
            ParseTree::GroupingSet(grouping_set) => grouping_set.get_last_child(),
            ParseTree::RelationOrQuery(relation_or_query) => relation_or_query.get_last_child(),
            ParseTree::EmptyGroupingSet(empty_grouping_set) => empty_grouping_set.get_last_child(),
            ParseTree::ExpressionOrQuery(expression_or_query) => {
                expression_or_query.get_last_child()
            }
            ParseTree::Entrypoint(entrypoint) => entrypoint.get_last_child(),
            ParseTree::NullTreatment(null_treatment) => null_treatment.get_last_child(),
        }
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        match self {
            ParseTree::Token(token) => Some(&token.token),
            ParseTree::List(list) => list.get_first_token(),
            ParseTree::Error(_) => None,
            ParseTree::Empty(_) => None,
            ParseTree::Query(query) => query.get_first_token(),
            ParseTree::With(with) => with.get_first_token(),
            ParseTree::NamedQuery(named_query) => named_query.get_first_token(),
            ParseTree::QueryNoWith(query_no_with) => query_no_with.get_first_token(),
            ParseTree::OrderBy(order_by) => order_by.get_first_token(),
            ParseTree::Limit(limit) => limit.get_first_token(),
            ParseTree::QuerySetOperation(query_set_operation) => {
                query_set_operation.get_first_token()
            }
            ParseTree::SortItem(sort_item) => sort_item.get_first_token(),
            ParseTree::Subquery(subquery) => subquery.get_first_token(),
            ParseTree::InlineTable(inline_table) => inline_table.get_first_token(),
            ParseTree::Table(table) => table.get_first_token(),
            ParseTree::QuerySpecification(query_specification) => {
                query_specification.get_first_token()
            }
            ParseTree::QualifiedName(qualified_name) => qualified_name.get_first_token(),
            ParseTree::SelectAll(select_all) => select_all.get_first_token(),
            ParseTree::QualifiedSelectAll(qualified_select_all) => {
                qualified_select_all.get_first_token()
            }
            ParseTree::SelectItem(select_item) => select_item.get_first_token(),
            ParseTree::SubqueryRelation(subquery_relation) => subquery_relation.get_first_token(),
            ParseTree::ParenthesizedRelation(parenthesized_relation) => {
                parenthesized_relation.get_first_token()
            }
            ParseTree::TableName(table_name) => table_name.get_first_token(),
            ParseTree::Lateral(lateral) => lateral.get_first_token(),
            ParseTree::Unnest(unnest) => unnest.get_first_token(),
            ParseTree::SampledRelation(sampled_relation) => sampled_relation.get_first_token(),
            ParseTree::AliasedRelation(aliased_relation) => aliased_relation.get_first_token(),
            ParseTree::CrossJoin(cross_join) => cross_join.get_first_token(),
            ParseTree::Join(join) => join.get_first_token(),
            ParseTree::NaturalJoin(natural_join) => natural_join.get_first_token(),
            ParseTree::OuterJoinKind(outer_join_kind) => outer_join_kind.get_first_token(),
            ParseTree::OnJoinCriteria(on_join_criteria) => on_join_criteria.get_first_token(),
            ParseTree::UsingJoinCriteria(using_join_criteria) => {
                using_join_criteria.get_first_token()
            }
            ParseTree::GroupBy(group_by) => group_by.get_first_token(),
            ParseTree::Rollup(rollup) => rollup.get_first_token(),
            ParseTree::Cube(cube) => cube.get_first_token(),
            ParseTree::GroupingSets(grouping_sets) => grouping_sets.get_first_token(),
            ParseTree::BinaryExpression(binary_expression) => binary_expression.get_first_token(),
            ParseTree::UnaryExpression(unary_expression) => unary_expression.get_first_token(),
            ParseTree::QuantifiedComparison(quantified_comparison) => {
                quantified_comparison.get_first_token()
            }
            ParseTree::NullPredicate(null_predicate) => null_predicate.get_first_token(),
            ParseTree::DistinctFrom(distinct_from) => distinct_from.get_first_token(),
            ParseTree::Between(between) => between.get_first_token(),
            ParseTree::Like(like) => like.get_first_token(),
            ParseTree::InSubquery(in_subquery) => in_subquery.get_first_token(),
            ParseTree::InList(in_list) => in_list.get_first_token(),
            ParseTree::AtTimeZone(at_time_zone) => at_time_zone.get_first_token(),
            ParseTree::Dereference(dereference) => dereference.get_first_token(),
            ParseTree::Subscript(subscript) => subscript.get_first_token(),
            ParseTree::Lambda(lambda) => lambda.get_first_token(),
            ParseTree::Literal(literal) => literal.get_first_token(),
            ParseTree::RowConstructor(row_constructor) => row_constructor.get_first_token(),
            ParseTree::ParenthesizedExpression(parenthesized_expression) => {
                parenthesized_expression.get_first_token()
            }
            ParseTree::Identifier(identifier) => identifier.get_first_token(),
            ParseTree::FunctionCall(function_call) => function_call.get_first_token(),
            ParseTree::Filter(filter) => filter.get_first_token(),
            ParseTree::Over(over) => over.get_first_token(),
            ParseTree::WindowFrame(window_frame) => window_frame.get_first_token(),
            ParseTree::UnboundedFrame(unbounded_frame) => unbounded_frame.get_first_token(),
            ParseTree::CurrentRowBound(current_row_bound) => current_row_bound.get_first_token(),
            ParseTree::BoundedFrame(bounded_frame) => bounded_frame.get_first_token(),
            ParseTree::UnicodeString(unicode_string) => unicode_string.get_first_token(),
            ParseTree::ConfigureExpression(configure_expression) => {
                configure_expression.get_first_token()
            }
            ParseTree::SubqueryExpression(subquery_expression) => {
                subquery_expression.get_first_token()
            }
            ParseTree::Grouping(grouping) => grouping.get_first_token(),
            ParseTree::Extract(extract) => extract.get_first_token(),
            ParseTree::CurrentTime(current_time) => current_time.get_first_token(),
            ParseTree::CurrentTimestamp(current_timestamp) => current_timestamp.get_first_token(),
            ParseTree::Normalize(normalize) => normalize.get_first_token(),
            ParseTree::Localtime(localtime) => localtime.get_first_token(),
            ParseTree::Localtimestamp(localtimestamp) => localtimestamp.get_first_token(),
            ParseTree::Cast(cast) => cast.get_first_token(),
            ParseTree::WhenClause(when_clause) => when_clause.get_first_token(),
            ParseTree::Case(case) => case.get_first_token(),
            ParseTree::Exists(exists) => exists.get_first_token(),
            ParseTree::TypeConstructor(type_constructor) => type_constructor.get_first_token(),
            ParseTree::Array(array) => array.get_first_token(),
            ParseTree::Interval(interval) => interval.get_first_token(),
            ParseTree::Row(row) => row.get_first_token(),
            ParseTree::TryCast(try_cast) => try_cast.get_first_token(),
            ParseTree::Substring(substring) => substring.get_first_token(),
            ParseTree::Position(position) => position.get_first_token(),
            ParseTree::ArrayTypeSuffix(array_type_suffix) => array_type_suffix.get_first_token(),
            ParseTree::NamedType(named_type) => named_type.get_first_token(),
            ParseTree::ArrayType(array_type) => array_type.get_first_token(),
            ParseTree::MapType(map_type) => map_type.get_first_token(),
            ParseTree::RowType(row_type) => row_type.get_first_token(),
            ParseTree::RowTypeElement(row_type_element) => row_type_element.get_first_token(),
            ParseTree::IntervalType(interval_type) => interval_type.get_first_token(),
            ParseTree::IfNotExists(if_not_exists) => if_not_exists.get_first_token(),
            ParseTree::CreateTable(create_table) => create_table.get_first_token(),
            ParseTree::CreateView(create_view) => create_view.get_first_token(),
            ParseTree::CreateRole(create_role) => create_role.get_first_token(),
            ParseTree::WithAdminGrantor(with_admin_grantor) => with_admin_grantor.get_first_token(),
            ParseTree::UserPrincipal(user_principal) => user_principal.get_first_token(),
            ParseTree::RolePrincipal(role_principal) => role_principal.get_first_token(),
            ParseTree::UnspecifiedPrincipal(unspecified_principal) => {
                unspecified_principal.get_first_token()
            }
            ParseTree::CreateTableAsSelect(create_table_as_select) => {
                create_table_as_select.get_first_token()
            }
            ParseTree::WithProperties(with_properties) => with_properties.get_first_token(),
            ParseTree::Property(property) => property.get_first_token(),
            ParseTree::WithData(with_data) => with_data.get_first_token(),
            ParseTree::Comment(comment) => comment.get_first_token(),
            ParseTree::ColumnDefinition(column_definition) => column_definition.get_first_token(),
            ParseTree::NotNull(not_null) => not_null.get_first_token(),
            ParseTree::LikeClause(like_clause) => like_clause.get_first_token(),
            ParseTree::InsertInto(insert_into) => insert_into.get_first_token(),
            ParseTree::Delete(delete) => delete.get_first_token(),
            ParseTree::GroupingSet(grouping_set) => grouping_set.get_first_token(),
            ParseTree::RelationOrQuery(relation_or_query) => relation_or_query.get_first_token(),
            ParseTree::EmptyGroupingSet(empty_grouping_set) => empty_grouping_set.get_first_token(),
            ParseTree::ExpressionOrQuery(expression_or_query) => {
                expression_or_query.get_first_token()
            }
            ParseTree::Entrypoint(entrypoint) => entrypoint.get_first_token(),
            ParseTree::NullTreatment(null_treatment) => null_treatment.get_first_token(),
        }
    }

    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        match self {
            ParseTree::Token(token) => Some(&token.token),
            ParseTree::List(list) => list.get_last_token(),
            ParseTree::Error(_) => None,
            ParseTree::Empty(_) => None,
            ParseTree::NullTreatment(null_treatment) => null_treatment.get_last_token(),
            ParseTree::Entrypoint(entrypoint) => entrypoint.get_last_token(),
            ParseTree::ExpressionOrQuery(expression_or_query) => {
                expression_or_query.get_last_token()
            }
            ParseTree::EmptyGroupingSet(empty_grouping_set) => empty_grouping_set.get_last_token(),
            ParseTree::RelationOrQuery(relation_or_query) => relation_or_query.get_last_token(),
            ParseTree::GroupingSet(grouping_set) => grouping_set.get_last_token(),
            ParseTree::Delete(delete) => delete.get_last_token(),
            ParseTree::InsertInto(insert_into) => insert_into.get_last_token(),
            ParseTree::LikeClause(like_clause) => like_clause.get_last_token(),
            ParseTree::NotNull(not_null) => not_null.get_last_token(),
            ParseTree::ColumnDefinition(column_definition) => column_definition.get_last_token(),
            ParseTree::Comment(comment) => comment.get_last_token(),
            ParseTree::WithData(with_data) => with_data.get_last_token(),
            ParseTree::Property(property) => property.get_last_token(),
            ParseTree::WithProperties(with_properties) => with_properties.get_last_token(),
            ParseTree::CreateTableAsSelect(create_table_as_select) => {
                create_table_as_select.get_last_token()
            }
            ParseTree::UnspecifiedPrincipal(unspecified_principal) => {
                unspecified_principal.get_last_token()
            }
            ParseTree::RolePrincipal(role_principal) => role_principal.get_last_token(),
            ParseTree::UserPrincipal(user_principal) => user_principal.get_last_token(),
            ParseTree::WithAdminGrantor(with_admin_grantor) => with_admin_grantor.get_last_token(),
            ParseTree::CreateRole(create_role) => create_role.get_last_token(),
            ParseTree::CreateView(create_view) => create_view.get_last_token(),
            ParseTree::CreateTable(create_table) => create_table.get_last_token(),
            ParseTree::IfNotExists(if_not_exists) => if_not_exists.get_last_token(),
            ParseTree::IntervalType(interval_type) => interval_type.get_last_token(),
            ParseTree::RowTypeElement(row_type_element) => row_type_element.get_last_token(),
            ParseTree::RowType(row_type) => row_type.get_last_token(),
            ParseTree::MapType(map_type) => map_type.get_last_token(),
            ParseTree::ArrayType(array_type) => array_type.get_last_token(),
            ParseTree::NamedType(named_type) => named_type.get_last_token(),
            ParseTree::ArrayTypeSuffix(array_type_suffix) => array_type_suffix.get_last_token(),
            ParseTree::Position(position) => position.get_last_token(),
            ParseTree::Substring(substring) => substring.get_last_token(),
            ParseTree::TryCast(try_cast) => try_cast.get_last_token(),
            ParseTree::Row(row) => row.get_last_token(),
            ParseTree::Interval(interval) => interval.get_last_token(),
            ParseTree::Array(array) => array.get_last_token(),
            ParseTree::TypeConstructor(type_constructor) => type_constructor.get_last_token(),
            ParseTree::Exists(exists) => exists.get_last_token(),
            ParseTree::Case(case) => case.get_last_token(),
            ParseTree::WhenClause(when_clause) => when_clause.get_last_token(),
            ParseTree::Cast(cast) => cast.get_last_token(),
            ParseTree::Localtimestamp(localtimestamp) => localtimestamp.get_last_token(),
            ParseTree::Localtime(localtime) => localtime.get_last_token(),
            ParseTree::Normalize(normalize) => normalize.get_last_token(),
            ParseTree::CurrentTimestamp(current_timestamp) => current_timestamp.get_last_token(),
            ParseTree::CurrentTime(current_time) => current_time.get_last_token(),
            ParseTree::Extract(extract) => extract.get_last_token(),
            ParseTree::Grouping(grouping) => grouping.get_last_token(),
            ParseTree::SubqueryExpression(subquery_expression) => {
                subquery_expression.get_last_token()
            }
            ParseTree::ConfigureExpression(configure_expression) => {
                configure_expression.get_last_token()
            }
            ParseTree::UnicodeString(unicode_string) => unicode_string.get_last_token(),
            ParseTree::BoundedFrame(bounded_frame) => bounded_frame.get_last_token(),
            ParseTree::CurrentRowBound(current_row_bound) => current_row_bound.get_last_token(),
            ParseTree::UnboundedFrame(unbounded_frame) => unbounded_frame.get_last_token(),
            ParseTree::WindowFrame(window_frame) => window_frame.get_last_token(),
            ParseTree::Over(over) => over.get_last_token(),
            ParseTree::Filter(filter) => filter.get_last_token(),
            ParseTree::FunctionCall(function_call) => function_call.get_last_token(),
            ParseTree::Identifier(identifier) => identifier.get_last_token(),
            ParseTree::ParenthesizedExpression(parenthesized_expression) => {
                parenthesized_expression.get_last_token()
            }
            ParseTree::RowConstructor(row_constructor) => row_constructor.get_last_token(),
            ParseTree::Literal(literal) => literal.get_last_token(),
            ParseTree::Lambda(lambda) => lambda.get_last_token(),
            ParseTree::Subscript(subscript) => subscript.get_last_token(),
            ParseTree::Dereference(dereference) => dereference.get_last_token(),
            ParseTree::AtTimeZone(at_time_zone) => at_time_zone.get_last_token(),
            ParseTree::InList(in_list) => in_list.get_last_token(),
            ParseTree::InSubquery(in_subquery) => in_subquery.get_last_token(),
            ParseTree::Like(like) => like.get_last_token(),
            ParseTree::Between(between) => between.get_last_token(),
            ParseTree::DistinctFrom(distinct_from) => distinct_from.get_last_token(),
            ParseTree::NullPredicate(null_predicate) => null_predicate.get_last_token(),
            ParseTree::QuantifiedComparison(quantified_comparison) => {
                quantified_comparison.get_last_token()
            }
            ParseTree::UnaryExpression(unary_expression) => unary_expression.get_last_token(),
            ParseTree::BinaryExpression(binary_expression) => binary_expression.get_last_token(),
            ParseTree::GroupingSets(grouping_sets) => grouping_sets.get_last_token(),
            ParseTree::Cube(cube) => cube.get_last_token(),
            ParseTree::Rollup(rollup) => rollup.get_last_token(),
            ParseTree::GroupBy(group_by) => group_by.get_last_token(),
            ParseTree::UsingJoinCriteria(using_join_criteria) => {
                using_join_criteria.get_last_token()
            }
            ParseTree::OnJoinCriteria(on_join_criteria) => on_join_criteria.get_last_token(),
            ParseTree::OuterJoinKind(outer_join_kind) => outer_join_kind.get_last_token(),
            ParseTree::NaturalJoin(natural_join) => natural_join.get_last_token(),
            ParseTree::Join(join) => join.get_last_token(),
            ParseTree::CrossJoin(cross_join) => cross_join.get_last_token(),
            ParseTree::AliasedRelation(aliased_relation) => aliased_relation.get_last_token(),
            ParseTree::SampledRelation(sampled_relation) => sampled_relation.get_last_token(),
            ParseTree::Unnest(unnest) => unnest.get_last_token(),
            ParseTree::Lateral(lateral) => lateral.get_last_token(),
            ParseTree::TableName(table_name) => table_name.get_last_token(),
            ParseTree::ParenthesizedRelation(parenthesized_relation) => {
                parenthesized_relation.get_last_token()
            }
            ParseTree::SubqueryRelation(subquery_relation) => subquery_relation.get_last_token(),
            ParseTree::SelectItem(select_item) => select_item.get_last_token(),
            ParseTree::QualifiedSelectAll(qualified_select_all) => {
                qualified_select_all.get_last_token()
            }
            ParseTree::SelectAll(select_all) => select_all.get_last_token(),
            ParseTree::QualifiedName(qualified_name) => qualified_name.get_last_token(),
            ParseTree::QuerySpecification(query_specification) => {
                query_specification.get_last_token()
            }
            ParseTree::Table(table) => table.get_last_token(),
            ParseTree::InlineTable(inline_table) => inline_table.get_last_token(),
            ParseTree::Subquery(subquery) => subquery.get_last_token(),
            ParseTree::SortItem(sort_item) => sort_item.get_last_token(),
            ParseTree::QuerySetOperation(query_set_operation) => {
                query_set_operation.get_last_token()
            }
            ParseTree::Limit(limit) => limit.get_last_token(),
            ParseTree::OrderBy(order_by) => order_by.get_last_token(),
            ParseTree::QueryNoWith(query_no_with) => query_no_with.get_last_token(),
            ParseTree::NamedQuery(named_query) => named_query.get_last_token(),
            ParseTree::With(with) => with.get_last_token(),
            ParseTree::Query(query) => query.get_last_token(),
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.with);
        result.push(&*self.query_no_with);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.with, *self.query_no_with)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.with
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.query_no_with
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.with.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query_no_with.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.query_no_with.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.with.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.with);
        result.push(&*self.recursive);
        result.push(&*self.named_queries);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.with, *self.recursive, *self.named_queries)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.with
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.named_queries
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.with.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.recursive.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.named_queries.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.named_queries.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.recursive.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.with.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.name);
        result.push(&*self.column_aliases_opt);
        result.push(&*self.as_);
        result.push(&*self.open_paren);
        result.push(&*self.query);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.name
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.name.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.column_aliases_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.as_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.as_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.column_aliases_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.name.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.query_term);
        result.push(&*self.order_by_opt);
        result.push(&*self.limit_opt);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.query_term, *self.order_by_opt, *self.limit_opt)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.query_term
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.limit_opt
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.query_term.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.order_by_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.limit_opt.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.limit_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.order_by_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.query_term.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.order);
        result.push(&*self.by);
        result.push(&*self.sort_items);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.order, *self.by, *self.sort_items)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.order
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.sort_items
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.order.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.by.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.sort_items.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.sort_items.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.by.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.order.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.limit);
        result.push(&*self.value);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.limit, *self.value)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.limit
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.limit.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.limit.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.left);
        result.push(&*self.operator);
        result.push(&*self.set_quantifier_opt);
        result.push(&*self.right);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.left,
            *self.operator,
            *self.set_quantifier_opt,
            *self.right,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.left
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.right
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.left.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.operator.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.set_quantifier_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.right.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.right.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.set_quantifier_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.operator.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.left.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.expression);
        result.push(&*self.ordering_opt);
        result.push(&*self.nulls);
        result.push(&*self.null_ordering_opt);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.expression,
            *self.ordering_opt,
            *self.nulls,
            *self.null_ordering_opt,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.expression
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.null_ordering_opt
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.expression.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.ordering_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.nulls.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.null_ordering_opt.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.null_ordering_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.nulls.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.ordering_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.expression.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.open_paren);
        result.push(&*self.query_no_with);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.open_paren, *self.query_no_with, *self.close_paren)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.open_paren
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query_no_with.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.query_no_with.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.values);
        result.push(&*self.expressions);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.values, *self.expressions)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.values
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.expressions
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.values.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.expressions.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.expressions.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.values.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.table);
        result.push(&*self.qualified_name);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.table, *self.qualified_name)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.table
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.qualified_name
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.table.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.qualified_name.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.qualified_name.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.table.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(12);
        result.push(&*self.select);
        result.push(&*self.set_quantifier_opt);
        result.push(&*self.select_items);
        result.push(&*self.from);
        result.push(&*self.relations);
        result.push(&*self.where_);
        result.push(&*self.where_predicate);
        result.push(&*self.group);
        result.push(&*self.by);
        result.push(&*self.group_by);
        result.push(&*self.having);
        result.push(&*self.having_predicate);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.select
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.having_predicate
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.select.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.set_quantifier_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.select_items.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.relations.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.where_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.where_predicate.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.group.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.by.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.group_by.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.having.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.having_predicate.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.having_predicate.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.having.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.group_by.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.by.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.group.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.where_predicate.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.where_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.relations.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.select_items.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.set_quantifier_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.select.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(1);
        result.push(&*self.names);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.names,)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.names
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.names
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.names.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.names.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(1);
        result.push(&*self.asterisk);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.asterisk,)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.asterisk
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.asterisk
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.asterisk.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.asterisk.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.qualifier);
        result.push(&*self.period);
        result.push(&*self.asterisk);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.qualifier, *self.period, *self.asterisk)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.qualifier
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.asterisk
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.qualifier.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.period.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.asterisk.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.asterisk.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.period.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.qualifier.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.expression);
        result.push(&*self.as_);
        result.push(&*self.identifier);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.expression, *self.as_, *self.identifier)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.expression
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.identifier
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.expression.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.as_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.identifier.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.identifier.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.as_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.expression.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.open_paren);
        result.push(&*self.query);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.open_paren, *self.query, *self.close_paren)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.open_paren
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.open_paren);
        result.push(&*self.relation);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.open_paren, *self.relation, *self.close_paren)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.open_paren
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.relation.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.relation.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(1);
        result.push(&*self.name);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.name,)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.name
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.name
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.name.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.name.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.lateral);
        result.push(&*self.open_paren);
        result.push(&*self.query);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.lateral,
            *self.open_paren,
            *self.query,
            *self.close_paren,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.lateral
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.lateral.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.lateral.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.unnest);
        result.push(&*self.expressions);
        result.push(&*self.with);
        result.push(&*self.ordinality);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.unnest,
            *self.expressions,
            *self.with,
            *self.ordinality,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.unnest
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.ordinality
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.unnest.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.expressions.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.with.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.ordinality.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.ordinality.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.with.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.expressions.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.unnest.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.aliased_relation);
        result.push(&*self.tablesample);
        result.push(&*self.sample_type);
        result.push(&*self.open_paren);
        result.push(&*self.expression);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.aliased_relation
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.aliased_relation.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.tablesample.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.sample_type.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.expression.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.expression.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.sample_type.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.tablesample.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.aliased_relation.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.relation_primary);
        result.push(&*self.as_opt);
        result.push(&*self.identifier);
        result.push(&*self.column_aliases_opt);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.relation_primary,
            *self.as_opt,
            *self.identifier,
            *self.column_aliases_opt,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.relation_primary
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.column_aliases_opt
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.relation_primary.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.as_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.identifier.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.column_aliases_opt.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.column_aliases_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.identifier.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.as_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.relation_primary.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.left);
        result.push(&*self.cross);
        result.push(&*self.join);
        result.push(&*self.right);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.left, *self.cross, *self.join, *self.right)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.left
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.right
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.left.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.cross.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.join.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.right.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.right.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.join.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.cross.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.left.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(5);
        result.push(&*self.left);
        result.push(&*self.join_type);
        result.push(&*self.join);
        result.push(&*self.right);
        result.push(&*self.join_criteria);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.left
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.join_criteria
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.left.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.join_type.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.join.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.right.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.join_criteria.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.join_criteria.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.right.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.join.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.join_type.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.left.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(5);
        result.push(&*self.left);
        result.push(&*self.natural);
        result.push(&*self.join_type);
        result.push(&*self.join);
        result.push(&*self.right);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.left
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.right
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.left.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.natural.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.join_type.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.join.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.right.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.right.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.join.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.join_type.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.natural.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.left.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.kind);
        result.push(&*self.outer_opt);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.kind, *self.outer_opt)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.kind
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.outer_opt
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.kind.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.outer_opt.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.outer_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.kind.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.on);
        result.push(&*self.predicate);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.on, *self.predicate)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.on
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.predicate
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.on.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.predicate.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.predicate.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.on.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.using);
        result.push(&*self.names);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.using, *self.names)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.using
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.names
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.using.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.names.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.names.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.using.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.set_quantifier_opt);
        result.push(&*self.grouping_elements);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.set_quantifier_opt, *self.grouping_elements)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.set_quantifier_opt
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.grouping_elements
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.set_quantifier_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.grouping_elements.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.grouping_elements.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.set_quantifier_opt.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.rollup);
        result.push(&*self.expressions);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.rollup, *self.expressions)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.rollup
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.expressions
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.rollup.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.expressions.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.expressions.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.rollup.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.cube);
        result.push(&*self.expressions);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.cube, *self.expressions)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.cube
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.expressions
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.cube.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.expressions.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.expressions.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.cube.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.grouping);
        result.push(&*self.sets);
        result.push(&*self.grouping_sets);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.grouping, *self.sets, *self.grouping_sets)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.grouping
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.grouping_sets
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.grouping.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.sets.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.grouping_sets.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.grouping_sets.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.sets.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.grouping.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.left);
        result.push(&*self.operator);
        result.push(&*self.right);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.left, *self.operator, *self.right)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.left
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.right
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.left.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.operator.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.right.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.right.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.operator.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.left.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.operator);
        result.push(&*self.operand);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.operator, *self.operand)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.operator
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.operand
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.operator.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.operand.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.operand.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.operator.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.operand);
        result.push(&*self.operator);
        result.push(&*self.comparison_quantifier);
        result.push(&*self.open_paren);
        result.push(&*self.query);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.operand
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.operand.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.operator.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.comparison_quantifier.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.comparison_quantifier.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.operator.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.operand.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.value);
        result.push(&*self.is);
        result.push(&*self.not_opt);
        result.push(&*self.null);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.value, *self.is, *self.not_opt, *self.null)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.null
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.is.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.not_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.null.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.null.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.not_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.is.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.left);
        result.push(&*self.distinct);
        result.push(&*self.from);
        result.push(&*self.right);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.left, *self.distinct, *self.from, *self.right)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.left
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.right
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.left.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.distinct.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.right.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.right.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.distinct.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.left.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.value);
        result.push(&*self.not_opt);
        result.push(&*self.between);
        result.push(&*self.lower);
        result.push(&*self.and);
        result.push(&*self.upper);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.upper
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.not_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.between.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.lower.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.and.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.upper.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.upper.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.and.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.lower.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.between.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.not_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.value);
        result.push(&*self.not_opt);
        result.push(&*self.like);
        result.push(&*self.patrern);
        result.push(&*self.escape_opt);
        result.push(&*self.escape_value_opt);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.escape_value_opt
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.not_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.like.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.patrern.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.escape_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.escape_value_opt.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.escape_value_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.escape_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.patrern.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.like.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.not_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.value);
        result.push(&*self.not_opt);
        result.push(&*self.in_);
        result.push(&*self.open_paren);
        result.push(&*self.query);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.not_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.in_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.in_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.not_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.value);
        result.push(&*self.not_opt);
        result.push(&*self.in_);
        result.push(&*self.expressions);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.value, *self.not_opt, *self.in_, *self.expressions)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.expressions
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.not_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.in_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.expressions.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.expressions.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.in_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.not_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(5);
        result.push(&*self.value);
        result.push(&*self.at);
        result.push(&*self.time);
        result.push(&*self.zone);
        result.push(&*self.specifier);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.specifier
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.at.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.time.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.zone.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.specifier.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.specifier.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.zone.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.time.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.at.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.object);
        result.push(&*self.period);
        result.push(&*self.field_name);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.object, *self.period, *self.field_name)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.object
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.field_name
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.object.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.period.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.field_name.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.field_name.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.period.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.object.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.operand);
        result.push(&*self.open_square);
        result.push(&*self.index);
        result.push(&*self.close_square);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.operand,
            *self.open_square,
            *self.index,
            *self.close_square,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.operand
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_square
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.operand.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_square.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.index.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_square.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_square.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.index.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_square.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.operand.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.parameters);
        result.push(&*self.array);
        result.push(&*self.body);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.parameters, *self.array, *self.body)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.parameters
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.body
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.parameters.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.array.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.body.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.body.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.array.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.parameters.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(1);
        result.push(&*self.value);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.value,)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(1);
        result.push(&*self.elements);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.elements,)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.elements
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.elements
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.elements.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.elements.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.open_paren);
        result.push(&*self.value);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.open_paren, *self.value, *self.close_paren)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.open_paren
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(1);
        result.push(&*self.value);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.value,)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        None
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
    pub null_treatment_opt: Box<ParseTree<'a>>,
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
    null_treatment_opt: ParseTree<'a>,
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
        null_treatment_opt: Box::new(null_treatment_opt),
        over_opt: Box::new(over_opt),
    })
}

impl<'a> FunctionCall<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::FunctionCall(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(9);
        result.push(&*self.name);
        result.push(&*self.open_paren);
        result.push(&*self.set_quantifier_opt);
        result.push(&*self.arguments);
        result.push(&*self.order_by_opt);
        result.push(&*self.close_paren);
        result.push(&*self.filter_opt);
        result.push(&*self.null_treatment_opt);
        result.push(&*self.over_opt);
        result
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
    ) {
        (
            *self.name,
            *self.open_paren,
            *self.set_quantifier_opt,
            *self.arguments,
            *self.order_by_opt,
            *self.close_paren,
            *self.filter_opt,
            *self.null_treatment_opt,
            *self.over_opt,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.name
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.over_opt
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.name.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.set_quantifier_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.arguments.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.order_by_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.filter_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.null_treatment_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.over_opt.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.over_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.null_treatment_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.filter_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.order_by_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.arguments.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.set_quantifier_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.name.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(5);
        result.push(&*self.filter);
        result.push(&*self.open_paren);
        result.push(&*self.where_);
        result.push(&*self.predicate);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.filter
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.filter.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.where_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.predicate.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.predicate.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.where_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.filter.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(8);
        result.push(&*self.over);
        result.push(&*self.open_paren);
        result.push(&*self.partition_opt);
        result.push(&*self.by);
        result.push(&*self.partitions);
        result.push(&*self.order_by_opt);
        result.push(&*self.window_frame);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.over
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.over.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.partition_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.by.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.partitions.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.order_by_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.window_frame.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.window_frame.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.order_by_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.partitions.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.by.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.partition_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.over.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(5);
        result.push(&*self.frame_type);
        result.push(&*self.between_opt);
        result.push(&*self.start);
        result.push(&*self.and);
        result.push(&*self.end);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.frame_type
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.end
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.frame_type.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.between_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.start.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.and.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.end.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.end.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.and.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.start.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.between_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.frame_type.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.unbounded);
        result.push(&*self.bound_type);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.unbounded, *self.bound_type)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.unbounded
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.bound_type
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.unbounded.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.bound_type.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.bound_type.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.unbounded.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.current);
        result.push(&*self.row);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.current, *self.row)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.current
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.row
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.current.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.row.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.row.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.current.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.bound);
        result.push(&*self.bound_type);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.bound, *self.bound_type)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.bound
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.bound_type
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.bound.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.bound_type.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.bound_type.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.bound.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.string);
        result.push(&*self.uescape_opt);
        result.push(&*self.escape);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.string, *self.uescape_opt, *self.escape)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.string
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.escape
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.string.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.uescape_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.escape.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.escape.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.uescape_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.string.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.configure);
        result.push(&*self.open_paren);
        result.push(&*self.identifier);
        result.push(&*self.comma);
        result.push(&*self.value);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.configure
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.configure.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.identifier.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.comma.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.comma.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.identifier.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.configure.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.open_paren);
        result.push(&*self.query);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.open_paren, *self.query, *self.close_paren)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.open_paren
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.grouping);
        result.push(&*self.groups);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.grouping, *self.groups)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.grouping
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.groups
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.grouping.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.groups.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.groups.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.grouping.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.extract);
        result.push(&*self.open_paren);
        result.push(&*self.identifier);
        result.push(&*self.from);
        result.push(&*self.value);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.extract
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.extract.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.identifier.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.identifier.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.extract.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.current_time);
        result.push(&*self.open_paren);
        result.push(&*self.precision);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.current_time,
            *self.open_paren,
            *self.precision,
            *self.close_paren,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.current_time
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.current_time.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.precision.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.precision.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.current_time.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.current_timestamp);
        result.push(&*self.open_paren);
        result.push(&*self.precision);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.current_timestamp,
            *self.open_paren,
            *self.precision,
            *self.close_paren,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.current_timestamp
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.current_timestamp.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.precision.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.precision.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.current_timestamp.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.normalize);
        result.push(&*self.open_paren);
        result.push(&*self.value);
        result.push(&*self.comma_opt);
        result.push(&*self.normal_form);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.normalize
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.normalize.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.comma_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.normal_form.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.normal_form.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.comma_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.normalize.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.localtime);
        result.push(&*self.open_paren);
        result.push(&*self.precision);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.localtime,
            *self.open_paren,
            *self.precision,
            *self.close_paren,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.localtime
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.localtime.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.precision.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.precision.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.localtime.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.localtimestamp);
        result.push(&*self.open_paren);
        result.push(&*self.precision);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.localtimestamp,
            *self.open_paren,
            *self.precision,
            *self.close_paren,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.localtimestamp
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.localtimestamp.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.precision.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.precision.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.localtimestamp.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.cast);
        result.push(&*self.open_paren);
        result.push(&*self.value);
        result.push(&*self.as_);
        result.push(&*self.type_);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.cast
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.cast.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.as_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.type_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.type_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.as_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.cast.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.when);
        result.push(&*self.condition);
        result.push(&*self.then);
        result.push(&*self.result);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.when, *self.condition, *self.then, *self.result)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.when
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.result
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.when.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.condition.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.then.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.result.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.result.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.then.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.condition.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.when.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.case);
        result.push(&*self.value_opt);
        result.push(&*self.when_clauses);
        result.push(&*self.else_opt);
        result.push(&*self.default);
        result.push(&*self.end);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.case
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.end
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.case.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.when_clauses.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.else_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.default.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.end.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.end.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.default.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.else_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.when_clauses.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.case.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.exists);
        result.push(&*self.open_paren);
        result.push(&*self.query);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.exists,
            *self.open_paren,
            *self.query,
            *self.close_paren,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.exists
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.exists.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.exists.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.type_);
        result.push(&*self.value);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.type_, *self.value)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.type_
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.type_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.type_.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.array);
        result.push(&*self.elements);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.array, *self.elements)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.array
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.elements
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.array.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.elements.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.elements.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.array.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.interval);
        result.push(&*self.sign_opt);
        result.push(&*self.value);
        result.push(&*self.from);
        result.push(&*self.to_kw_opt);
        result.push(&*self.to);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.interval
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.to
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.interval.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.sign_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.to_kw_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.to.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.to.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.to_kw_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.sign_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.interval.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.row);
        result.push(&*self.elements);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.row, *self.elements)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.row
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.elements
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.row.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.elements.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.elements.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.row.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.try_cast);
        result.push(&*self.open_paren);
        result.push(&*self.value);
        result.push(&*self.as_);
        result.push(&*self.type_);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.try_cast
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.try_cast.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.as_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.type_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.type_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.as_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.try_cast.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(8);
        result.push(&*self.substring);
        result.push(&*self.open_paren);
        result.push(&*self.value);
        result.push(&*self.from);
        result.push(&*self.from_value);
        result.push(&*self.for_opt);
        result.push(&*self.for_value);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.substring
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.substring.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.from_value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.for_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.for_value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.for_value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.for_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.from_value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.substring.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.position);
        result.push(&*self.open_paren);
        result.push(&*self.value);
        result.push(&*self.in_);
        result.push(&*self.target);
        result.push(&*self.close_paren);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.position
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.position.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.in_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.target.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.target.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.in_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.position.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.base_type);
        result.push(&*self.array);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.base_type, *self.array)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.base_type
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.array
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.base_type.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.array.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.array.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.base_type.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.name);
        result.push(&*self.type_parameters);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.name, *self.type_parameters)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.name
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.type_parameters
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.name.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.type_parameters.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.type_parameters.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.name.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.array);
        result.push(&*self.open_angle);
        result.push(&*self.element_type);
        result.push(&*self.close_angle);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.array,
            *self.open_angle,
            *self.element_type,
            *self.close_angle,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.array
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_angle
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.array.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_angle.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.element_type.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_angle.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_angle.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.element_type.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_angle.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.array.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(6);
        result.push(&*self.map);
        result.push(&*self.open_angle);
        result.push(&*self.key_type);
        result.push(&*self.comma);
        result.push(&*self.value_type);
        result.push(&*self.close_angle);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.map
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_angle
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.map.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_angle.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.key_type.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.comma.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value_type.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_angle.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_angle.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.value_type.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.comma.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.key_type.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_angle.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.map.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.row);
        result.push(&*self.element_types);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.row, *self.element_types)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.row
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.element_types
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.row.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.element_types.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.element_types.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.row.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.identifier);
        result.push(&*self.type_);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.identifier, *self.type_)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.identifier
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.type_
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.identifier.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.type_.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.type_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.identifier.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.interval);
        result.push(&*self.from);
        result.push(&*self.to_kw);
        result.push(&*self.to);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.interval, *self.from, *self.to_kw, *self.to)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.interval
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.to
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.interval.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.to_kw.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.to.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.to.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.to_kw.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.interval.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.if_);
        result.push(&*self.not);
        result.push(&*self.exists);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.if_, *self.not, *self.exists)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.if_
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.exists
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.if_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.not.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.exists.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.exists.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.not.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.if_.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(7);
        result.push(&*self.create);
        result.push(&*self.table);
        result.push(&*self.if_not_exists_opt);
        result.push(&*self.table_name);
        result.push(&*self.table_elements);
        result.push(&*self.comment_opt);
        result.push(&*self.with_properties_opt);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.create
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.with_properties_opt
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.create.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.table.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.if_not_exists_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.table_name.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.table_elements.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.comment_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.with_properties_opt.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.with_properties_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.comment_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.table_elements.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.table_name.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.if_not_exists_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.table.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.create.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct CreateView<'a> {
    pub create: Box<ParseTree<'a>>,
    pub or_opt: Box<ParseTree<'a>>,
    pub replace_opt: Box<ParseTree<'a>>,
    pub view: Box<ParseTree<'a>>,
    pub qualified_name: Box<ParseTree<'a>>,
    pub as_: Box<ParseTree<'a>>,
    pub query: Box<ParseTree<'a>>,
}

pub fn create_view<'a>(
    create: ParseTree<'a>,
    or_opt: ParseTree<'a>,
    replace_opt: ParseTree<'a>,
    view: ParseTree<'a>,
    qualified_name: ParseTree<'a>,
    as_: ParseTree<'a>,
    query: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::CreateView(CreateView {
        create: Box::new(create),
        or_opt: Box::new(or_opt),
        replace_opt: Box::new(replace_opt),
        view: Box::new(view),
        qualified_name: Box::new(qualified_name),
        as_: Box::new(as_),
        query: Box::new(query),
    })
}

impl<'a> CreateView<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::CreateView(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(7);
        result.push(&*self.create);
        result.push(&*self.or_opt);
        result.push(&*self.replace_opt);
        result.push(&*self.view);
        result.push(&*self.qualified_name);
        result.push(&*self.as_);
        result.push(&*self.query);
        result
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
            *self.or_opt,
            *self.replace_opt,
            *self.view,
            *self.qualified_name,
            *self.as_,
            *self.query,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.create
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.query
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.create.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.or_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.replace_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.view.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.qualified_name.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.as_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.query.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.as_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.qualified_name.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.view.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.replace_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.or_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.create.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct CreateRole<'a> {
    pub create: Box<ParseTree<'a>>,
    pub role: Box<ParseTree<'a>>,
    pub name: Box<ParseTree<'a>>,
    pub with_admin_grantor_opt: Box<ParseTree<'a>>,
}

pub fn create_role<'a>(
    create: ParseTree<'a>,
    role: ParseTree<'a>,
    name: ParseTree<'a>,
    with_admin_grantor_opt: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::CreateRole(CreateRole {
        create: Box::new(create),
        role: Box::new(role),
        name: Box::new(name),
        with_admin_grantor_opt: Box::new(with_admin_grantor_opt),
    })
}

impl<'a> CreateRole<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::CreateRole(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.create);
        result.push(&*self.role);
        result.push(&*self.name);
        result.push(&*self.with_admin_grantor_opt);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.create,
            *self.role,
            *self.name,
            *self.with_admin_grantor_opt,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.create
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.with_admin_grantor_opt
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.create.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.role.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.name.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.with_admin_grantor_opt.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.with_admin_grantor_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.name.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.role.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.create.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct WithAdminGrantor<'a> {
    pub with: Box<ParseTree<'a>>,
    pub admin: Box<ParseTree<'a>>,
    pub grantor: Box<ParseTree<'a>>,
}

pub fn with_admin_grantor<'a>(
    with: ParseTree<'a>,
    admin: ParseTree<'a>,
    grantor: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::WithAdminGrantor(WithAdminGrantor {
        with: Box::new(with),
        admin: Box::new(admin),
        grantor: Box::new(grantor),
    })
}

impl<'a> WithAdminGrantor<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::WithAdminGrantor(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.with);
        result.push(&*self.admin);
        result.push(&*self.grantor);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.with, *self.admin, *self.grantor)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.with
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.grantor
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.with.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.admin.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.grantor.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.grantor.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.admin.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.with.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct UserPrincipal<'a> {
    pub user: Box<ParseTree<'a>>,
    pub identifier: Box<ParseTree<'a>>,
}

pub fn user_principal<'a>(user: ParseTree<'a>, identifier: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::UserPrincipal(UserPrincipal {
        user: Box::new(user),
        identifier: Box::new(identifier),
    })
}

impl<'a> UserPrincipal<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::UserPrincipal(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.user);
        result.push(&*self.identifier);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.user, *self.identifier)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.user
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.identifier
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.user.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.identifier.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.identifier.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.user.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct RolePrincipal<'a> {
    pub role: Box<ParseTree<'a>>,
    pub identifier: Box<ParseTree<'a>>,
}

pub fn role_principal<'a>(role: ParseTree<'a>, identifier: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::RolePrincipal(RolePrincipal {
        role: Box::new(role),
        identifier: Box::new(identifier),
    })
}

impl<'a> RolePrincipal<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::RolePrincipal(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.role);
        result.push(&*self.identifier);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.role, *self.identifier)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.role
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.identifier
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.role.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.identifier.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.identifier.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.role.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct UnspecifiedPrincipal<'a> {
    pub identifier: Box<ParseTree<'a>>,
}

pub fn unspecified_principal<'a>(identifier: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::UnspecifiedPrincipal(UnspecifiedPrincipal {
        identifier: Box::new(identifier),
    })
}

impl<'a> UnspecifiedPrincipal<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::UnspecifiedPrincipal(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(1);
        result.push(&*self.identifier);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.identifier,)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.identifier
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.identifier
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.identifier.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.identifier.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(12);
        result.push(&*self.create);
        result.push(&*self.table);
        result.push(&*self.if_not_exists_opt);
        result.push(&*self.table_name);
        result.push(&*self.column_aliases_opt);
        result.push(&*self.comment_opt);
        result.push(&*self.with_properties_opt);
        result.push(&*self.as_);
        result.push(&*self.open_paren_opt);
        result.push(&*self.query);
        result.push(&*self.close_paren_opt);
        result.push(&*self.with_data_opt);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.create
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.with_data_opt
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.create.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.table.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.if_not_exists_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.table_name.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.column_aliases_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.comment_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.with_properties_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.as_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.with_data_opt.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.with_data_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.as_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.with_properties_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.comment_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.column_aliases_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.table_name.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.if_not_exists_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.table.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.create.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.with);
        result.push(&*self.properties);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.with, *self.properties)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.with
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.properties
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.with.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.properties.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.properties.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.with.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.identifier);
        result.push(&*self.eq);
        result.push(&*self.value);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.identifier, *self.eq, *self.value)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.identifier
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.identifier.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.eq.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.eq.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.identifier.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.with);
        result.push(&*self.no_opt);
        result.push(&*self.data);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.with, *self.no_opt, *self.data)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.with
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.data
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.with.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.no_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.data.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.data.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.no_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.with.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.comment);
        result.push(&*self.value);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.comment, *self.value)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.comment
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.value
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.comment.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.value.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.value.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.comment.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(5);
        result.push(&*self.identifier);
        result.push(&*self.type_);
        result.push(&*self.not_null_opt);
        result.push(&*self.comment_opt);
        result.push(&*self.with_properties_opt);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.identifier
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.with_properties_opt
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.identifier.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.type_.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.not_null_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.comment_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.with_properties_opt.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.with_properties_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.comment_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.not_null_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.type_.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.identifier.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.not);
        result.push(&*self.null);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.not, *self.null)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.not
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.null
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.not.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.null.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.null.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.not.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(4);
        result.push(&*self.like);
        result.push(&*self.name);
        result.push(&*self.option_type_opt);
        result.push(&*self.properties);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.like,
            *self.name,
            *self.option_type_opt,
            *self.properties,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.like
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.properties
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.like.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.name.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.option_type_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.properties.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.properties.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.option_type_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.name.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.like.get_last_token() {
            return Some(token);
        }
        None
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

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(5);
        result.push(&*self.insert);
        result.push(&*self.into);
        result.push(&*self.table_name);
        result.push(&*self.column_aliases_opt);
        result.push(&*self.query);
        result
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

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.insert
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.query
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.insert.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.into.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.table_name.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.column_aliases_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.query.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.column_aliases_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.table_name.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.into.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.insert.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct Delete<'a> {
    pub delete: Box<ParseTree<'a>>,
    pub from: Box<ParseTree<'a>>,
    pub table_name: Box<ParseTree<'a>>,
    pub where_opt: Box<ParseTree<'a>>,
    pub predicate: Box<ParseTree<'a>>,
}

pub fn delete<'a>(
    delete: ParseTree<'a>,
    from: ParseTree<'a>,
    table_name: ParseTree<'a>,
    where_opt: ParseTree<'a>,
    predicate: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Delete(Delete {
        delete: Box::new(delete),
        from: Box::new(from),
        table_name: Box::new(table_name),
        where_opt: Box::new(where_opt),
        predicate: Box::new(predicate),
    })
}

impl<'a> Delete<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Delete(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(5);
        result.push(&*self.delete);
        result.push(&*self.from);
        result.push(&*self.table_name);
        result.push(&*self.where_opt);
        result.push(&*self.predicate);
        result
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
            *self.delete,
            *self.from,
            *self.table_name,
            *self.where_opt,
            *self.predicate,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.delete
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.predicate
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.delete.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.table_name.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.where_opt.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.predicate.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.predicate.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.where_opt.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.table_name.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.from.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.delete.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct GroupingSet<'a> {
    pub elements: Box<ParseTree<'a>>,
}

pub fn grouping_set<'a>(elements: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::GroupingSet(GroupingSet {
        elements: Box::new(elements),
    })
}

impl<'a> GroupingSet<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::GroupingSet(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(1);
        result.push(&*self.elements);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>,) {
        (*self.elements,)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.elements
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.elements
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.elements.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.elements.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct RelationOrQuery<'a> {
    pub open_paren: Box<ParseTree<'a>>,
    pub query_or_relation: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn relation_or_query<'a>(
    open_paren: ParseTree<'a>,
    query_or_relation: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::RelationOrQuery(RelationOrQuery {
        open_paren: Box::new(open_paren),
        query_or_relation: Box::new(query_or_relation),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> RelationOrQuery<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::RelationOrQuery(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.open_paren);
        result.push(&*self.query_or_relation);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.open_paren, *self.query_or_relation, *self.close_paren)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.open_paren
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.query_or_relation.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.query_or_relation.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct EmptyGroupingSet<'a> {
    pub open_paren: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn empty_grouping_set<'a>(
    open_paren: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::EmptyGroupingSet(EmptyGroupingSet {
        open_paren: Box::new(open_paren),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> EmptyGroupingSet<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::EmptyGroupingSet(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.open_paren);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.open_paren, *self.close_paren)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.open_paren
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionOrQuery<'a> {
    pub open_paren: Box<ParseTree<'a>>,
    pub expression_or_query: Box<ParseTree<'a>>,
    pub close_paren: Box<ParseTree<'a>>,
}

pub fn expression_or_query<'a>(
    open_paren: ParseTree<'a>,
    expression_or_query: ParseTree<'a>,
    close_paren: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::ExpressionOrQuery(ExpressionOrQuery {
        open_paren: Box::new(open_paren),
        expression_or_query: Box::new(expression_or_query),
        close_paren: Box::new(close_paren),
    })
}

impl<'a> ExpressionOrQuery<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::ExpressionOrQuery(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.open_paren);
        result.push(&*self.expression_or_query);
        result.push(&*self.close_paren);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (
            *self.open_paren,
            *self.expression_or_query,
            *self.close_paren,
        )
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.open_paren
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.close_paren
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.open_paren.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.expression_or_query.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.close_paren.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.close_paren.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.expression_or_query.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.open_paren.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct Entrypoint<'a> {
    pub beginning_of_file: Box<ParseTree<'a>>,
    pub tree: Box<ParseTree<'a>>,
    pub end_of_file: Box<ParseTree<'a>>,
}

pub fn entrypoint<'a>(
    beginning_of_file: ParseTree<'a>,
    tree: ParseTree<'a>,
    end_of_file: ParseTree<'a>,
) -> ParseTree<'a> {
    ParseTree::Entrypoint(Entrypoint {
        beginning_of_file: Box::new(beginning_of_file),
        tree: Box::new(tree),
        end_of_file: Box::new(end_of_file),
    })
}

impl<'a> Entrypoint<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::Entrypoint(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(3);
        result.push(&*self.beginning_of_file);
        result.push(&*self.tree);
        result.push(&*self.end_of_file);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>, ParseTree<'a>) {
        (*self.beginning_of_file, *self.tree, *self.end_of_file)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.beginning_of_file
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.end_of_file
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.beginning_of_file.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.tree.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.end_of_file.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.end_of_file.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.tree.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.beginning_of_file.get_last_token() {
            return Some(token);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct NullTreatment<'a> {
    pub treatment: Box<ParseTree<'a>>,
    pub nulls: Box<ParseTree<'a>>,
}

pub fn null_treatment<'a>(treatment: ParseTree<'a>, nulls: ParseTree<'a>) -> ParseTree<'a> {
    ParseTree::NullTreatment(NullTreatment {
        treatment: Box::new(treatment),
        nulls: Box::new(nulls),
    })
}

impl<'a> NullTreatment<'a> {
    pub fn to_tree(self) -> ParseTree<'a> {
        ParseTree::NullTreatment(self)
    }

    pub fn children(&self) -> Vec<&ParseTree<'a>> {
        let mut result = Vec::with_capacity(2);
        result.push(&*self.treatment);
        result.push(&*self.nulls);
        result
    }

    pub fn unbox(self) -> (ParseTree<'a>, ParseTree<'a>) {
        (*self.treatment, *self.nulls)
    }

    pub fn get_first_child(&self) -> &ParseTree<'a> {
        &self.treatment
    }

    pub fn get_last_child(&self) -> &ParseTree<'a> {
        &self.nulls
    }

    pub fn get_first_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.treatment.get_first_token() {
            return Some(token);
        }
        if let Some(token) = self.nulls.get_first_token() {
            return Some(token);
        }
        None
    }
    pub fn get_last_token(&self) -> Option<&token::Token<'a>> {
        if let Some(token) = self.nulls.get_last_token() {
            return Some(token);
        }
        if let Some(token) = self.treatment.get_last_token() {
            return Some(token);
        }
        None
    }
}
