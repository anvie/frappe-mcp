# Comprehensive Guide to Frappe DocType Naming

## Overview

- The name field on every DocType is the primary key used to identify and
  retrieve documents. Naming can be automated or customized through several mechanisms:
  - Autoname options on the DocType
  - A naming_series field on the document
  - A full Format-based naming rule
  - A controller-based autoname method
  - Document Naming Rules (dynamic, conditional naming)
- This guide explains each mechanism, how to configure it, and best practices for maintainability and consistency.

## 1. Core concept: how names are generated

- Every DocType has a name field that serves as the unique identifier for records in the system. The method used to populate this name is called autonaming, and it can be defined in several ways as described below.
- When a new document is created, the system evaluates the configured naming strategy and assigns the resulting string to document.name. This process can be overridden or extended by custom Python code or by Document Naming Rules.

## 2. Autoname options on a DocType

- field:[fieldname]
  - Description: Use the value of a specific field as the document name.
  - Example: DocType Task with autoname = field:title results in name equal to Task.title.
  - Best for: simple, human-readable identifiers tied to a single field.
- [series]
  - Description: Use a predefined, incrementing pattern.
  - Example: naming = PRE.##### yields PRE00001, PRE00002, etc.
  - Best for: stable, fixed prefixes with a numeric sequence; single consistent pattern per DocType.
- naming_series
  - Description: The pattern is taken from a field named naming_series on the document; the value can vary per document.
  - Example: naming_series field value PRE.#####: first doc PRE0001, next PRE0002, etc.
  - Best for: multi-pattern naming within the same DocType when different documents require different prefixes.
- Prompt
  - Description: Requires manual entry of the name by the user at creation time.
  - Best for: rare cases where unique, manual IDs are essential.
