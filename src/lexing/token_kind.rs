#[allow(non_upper_case_globals, non_camel_case_types)]
pub enum TokenKind {
    // Common kinds
    BEGINING_OF_FILE,
    END_OF_FILE,
    ERROR,

    // operators and punctuators
    OpenParen,
    CloseParen,
    Comma,
    Period,
    OpenAngle,
    CloseAngle,
    OpenSquare,
    CloseSquare,
    Equal,
    LessGreater,
    BangEqual,
    LessEqual,
    GreaterEqual,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    BarBar,
    DoubleArrow,

    // literals
    String,
    UnicodeString,
    BinaryLiteral,
    Integer,
    Decimal,
    Double,

    // identifier
    Identifier,
    QuotedIdentifier,
    BackquotedIdentifier,

    // reserved words
    ALTER,
    AND,
    AS,
    BETWEEN,
    BIGINT,
    BOOLEAN,
    BY,
    CASE,
    CAST,
    CONSTRAINT,
    CREATE,
    CROSS,
    CUBE,
    CURRENT_DATE,
    CURRENT_PATH,
    CURRENT_TIME,
    CURRENT_TIMESTAMP,
    CURRENT_USER,
    DEALLOCATE,
    DECIMAL,
    DELETE,
    DESCRIBE,
    DISTINCT,
    DOUBLE,
    DROP,
    ELSE,
    END,
    ESCAPE,
    EXCEPT,
    EXECUTE,
    EXISTS,
    EXTRACT,
    FALSE,
    FOR,
    FROM,
    FULL,
    FUNCTION,
    GROUP,
    GROUPING,
    HAVING,
    IN,
    INNER,
    INSERT,
    INTEGER,
    INTERSECT,
    INTO,
    IS,
    JOIN,
    LEFT,
    LIKE,
    LOCALTIME,
    LOCALTIMESTAMP,
    NATURAL,
    NORMALIZE,
    NOT,
    NULL,
    ON,
    OR,
    ORDER,
    OUTER,
    PREPARE,
    REAL,
    RECURSIVE,
    RIGHT,
    ROLLUP,
    SELECT,
    SMALLINT,
    TABLE,
    THEN,
    TINYINT,
    TRUE,
    UESCAPE,
    UNION,
    UNNEST,
    USING,
    VALUES,
    VARBINARY,
    VARCHAR,
    WHEN,
    WHERE,
    WITH,
}

impl TokenKind {
    fn is_keyword(&self) -> bool {
        match self {
            TokenKind::ALTER
            | TokenKind::AND
            | TokenKind::AS
            | TokenKind::BETWEEN
            | TokenKind::BIGINT
            | TokenKind::BOOLEAN
            | TokenKind::BY
            | TokenKind::CASE
            | TokenKind::CAST
            | TokenKind::CONSTRAINT
            | TokenKind::CREATE
            | TokenKind::CROSS
            | TokenKind::CUBE
            | TokenKind::CURRENT_DATE
            | TokenKind::CURRENT_PATH
            | TokenKind::CURRENT_TIME
            | TokenKind::CURRENT_TIMESTAMP
            | TokenKind::CURRENT_USER
            | TokenKind::DEALLOCATE
            | TokenKind::DECIMAL
            | TokenKind::DELETE
            | TokenKind::DESCRIBE
            | TokenKind::DISTINCT
            | TokenKind::DOUBLE
            | TokenKind::DROP
            | TokenKind::ELSE
            | TokenKind::END
            | TokenKind::ESCAPE
            | TokenKind::EXCEPT
            | TokenKind::EXECUTE
            | TokenKind::EXISTS
            | TokenKind::EXTRACT
            | TokenKind::FALSE
            | TokenKind::FOR
            | TokenKind::FROM
            | TokenKind::FULL
            | TokenKind::FUNCTION
            | TokenKind::GROUP
            | TokenKind::GROUPING
            | TokenKind::HAVING
            | TokenKind::IN
            | TokenKind::INNER
            | TokenKind::INSERT
            | TokenKind::INTEGER
            | TokenKind::INTERSECT
            | TokenKind::INTO
            | TokenKind::IS
            | TokenKind::JOIN
            | TokenKind::LEFT
            | TokenKind::LIKE
            | TokenKind::LOCALTIME
            | TokenKind::LOCALTIMESTAMP
            | TokenKind::NATURAL
            | TokenKind::NORMALIZE
            | TokenKind::NOT
            | TokenKind::NULL
            | TokenKind::ON
            | TokenKind::OR
            | TokenKind::ORDER
            | TokenKind::OUTER
            | TokenKind::PREPARE
            | TokenKind::REAL
            | TokenKind::RECURSIVE
            | TokenKind::RIGHT
            | TokenKind::ROLLUP
            | TokenKind::SELECT
            | TokenKind::SMALLINT
            | TokenKind::TABLE
            | TokenKind::THEN
            | TokenKind::TINYINT
            | TokenKind::TRUE
            | TokenKind::UESCAPE
            | TokenKind::UNION
            | TokenKind::UNNEST
            | TokenKind::USING
            | TokenKind::VALUES
            | TokenKind::VARBINARY
            | TokenKind::VARCHAR
            | TokenKind::WHEN
            | TokenKind::WHERE
            | TokenKind::WITH => true,
            _ => false,
        }
    }

    fn is_complex(&self) -> bool {
        match self {
            TokenKind::String
            | TokenKind::UnicodeString
            | TokenKind::BinaryLiteral
            | TokenKind::Decimal
            | TokenKind::Double
            | TokenKind::Identifier
            | TokenKind::QuotedIdentifier
            | TokenKind::BackquotedIdentifier => true,
            _ => false,
        }
    }

    fn is_simple(&self) -> bool {
        return !self.is_complex();
    }
}
