"""
PACT SDK Type Definitions

Core types for building PACT domain plugins.
"""

from __future__ import annotations

from datetime import datetime
from enum import Enum
from typing import Any, Literal, Union

from pydantic import BaseModel, Field


class Severity(str, Enum):
    """Rule severity levels."""

    LOW = "LOW"
    MEDIUM = "MEDIUM"
    HIGH = "HIGH"
    CRITICAL = "CRITICAL"


class Decision(str, Enum):
    """Rule consequence decisions."""

    ALLOW = "ALLOW"
    DENY = "DENY"
    FLAG = "FLAG"


class DecisionStatus(str, Enum):
    """Overall decision status."""

    ALLOW = "ALLOW"
    ALLOW_WITH_FLAGS = "ALLOW_WITH_FLAGS"
    DENY = "DENY"


class PluginManifest(BaseModel):
    """Plugin manifest describing capabilities."""

    id: str = Field(..., description="Unique plugin identifier")
    name: str = Field(..., description="Human-readable name")
    version: str = Field(..., description="Semantic version")
    author: str = Field(..., description="Plugin author/organization")
    description: str = Field(..., description="Plugin description")
    entity_types: list[str] = Field(default_factory=list, description="Entity types defined")
    event_types: list[str] = Field(default_factory=list, description="Event types handled")
    dependencies: list[str] = Field(default_factory=list, description="Plugin dependencies")
    pact_core_version: str = Field(..., description="Required PACT Core version")
    config_schema: dict[str, Any] | None = Field(default=None, description="Config JSON schema")


class DomainEvent(BaseModel):
    """Domain-specific event before conversion to canonical format."""

    type: str = Field(..., description="Domain-specific event type")
    payload: dict[str, Any] = Field(default_factory=dict, description="Event payload")
    entity_id: str | None = Field(default=None, description="Related entity ID")
    jurisdiction: str | None = Field(default=None, description="Jurisdiction code")
    occurred_at: datetime | None = Field(default=None, description="When event occurred")
    metadata: dict[str, Any] = Field(default_factory=dict, description="Additional metadata")


class PactEvent(BaseModel):
    """Canonical PACT event format."""

    id: str = Field(..., description="Unique event ID")
    source: str = Field(..., description="Event source (plugin ID)")
    type: str = Field(..., description="Event type")
    entity_id: str | None = Field(default=None, description="Related entity ID")
    entity_type: str | None = Field(default=None, description="Entity type")
    jurisdiction: str | None = Field(default=None, description="Jurisdiction code")
    payload: dict[str, Any] = Field(default_factory=dict, description="Event payload")
    occurred_at: datetime = Field(..., description="When event occurred")
    received_at: datetime = Field(..., description="When event was received")
    metadata: dict[str, Any] = Field(default_factory=dict, description="Additional metadata")


class PactEntity(BaseModel):
    """PACT entity representation."""

    id: str = Field(..., description="Unique entity ID")
    type: str = Field(..., description="Entity type name")
    data: dict[str, Any] = Field(default_factory=dict, description="Entity data")
    created_at: datetime = Field(..., description="Creation timestamp")
    updated_at: datetime = Field(..., description="Last update timestamp")
    jurisdiction: str | None = Field(default=None, description="Jurisdiction code")
    metadata: dict[str, Any] = Field(default_factory=dict, description="Additional metadata")


class PactEntityType(BaseModel):
    """Entity type definition with schema."""

    name: str = Field(..., description="Entity type name")
    description: str = Field(default="", description="Entity type description")
    schema: dict[str, Any] = Field(..., description="JSON Schema for validation")
    required_fields: list[str] = Field(default_factory=list, description="Required fields")


# Condition types
class CompareCondition(BaseModel):
    """Comparison condition."""

    type: Literal["compare"] = "compare"
    field: str = Field(..., description="Field path to compare")
    operator: str = Field(..., description="Comparison operator")
    value: Any = Field(..., description="Value to compare against")


class AndCondition(BaseModel):
    """Logical AND condition."""

    type: Literal["and"] = "and"
    conditions: list["PactCondition"] = Field(..., description="Conditions to AND together")


class OrCondition(BaseModel):
    """Logical OR condition."""

    type: Literal["or"] = "or"
    conditions: list["PactCondition"] = Field(..., description="Conditions to OR together")


