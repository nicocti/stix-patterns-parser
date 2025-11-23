from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from typing import List, Literal, Optional, TypeAlias, Union

class ComparisonOp(Enum):
    EQ = "="
    NEQ = "!="
    GT = ">"
    LT = "<"
    GE = ">="
    LE = "<="
    IN = "IN"
    LIKE = "LIKE"
    MATCHES = "MATCHES"
    ISSUBSET = "ISSUBSET"
    ISSUPERSET = "ISSUPERSET"

class UnaryOp(Enum):
    EXISTS = "EXISTS"

class BooleanOp(Enum):
    AND = "AND"
    OR = "OR"

class ObservationOp(Enum):
    AND = "AND"
    OR = "OR"
    FOLLOWEDBY = "FOLLOWEDBY"

StixConstant = Union[str, int, float, bool, datetime]

@dataclass(frozen=True)
class PathComponent:
    property: str
    index: Optional[Union[int, Literal["*"]]] = None

@dataclass(frozen=True)
class ObjectPath:
    object_type: str
    property_path: List[PathComponent]

ComparisonExpression: TypeAlias = Union["Comparison", "CompositeComparison"]
PatternExpression: TypeAlias = Union[
    ComparisonExpression, "CompositePattern", "QualifiedPattern"
]

@dataclass(frozen=True)
class Comparison:
    object_path: ObjectPath
    op: Union[ComparisonOp, UnaryOp]
    constant: Optional[Union[StixConstant, List[StixConstant]]] = None
    negated: bool = False

@dataclass(frozen=True)
class CompositeComparison:
    left: ComparisonExpression
    op: BooleanOp
    right: ComparisonExpression

@dataclass(frozen=True)
class CompositePattern:
    left: PatternExpression
    op: ObservationOp
    right: PatternExpression

@dataclass(frozen=True)
class QualifiedPattern:
    pattern: PatternExpression
    repeat: Optional[int] = None
    within: Optional[float] = None
    start: Optional[datetime] = None
    stop: Optional[datetime] = None

def parse(pattern: str) -> PatternExpression: ...