- Format
  - Description: The most flexible approach; a template that can mix plain text with placeholders that are evaluated at creation.
  - Example: EXAMPLE-{MM}-test-{fieldname1}-{fieldname2}-{#####} -> produces a dynamic name incorporating month, field values, and a numeric suffix.
  - Best for: highly customized naming schemes that depend on multiple fields, dates, and a serial portion.
- By Controller Method
  - Description: Implement a custom autoname(self) method in the DocType’s Python controller to set self.name programmatically (e.g., using a getseries helper).
  - Example (conceptual):
    def autoname(self):
    prefix = f"PRJ-{self.customer}-"
    self.name = getseries(prefix, 4)
  - Best for: complex, project-specific logic not covered by built-in options.
- By Document Naming Rule
  - Description: Define multiple conditional rules that apply to a DocType. Each rule can specify a prefix, digits, and conditions under which it applies.
  - Best for: multi-branch, multi-country, multi-category naming needs; highly dynamic environments.

## 3. Document Naming Rules (dynamic naming)

- Purpose: Apply conditional naming based on document field values or state.
- How to configure:
  - Create a Document Naming Rule entry for a specific DocType.
  - Set Priority: higher numbers apply first.
  - Add one or more Conditions to determine when the rule should apply.
  - Define the Naming Pattern for that rule (Prefix, Digits, or a Format-like pattern).
- Behavior:
  - If multiple rules match, the rule with the highest priority is used.
  - If no rule matches, the system falls back to the DocType’s global naming setting (e.g., the default autoname configuration).
- Example use case:
  - Sales Order naming:
    - Rule 1 (Priority 10): Prefix = SO-INDIA-, Condition = country = India; Digits = 5
    - Rule 2 (Priority 8): Prefix = SO-USA-, Condition = country = USA; Digits = 5
  - Result: Indian orders named SO-INDIA-00001, 00002, …; US orders named SO-USA-00001, 00002, …
- Practical notes:
  - Ensure conditions are specific to avoid overlap or conflicts.
  - Document Naming Rules can be edited or extended without code changes, aiding governance.

## 4. Formatting and placeholders (Format option)

- Placeholders and examples:
  - Static text remains as-is.
  - Field placeholders: {fieldname}
  - Date placeholders: {YY}, {MM}, {DD} (and other date tokens depending on the system)
  - Serial placeholder: {#####} or other digits placeholders
- Example:
  - Pattern: INV-{YY}{MM}-{customer}-{#####}
  - Result: INV-2510-ACME-00001 for October 2025, customer ACME
- Guidance:
  - Use descriptive, stable placeholders to ensure readability and future-proofing.
  - Combine with a naming_series or a serial portion for uniqueness.

## 5. Naming Series (the standard, backwards-compatible approach)

- Naming Series is a per-DocType feature that allows a single prefix with a numeric suffix.
- Config: In the DocType, pick Autoname: naming_series and set a default series value in the Doctype or per-document via a naming_series field.
- Examples:
  - Prefix: SO-.##### -> SO-00001, SO-00002
  - Company-wide naming: SINV-.#####
- Useful for: simple, predictable sequencing across a DocType, possibly with per-document company or branch differentiation if implemented via rules.

## 6. Controller-based autonaming (custom code)

- When the standard options are insufficient, implement autoname in the DocType’s controller to set the name dynamically.
- Typical pattern:
  - from frappe.model.naming import getseries
  - class MyDocType(Document):
    def autoname(self):
    prefix = f"M-{self.customer}-"
    self.name = getseries(prefix, 4)
- Pros:
  - Maximum flexibility; can depend on multiple fields, external lookups, or business logic.
- Cons:
  - Requires Python code changes and careful testing across upgrades.

## 7. Practical guidance and best practices

- Start with a simple, consistent scheme (naming_series or Format) before moving to Conditional Naming Rules.
- Document naming conventions in a central governance document for the team.
- Favor human-readable prefixes where possible to aid users in recognizing documents at a glance.
- Favor Document Naming Rules for complex, multi-branch scenarios to avoid hard-coding in code.
- Maintain backward compatibility: plan migration paths when changing naming rules for existing documents.
- Test thoroughly in a sandbox before applying naming changes in production.

## 8. Configuration steps (high-level)

- Autoname options
  - Navigate to DocType settings.
  - Set Autoname to one of the supported options: field, [series], naming_series, Prompt, or Format.
  - If using Format, define the template with proper placeholders.
  - If using naming_series, configure the default or per-document naming_series value.
- Document Naming Rules
  - Open the target DocType.
  - Add a new Document Naming Rule.
  - Set Priority, Conditions, Prefix/Digits or Format for the rule.
  - Save and test by creating new documents to validate the naming behavior.
- Controller-based autonaming
  - Implement an autoname(self) method in the DocType’s Python controller as shown above.
  - Ensure the method handles edge cases (missing fields, duplicates, etc.).
- Testing and data migration
  - Create a small set of test documents to verify naming behavior under different conditions.
  - If migrating existing data, consider a one-time script to rename existing documents to new patterns, if necessary, and maintain a mapping log.

## 9. References and further reading

- Naming concepts and options (autoname, field, series, naming_series, Prompt, Format) in the Frappe framework documentation.
- Document Naming Rules: dynamic, conditional naming with priorities and multiple rules.
- Document Naming Settings: base-series and their configuration.
- Community examples and tutorials illustrating practical naming strategies, including custom naming via controllers and naming series customization.
- Additional context and examples from naming guidelines and related discussions.

Direct summary for usage

- If a stable, straightforward naming is needed: use a naming_series (or autoname with [series]) to generate consistent IDs like INV-00001.
- If names must reflect document fields or business context: use autoname = field:[fieldname] or autoname = Format with a detailed template.
- If conditions drive different naming schemes across branches or countries: use Document Naming Rules with appropriate priorities and conditions.
- If complex logic or data lookups are required: implement an autoname(self) controller method or use Document Naming Rules combined with custom logic.

Would you like this turned into a structured, ready-to-publish document (e.g., with a formal table of contents, glossary, and a complete example gallery), shipped as a Markdown, reST, or PDF draft? If yes, I can tailor the formatting, add code snippets in Python, and include a sample migration plan.