class NotCondition(BaseModel):
    """Logical NOT condition."""

    type: Literal["not"] = "not"
    condition: "PactCondition" = Field(..., description="Condition to negate")


class ExistsCondition(BaseModel):
    """Field existence condition."""

    type: Literal["exists"] = "exists"
    field: str = Field(..., description="Field path to check")


class InCondition(BaseModel):
    """Value in set condition."""

    type: Literal["in"] = "in"
    field: str = Field(..., description="Field path to check")
    values: list[Any] = Field(..., description="Set of allowed values")


class RangeCondition(BaseModel):
    """Numeric range condition."""

    type: Literal["range"] = "range"
    field: str = Field(..., description="Field path to check")
    min: float | None = Field(default=None, description="Minimum value")
    max: float | None = Field(default=None, description="Maximum value")
    min_inclusive: bool = Field(default=True, description="Include minimum")
    max_inclusive: bool = Field(default=True, description="Include maximum")


class ExpressionCondition(BaseModel):
    """Custom expression condition."""

    type: Literal["expression"] = "expression"
    expression: str = Field(..., description="Expression string")
    language: str = Field(default="cel", description="Expression language")


PactCondition = Union[
    CompareCondition,
    AndCondition,
    OrCondition,
    NotCondition,
    ExistsCondition,
    InCondition,
    RangeCondition,
    ExpressionCondition,
]

# Update forward refs for recursive types
AndCondition.model_rebuild()
OrCondition.model_rebuild()
NotCondition.model_rebuild()


class PactConsequence(BaseModel):
    """Rule consequence definition."""

    decision: Decision = Field(..., description="Decision type")
    code: str = Field(..., description="Error/warning code")
    message: str = Field(..., description="Human-readable message")
    metadata: dict[str, Any] = Field(default_factory=dict, description="Additional metadata")


class PactRule(BaseModel):
    """PACT rule definition."""

    id: str = Field(..., description="Unique rule ID")
    name: str = Field(..., description="Rule name")
    description: str = Field(default="", description="Rule description")
    jurisdiction: str | None = Field(default=None, description="Jurisdiction scope")
    severity: Severity = Field(default=Severity.MEDIUM, description="Rule severity")
    condition: PactCondition = Field(..., description="Rule condition")
    consequence: PactConsequence = Field(..., description="Rule consequence")
    tags: list[str] = Field(default_factory=list, description="Rule tags")
    effective_from: datetime | None = Field(default=None, description="Effective start date")
    effective_to: datetime | None = Field(default=None, description="Effective end date")
    metadata: dict[str, Any] = Field(default_factory=dict, description="Additional metadata")


class RuleEvaluation(BaseModel):
    """Result of evaluating a single rule."""

    rule_id: str = Field(..., description="Rule ID")
    rule_name: str = Field(..., description="Rule name")
    result: Literal["PASS", "FAIL", "SKIP", "ERROR"] = Field(..., description="Evaluation result")
    code: str | None = Field(default=None, description="Result code")
    message: str | None = Field(default=None, description="Result message")
    duration_ms: int = Field(default=0, description="Evaluation duration in milliseconds")


class PactDecision(BaseModel):
    """Decision result from rule evaluation."""

    id: str = Field(..., description="Decision ID")
    event_id: str = Field(..., description="Source event ID")
    status: DecisionStatus = Field(..., description="Overall decision status")
    rule_evaluations: list[RuleEvaluation] = Field(
        default_factory=list, description="Individual rule results"
    )
    created_at: datetime = Field(..., description="Decision timestamp")
    metadata: dict[str, Any] = Field(default_factory=dict, description="Additional metadata")


class ValidationError(BaseModel):
    """Validation error details."""

    field: str = Field(..., description="Field with error")
    message: str = Field(..., description="Error message")
    code: str = Field(..., description="Error code")


class ValidationWarning(BaseModel):
    """Validation warning details."""

    field: str = Field(..., description="Field with warning")
    message: str = Field(..., description="Warning message")


class ValidationResult(BaseModel):
    """Result of data validation."""

    valid: bool = Field(..., description="Whether data is valid")
    errors: list[ValidationError] = Field(default_factory=list, description="Validation errors")
    warnings: list[ValidationWarning] = Field(
        default_factory=list, description="Validation warnings"
    )
