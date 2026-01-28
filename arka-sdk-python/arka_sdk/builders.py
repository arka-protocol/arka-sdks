"""
Rule and Condition Builders

Fluent builder APIs for creating PACT rules and conditions.
"""

from __future__ import annotations

from datetime import datetime
from typing import Any
from uuid import uuid4

from arka_sdk.types import (
    PactRule,
    PactCondition,
    PactConsequence,
    CompareCondition,
    AndCondition,
    OrCondition,
    NotCondition,
    ExistsCondition,
    InCondition,
    RangeCondition,
    ExpressionCondition,
    Severity,
    Decision,
)


class ConditionBuilder:
    """
    Fluent builder for creating PACT conditions.

    Example:
        >>> cond = (
        ...     condition()
        ...     .field("amount")
        ...     .gt(10000)
        ...     .build()
        ... )

        >>> complex_cond = (
        ...     condition()
        ...     .and_([
        ...         condition().field("amount").gt(10000).build(),
        ...         condition().field("currency").eq("USD").build(),
        ...     ])
        ...     .build()
        ... )
    """

    def __init__(self):
        self._condition: PactCondition | None = None
        self._field: str | None = None

    def field(self, path: str) -> ConditionBuilder:
        """Sets the field path for comparison."""
        self._field = path
        return self

    def eq(self, value: Any) -> ConditionBuilder:
        """Equal comparison."""
        if not self._field:
            raise ValueError("Field must be set before comparison")
        self._condition = CompareCondition(field=self._field, operator="eq", value=value)
        return self

    def ne(self, value: Any) -> ConditionBuilder:
        """Not equal comparison."""
        if not self._field:
            raise ValueError("Field must be set before comparison")
        self._condition = CompareCondition(field=self._field, operator="ne", value=value)
        return self

    def gt(self, value: Any) -> ConditionBuilder:
        """Greater than comparison."""
        if not self._field:
            raise ValueError("Field must be set before comparison")
        self._condition = CompareCondition(field=self._field, operator="gt", value=value)
        return self

    def gte(self, value: Any) -> ConditionBuilder:
        """Greater than or equal comparison."""
        if not self._field:
            raise ValueError("Field must be set before comparison")
        self._condition = CompareCondition(field=self._field, operator="gte", value=value)
        return self

    def lt(self, value: Any) -> ConditionBuilder:
        """Less than comparison."""
        if not self._field:
            raise ValueError("Field must be set before comparison")
        self._condition = CompareCondition(field=self._field, operator="lt", value=value)
        return self

    def lte(self, value: Any) -> ConditionBuilder:
        """Less than or equal comparison."""
        if not self._field:
            raise ValueError("Field must be set before comparison")
        self._condition = CompareCondition(field=self._field, operator="lte", value=value)
        return self

    def contains(self, value: str) -> ConditionBuilder:
        """String contains comparison."""
        if not self._field:
            raise ValueError("Field must be set before comparison")
        self._condition = CompareCondition(field=self._field, operator="contains", value=value)
        return self

    def starts_with(self, value: str) -> ConditionBuilder:
        """String starts with comparison."""
        if not self._field:
            raise ValueError("Field must be set before comparison")
        self._condition = CompareCondition(field=self._field, operator="startsWith", value=value)
        return self

    def ends_with(self, value: str) -> ConditionBuilder:
        """String ends with comparison."""
        if not self._field:
            raise ValueError("Field must be set before comparison")
        self._condition = CompareCondition(field=self._field, operator="endsWith", value=value)
        return self

    def matches(self, pattern: str) -> ConditionBuilder:
        """Regex match comparison."""
        if not self._field:
            raise ValueError("Field must be set before comparison")
        self._condition = CompareCondition(field=self._field, operator="regex", value=pattern)
        return self

    def exists(self) -> ConditionBuilder:
        """Field exists condition."""
        if not self._field:
            raise ValueError("Field must be set for exists check")
        self._condition = ExistsCondition(field=self._field)
        return self

    def in_(self, values: list[Any]) -> ConditionBuilder:
        """Value in set condition."""
        if not self._field:
            raise ValueError("Field must be set for in check")
        self._condition = InCondition(field=self._field, values=values)
        return self

    def between(
        self,
        min_val: float | None = None,
        max_val: float | None = None,
        *,
        min_inclusive: bool = True,
        max_inclusive: bool = True,
    ) -> ConditionBuilder:
        """Numeric range condition."""
        if not self._field:
            raise ValueError("Field must be set for range check")
        self._condition = RangeCondition(
            field=self._field,
            min=min_val,
            max=max_val,
            min_inclusive=min_inclusive,
            max_inclusive=max_inclusive,
        )
        return self

    def and_(self, conditions: list[PactCondition]) -> ConditionBuilder:
        """Logical AND of conditions."""
        self._condition = AndCondition(conditions=conditions)
        return self

    def or_(self, conditions: list[PactCondition]) -> ConditionBuilder:
        """Logical OR of conditions."""
        self._condition = OrCondition(conditions=conditions)
        return self

    def not_(self, cond: PactCondition) -> ConditionBuilder:
        """Logical NOT of condition."""
        self._condition = NotCondition(condition=cond)
        return self

    def expression(self, expr: str, language: str = "cel") -> ConditionBuilder:
        """Custom expression condition."""
        self._condition = ExpressionCondition(expression=expr, language=language)
        return self

    def build(self) -> PactCondition:
        """Builds and returns the condition."""
        if self._condition is None:
            raise ValueError("No condition set")
        return self._condition


