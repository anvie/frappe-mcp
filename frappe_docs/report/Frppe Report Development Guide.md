# Frappe Report Development Guide

## Table of Contents
1. [Overview](#overview)
2. [Report Types](#report-types)
3. [Creating Reports](#creating-reports)
4. [Report Structure](#report-structure)
5. [Permissions and Security](#permissions-and-security)
6. [Advanced Features](#advanced-features)
7. [Best Practices](#best-practices)
8. [Examples](#examples)

## Overview

Reports in Frappe are powerful tools for displaying and analyzing data from your application. The framework provides a comprehensive reporting system that supports various report types, filters, permissions, and data visualization options.

### Key Components
- **Report DocType**: Core doctype that defines report metadata (frappe/core/doctype/report/report.py:20)
- **Query Report Module**: Handles report execution and rendering (frappe/desk/query_report.py)
- **Report View**: Frontend component for displaying reports (frappe/public/js/frappe/views/reports/query_report.js:30)

## Report Types

Frappe supports four main types of reports:

### 1. Report Builder
- **Description**: Simple drag-and-drop interface for creating basic reports
- **Use Case**: Quick reports without coding
- **Location**: Created via the UI in Report View
- **Features**:
  - Visual column selection
  - Basic filtering
  - Sorting and grouping
  - No coding required

### 2. Query Report
- **Description**: Reports based on SQL queries
- **Use Case**: Complex database queries with joins
- **Requirements**: SQL knowledge
- **Security**: Uses `check_safe_sql_query()` for SQL injection prevention (frappe/core/doctype/report/report.py:149)

### 3. Script Report
- **Description**: Python-based reports with full control over data processing
- **Use Case**: Complex business logic, data manipulation, external API calls
- **Files Required**:
  - Python file: `{module}/report/{report_name}/{report_name}.py`
  - JavaScript file (optional): `{module}/report/{report_name}/{report_name}.js`
  - JSON metadata: `{module}/report/{report_name}/{report_name}.json`

### 4. Custom Report
- **Description**: Extends existing reports with custom columns and filters
- **Use Case**: Customizing standard reports without modifying core code
- **Reference**: Links to another report via `reference_report` field

## Creating Reports

### Method 1: Via UI (Report Builder)

1. Navigate to any DocType's List View
2. Click Menu → "Create a Report"
3. Configure columns, filters, and settings
4. Save with a unique name

### Method 2: Via Code (Script Report)

#### Step 1: Create Report Metadata
Create a new Report document programmatically or via UI:

```python
report = frappe.get_doc({
    "doctype": "Report",
    "report_name": "My Custom Report",
    "ref_doctype": "Sales Order",  # The primary doctype
    "report_type": "Script Report",
    "is_standard": "Yes",  # If part of an app
    "module": "Selling",
    "roles": [
        {"role": "Sales User"},
        {"role": "Sales Manager"}
    ]
})
report.insert()
```

#### Step 2: Create Python Handler
Location: `{app}/{module}/report/{report_name}/{report_name}.py`

```python
# Copyright (c) 2024, Your Company and contributors
# For license information, please see license.txt

import frappe
from frappe import _

def execute(filters=None):
    """
    Main execution function for the report

    Args:
        filters: Dictionary of filter values from the UI

    Returns:
        columns: List of column definitions
        data: List of rows (can be list of lists or list of dicts)
        message: Optional message to display
        chart: Optional chart configuration
        report_summary: Optional summary cards
        skip_total_row: Boolean to skip total row
    """
    columns = get_columns()
    data = get_data(filters)
    chart = get_chart_data(data)
    report_summary = get_report_summary(data)

    return columns, data, None, chart, report_summary

def get_columns():
    """Define report columns"""
    return [
        {
            "label": _("Document"),
            "fieldname": "name",
            "fieldtype": "Link",
            "options": "Sales Order",
            "width": 200
        },
        {
            "label": _("Customer"),
            "fieldname": "customer",
            "fieldtype": "Link",
            "options": "Customer",
            "width": 150
        },
        {
            "label": _("Total"),
            "fieldname": "grand_total",
            "fieldtype": "Currency",
            "width": 120
        },
        {
            "label": _("Status"),
            "fieldname": "status",
            "fieldtype": "Data",
            "width": 100
        }
    ]

def get_data(filters):
    """Fetch and process report data"""
    conditions = get_conditions(filters)

    data = frappe.db.sql("""
        SELECT
            name,
            customer,
            grand_total,
            status
        FROM `tabSales Order`
        WHERE docstatus = 1
            {conditions}
        ORDER BY transaction_date DESC
    """.format(conditions=conditions), filters, as_dict=1)

    return data

def get_conditions(filters):
    """Build SQL conditions from filters"""
    conditions = []

    if filters.get("company"):
        conditions.append("company = %(company)s")

    if filters.get("from_date"):
        conditions.append("transaction_date >= %(from_date)s")

    if filters.get("to_date"):
        conditions.append("transaction_date <= %(to_date)s")

    return "AND " + " AND ".join(conditions) if conditions else ""

def get_chart_data(data):
    """Generate chart configuration"""
    return {
        "data": {
            "labels": [d.get("customer") for d in data[:10]],
            "datasets": [{
                "name": "Sales",
                "values": [d.get("grand_total") for d in data[:10]]
            }]
        },
        "type": "bar",
        "colors": ["#7cd6fd"]
    }

def get_report_summary(data):
    """Generate summary cards"""
    if not data:
        return []

    total_sales = sum(d.get("grand_total", 0) for d in data)
    total_orders = len(data)
    avg_order_value = total_sales / total_orders if total_orders else 0

    return [
        {
            "value": total_orders,
            "label": _("Total Orders"),
            "datatype": "Int",
            "color": "blue"
        },
        {
            "value": total_sales,
            "label": _("Total Sales"),
            "datatype": "Currency",
            "color": "green"
        },
        {
            "value": avg_order_value,
            "label": _("Average Order Value"),
            "datatype": "Currency",
            "color": "orange"
        }
    ]
```

#### Step 3: Create JavaScript Configuration (Optional)
Location: `{app}/{module}/report/{report_name}/{report_name}.js`

```javascript
// Copyright (c) 2024, Your Company and contributors
// For license information, please see license.txt

frappe.query_reports["My Custom Report"] = {
    "filters": [
        {
            "fieldname": "company",
            "label": __("Company"),
            "fieldtype": "Link",
            "options": "Company",
            "default": frappe.defaults.get_user_default("Company"),
            "reqd": 1
        },
        {
            "fieldname": "from_date",
            "label": __("From Date"),
            "fieldtype": "Date",
            "default": frappe.datetime.add_months(frappe.datetime.get_today(), -1),
            "reqd": 1
        },
        {
            "fieldname": "to_date",
            "label": __("To Date"),
            "fieldtype": "Date",
            "default": frappe.datetime.get_today(),
            "reqd": 1
        },
        {
            "fieldname": "customer",
            "label": __("Customer"),
            "fieldtype": "Link",
            "options": "Customer",
            "get_query": function() {
                return {
                    "filters": {
                        "disabled": 0
                    }
                };
            }
        },
        {
            "fieldname": "status",
            "label": __("Status"),
            "fieldtype": "MultiSelect",
            "options": "Draft\nTo Deliver and Bill\nTo Bill\nTo Deliver\nCompleted\nCancelled\nClosed",
            "default": "To Deliver and Bill,To Bill,To Deliver"
        }
    ],

    "formatter": function(value, row, column, data, default_formatter) {
        value = default_formatter(value, row, column, data);

        if (column.fieldname == "status") {
            if (value == "Completed") {
                value = "<span class='text-success'>" + value + "</span>";
            } else if (value == "Cancelled") {
                value = "<span class='text-danger'>" + value + "</span>";
            }
        }

        return value;
    },

    onload: function(report) {
        // Custom initialization code
        report.page.add_inner_button(__("Export to Excel"), function() {
            frappe.tools.downloadify(report.get_data_for_csv(), null, report.report_name);
        });
    }
};
```

### Method 3: Query Report

For Query Reports, you only need to define the SQL query in the Report document:

```sql
-- In the Report's "Query" field
SELECT
    so.name as "Sales Order:Link/Sales Order:200",
    so.customer as "Customer:Link/Customer:150",
    so.transaction_date as "Date:Date:100",
    so.grand_total as "Total:Currency:120",
    so.status as "Status:Data:100"
FROM `tabSales Order` so
WHERE
    so.docstatus = 1
    AND so.company = %(company)s
    AND so.transaction_date BETWEEN %(from_date)s AND %(to_date)s
ORDER BY so.transaction_date DESC
```

## Report Structure

### Column Definition Format

Columns can be defined in multiple formats:

#### 1. Dictionary Format (Recommended for Script Reports)
```python
{
    "label": "Field Label",           # Display name
    "fieldname": "field_key",         # Key for data mapping
    "fieldtype": "Data",               # Frappe fieldtype
    "options": "DocType",              # For Link fields
    "width": 120,                      # Column width in pixels
    "precision": 2,                    # For float/currency
    "hidden": 0                        # Hide by default
}
```

#### 2. String Format (Simple, used in Query Reports)
```python
"Field Label:Fieldtype/Options:Width"
# Examples:
"Customer:Link/Customer:200"
"Amount:Currency:120"
"Status:Data:100"
```

### Supported Field Types
- **Data**: Plain text
- **Link**: Link to another DocType
- **Currency**: Monetary values
- **Float**: Decimal numbers
- **Int**: Integer numbers
- **Date**: Date picker
- **Datetime**: Date and time picker
- **Check**: Boolean checkbox
- **Select**: Dropdown selection
- **Dynamic Link**: Dynamic DocType reference

### Data Format

Data can be returned in two formats:

#### 1. List of Lists
```python
data = [
    ["SO-001", "Customer A", 1000.00, "Completed"],
    ["SO-002", "Customer B", 1500.00, "Draft"]
]
```

#### 2. List of Dictionaries (Recommended)
```python
data = [
    {
        "name": "SO-001",
        "customer": "Customer A",
        "grand_total": 1000.00,
        "status": "Completed"
    }
]
```

## Permissions and Security

### Role-Based Access
Reports use role-based permissions defined in the Report document (frappe/core/doctype/report/report.py:110-125):

```python
# In Report DocType
roles = [
    {"role": "Sales User"},
    {"role": "Sales Manager"}
]
```

### DocType Permissions
Users must have "Report" permission on the referenced DocType:
```python
if not frappe.has_permission(doc.ref_doctype, "report"):
    frappe.throw(_("Insufficient permissions"))
```

### Script Security
- Query Reports validate SQL queries for safety (frappe/core/doctype/report/report.py:149)
- Script Reports require "Script Manager" role for non-standard reports
- Custom code execution uses `safe_exec` sandbox environment

## Advanced Features

### 1. Prepared Reports
For long-running reports, enable prepared reports to run in background:

```python
# In Report settings
prepared_report = 1
timeout = 600  # seconds
```

Automatic detection: Reports taking >15 seconds automatically suggest prepared mode (frappe/core/doctype/report/report.py:163-169)

### 2. Report Summary Cards
Display key metrics above the report:

```python
report_summary = [
    {
        "value": total_value,
        "label": "Total Sales",
        "datatype": "Currency",
        "color": "green",
        "indicator": "Green" if growth > 0 else "Red"
    }
]
```

### 3. Charts Integration
Add visualizations to reports:

```python
chart = {
    "data": {
        "labels": ["Jan", "Feb", "Mar"],
        "datasets": [{
            "name": "Sales",
            "values": [100, 200, 150]
        }]
    },
    "type": "line",  # bar, pie, percentage
    "colors": ["#7cd6fd", "#ffa3ef", "#ff5858"]
}
```

### 4. Tree Reports
For hierarchical data display:

```javascript
frappe.query_reports["My Report"] = {
    "tree": true,
    "parent_field": "parent_account",
    "initial_depth": 2
}
```

### 5. Custom Formatters
Apply custom formatting to cells:

```javascript
"formatter": function(value, row, column, data, default_formatter) {
    value = default_formatter(value, row, column, data);

    if (column.fieldname == "amount" && value < 0) {
        value = "<span class='text-danger'>" + value + "</span>";
    }

    return value;
}
```

### 6. Drill-Down Reports
Enable navigation to detailed views:

```python
# In column definition
{
    "label": "Invoice",
    "fieldname": "invoice",
    "fieldtype": "Link",
    "options": "Sales Invoice",
    "width": 150
}
```

### 7. Export Options
Reports automatically support multiple export formats:
- Excel
- CSV
- PDF (if letter_head is configured)

### 8. Filters with Dependencies
Create dynamic filter relationships:

```javascript
{
    "fieldname": "item",
    "label": "Item",
    "fieldtype": "Link",
    "options": "Item",
    "get_query": function() {
        let item_group = frappe.query_report.get_filter_value("item_group");
        return {
            "filters": {
                "item_group": item_group
            }
        };
    }
}
```

## Best Practices

### 1. Performance Optimization
- **Use indexes**: Ensure filtered fields have database indexes
- **Limit results**: Add pagination or limits for large datasets
- **Optimize queries**: Use EXPLAIN to analyze SQL performance
- **Cache when possible**: Use `frappe.cache` for expensive calculations

```python
@frappe.whitelist()
def get_cached_data():
    return frappe.cache().get_value("report_data",
        generator=lambda: expensive_calculation())
```

### 2. Error Handling
```python
def execute(filters=None):
    try:
        validate_filters(filters)
        columns = get_columns()
        data = get_data(filters)
    except Exception as e:
        frappe.log_error(f"Report error: {str(e)}")
        frappe.throw(_("Error generating report: {0}").format(str(e)))

    return columns, data
```

### 3. Localization
Always use translation functions:
```python
from frappe import _

columns = [
    {"label": _("Customer"), "fieldname": "customer"}
]
```

### 4. Security Best Practices
- **Never use string formatting for SQL**: Use parameterized queries
- **Validate user permissions**: Check both role and record-level permissions
- **Sanitize inputs**: Validate and clean filter values

```python
# Good - Parameterized query
frappe.db.sql("""
    SELECT * FROM `tabSales Order`
    WHERE customer = %s
""", (customer,))

# Bad - SQL injection risk
frappe.db.sql(f"""
    SELECT * FROM `tabSales Order`
    WHERE customer = '{customer}'
""")
```

### 5. Testing
Create test cases for reports:

```python
class TestMyReport(unittest.TestCase):
    def test_report_data(self):
        filters = {
            "company": "Test Company",
            "from_date": "2024-01-01",
            "to_date": "2024-12-31"
        }

        columns, data = execute(filters)

        self.assertTrue(len(columns) > 0)
        self.assertIsInstance(data, list)
```

## Examples

### Example 1: Simple Script Report
Location: frappe/desk/report/todo/todo.py:9-69

```python
def execute(filters=None):
    priority_map = {"High": 3, "Medium": 2, "Low": 1}

    todo_list = frappe.get_list(
        "ToDo",
        fields=["name", "date", "description", "priority", "owner"],
        filters={"status": "Open"}
    )

    # Sort by priority and date
    todo_list.sort(
        key=lambda x: (priority_map.get(x.priority, 0), x.date or "1900-01-01"),
        reverse=True
    )

    columns = [
        _("ID") + ":Link/ToDo:90",
        _("Priority") + "::60",
        _("Date") + ":Date",
        _("Description") + "::150",
        _("Owner") + ":Data:120"
    ]

    data = [[t.name, t.priority, t.date, t.description, t.owner]
            for t in todo_list]

    return columns, data
```

### Example 2: Report with Dynamic Filters
Location: frappe/contacts/report/addresses_and_contacts/addresses_and_contacts.js:4-33

```javascript
frappe.query_reports["Addresses And Contacts"] = {
    filters: [
        {
            reqd: 1,
            fieldname: "reference_doctype",
            label: __("Entity Type"),
            fieldtype: "Link",
            options: "DocType"
        },
        {
            fieldname: "reference_name",
            label: __("Entity Name"),
            fieldtype: "Dynamic Link",
            get_options: function() {
                return frappe.query_report.get_filter_value("reference_doctype");
            }
        }
    ]
};
```

### Example 3: Report with Permissions Check
Location: frappe/core/report/permitted_documents_for_user/permitted_documents_for_user.py:10-29

```python
def execute(filters=None):
    frappe.only_for("System Manager")  # Role check

    user = filters.get("user")
    doctype = filters.get("doctype")

    # Get documents user has access to
    data = frappe.get_list(doctype, fields=["*"], as_list=True, user=user)

    if filters.get("show_permissions"):
        # Add permission details for each document
        for i, doc in enumerate(data):
            permissions = frappe.permissions.get_doc_permissions(
                frappe.get_doc(doctype, doc[0]), user
            )
            data[i] = doc + tuple(permissions.values())

    return columns, data
```

## Directory Structure

Standard app structure for reports:

```
app_name/
├── module_name/
│   └── report/
│       └── report_name/
│           ├── __init__.py
│           ├── report_name.js       # Frontend filters and configuration
│           ├── report_name.json     # Report metadata
│           └── report_name.py       # Backend logic (Script Reports)
```

## Debugging Tips

### 1. Enable SQL Query Logging
```python
frappe.flags.print_sql = True
```

### 2. Debug Report Execution
```python
# In frappe console
from app.module.report.report_name.report_name import execute
columns, data = execute({"filter": "value"})
print(columns, data)
```

### 3. Check Report Permissions
```python
report = frappe.get_doc("Report", "Report Name")
print(report.is_permitted())  # Check if current user has access
```

### 4. Monitor Performance
Use `frappe.cache.hget("report_execution_time", report_name)` to check execution times.

## Migration and Deployment

### Exporting Reports
Standard reports are automatically exported when `is_standard = "Yes"` and developer mode is enabled:

```python
# Automatic export location:
# app/module/report/report_name/report_name.json
```

### Including in App
1. Set `is_standard = "Yes"`
2. Assign to appropriate module
3. Reports are automatically included in app migrations

## Conclusion

Frappe's reporting framework provides a flexible and powerful system for creating various types of reports. From simple Report Builder reports to complex Script Reports with charts and summaries, the framework handles diverse reporting needs while maintaining security and performance.

Key takeaways:
- Choose the right report type for your use case
- Follow security best practices
- Optimize for performance
- Leverage advanced features when needed
- Test thoroughly

For more examples, explore the existing reports in the frappe/core/report and frappe/desk/report directories.