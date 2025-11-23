from typing import Iterator, Union

from stix_patterns_parser import stix_patterns_parser

BooleanOp = stix_patterns_parser.BooleanOp
Comparison = stix_patterns_parser.Comparison
ComparisonOp = stix_patterns_parser.ComparisonOp
CompositeComparison = stix_patterns_parser.CompositeComparison
CompositePattern = stix_patterns_parser.CompositePattern
ObservationOp = stix_patterns_parser.ObservationOp
QualifiedPattern = stix_patterns_parser.QualifiedPattern
UnaryOp = stix_patterns_parser.UnaryOp
ObjectPath = stix_patterns_parser.ObjectPath
PathComponent = stix_patterns_parser.PathComponent

ComparisonExpression = Union[Comparison, CompositeComparison]
PatternExpression = Union[ComparisonExpression, CompositePattern, QualifiedPattern]


parse = stix_patterns_parser.parse


class StixPattern(object):
    nodes: PatternExpression
    raw: str

    def __init__(self, pattern: str) -> None:
        self.raw = pattern
        self.nodes = parse(pattern)

    def __str__(self) -> str:
        return self.raw

    def __repr__(self) -> str:
        return self.raw

    def comparisons(self) -> Iterator[Comparison]:
        yield from self.iter_comparisons(self.nodes)

    @classmethod
    def iter_comparisons(cls, node: PatternExpression) -> Iterator[Comparison]:
        """
        Structural pattern matching traversal to extract comparison leaves.
        """
        match node:
            case Comparison():
                yield node

            case CompositeComparison(left=l, right=r):
                yield from cls.iter_comparisons(l)
                yield from cls.iter_comparisons(r)

            case CompositePattern(left=l, right=r):
                yield from cls.iter_comparisons(l)
                yield from cls.iter_comparisons(r)

            case QualifiedPattern(pattern=p):
                yield from cls.iter_comparisons(p)