class RuleBuilder:
    """
    Fluent builder for creating PACT rules.

    Example:
        >>> my_rule = (
        ...     rule()
        ...     .name("High Value Transaction Check")
        ...     .description("Flag transactions over $10,000")
        ...     .jurisdiction("US")
        ...     .severity("HIGH")
        ...     .when(condition().field("amount").gt(10000).build())
        ...     .then_flag("HIGH_VALUE", "Transaction exceeds $10,000 threshold")
        ...     .tags("aml", "threshold")
        ...     .build()
        ... )
    """

    def __init__(self):
        self._id: str | None = None
        self._name: str | None = None
        self._description: str = ""
        self._jurisdiction: str | None = None
        self._severity: Severity = Severity.MEDIUM
        self._condition: PactCondition | None = None
        self._consequence: PactConsequence | None = None
        self._tags: list[str] = []
        self._effective_from: datetime | None = None
        self._effective_to: datetime | None = None
        self._metadata: dict[str, Any] = {}

    def id(self, rule_id: str) -> RuleBuilder:
        """Sets the rule ID."""
        self._id = rule_id
        return self

    def name(self, name: str) -> RuleBuilder:
        """Sets the rule name."""
        self._name = name
        return self

    def description(self, desc: str) -> RuleBuilder:
        """Sets the rule description."""
        self._description = desc
        return self

    def jurisdiction(self, jurisdiction: str) -> RuleBuilder:
        """Sets the jurisdiction scope."""
        self._jurisdiction = jurisdiction
        return self

    def severity(self, severity: str | Severity) -> RuleBuilder:
        """Sets the rule severity."""
        self._severity = Severity(severity) if isinstance(severity, str) else severity
        return self

    def when(self, cond: PactCondition) -> RuleBuilder:
        """Sets the rule condition."""
        self._condition = cond
        return self

    def when_field(self, field: str, operator: str, value: Any) -> RuleBuilder:
        """Sets a simple field comparison condition."""
        self._condition = CompareCondition(field=field, operator=operator, value=value)
        return self

    def and_conditions(self, conditions: list[PactCondition]) -> RuleBuilder:
        """Sets an AND condition."""
        self._condition = AndCondition(conditions=conditions)
        return self

    def or_conditions(self, conditions: list[PactCondition]) -> RuleBuilder:
        """Sets an OR condition."""
        self._condition = OrCondition(conditions=conditions)
        return self

    def then_deny(self, code: str, message: str) -> RuleBuilder:
        """Sets a DENY consequence."""
        self._consequence = PactConsequence(decision=Decision.DENY, code=code, message=message)
        return self

    def then_flag(self, code: str, message: str) -> RuleBuilder:
        """Sets a FLAG consequence."""
        self._consequence = PactConsequence(decision=Decision.FLAG, code=code, message=message)
        return self

    def then_allow(self, code: str = "ALLOWED", message: str = "Allowed") -> RuleBuilder:
        """Sets an ALLOW consequence."""
        self._consequence = PactConsequence(decision=Decision.ALLOW, code=code, message=message)
        return self

    def consequence(self, consequence: PactConsequence) -> RuleBuilder:
        """Sets the consequence directly."""
        self._consequence = consequence
        return self

    def tags(self, *tags: str) -> RuleBuilder:
        """Adds tags to the rule."""
        self._tags.extend(tags)
        return self

    def effective_from(self, date: datetime) -> RuleBuilder:
        """Sets the effective start date."""
        self._effective_from = date
        return self

    def effective_to(self, date: datetime) -> RuleBuilder:
        """Sets the effective end date."""
        self._effective_to = date
        return self

    def metadata(self, **kwargs: Any) -> RuleBuilder:
        """Adds metadata to the rule."""
        self._metadata.update(kwargs)
        return self

    def build(self) -> PactRule:
        """Builds and returns the rule."""
        if self._name is None:
            raise ValueError("Rule name is required")
        if self._condition is None:
            raise ValueError("Rule condition is required")
        if self._consequence is None:
            raise ValueError("Rule consequence is required")

        return PactRule(
            id=self._id or f"rule_{uuid4().hex[:12]}",
            name=self._name,
            description=self._description or self._name,
            jurisdiction=self._jurisdiction,
            severity=self._severity,
            condition=self._condition,
            consequence=self._consequence,
            tags=self._tags,
            effective_from=self._effective_from,
            effective_to=self._effective_to,
            metadata=self._metadata,
        )


def rule() -> RuleBuilder:
    """Creates a new rule builder."""
    return RuleBuilder()


def condition() -> ConditionBuilder:
    """Creates a new condition builder."""
    return ConditionBuilder()
