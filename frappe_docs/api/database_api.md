# Frappe Database API

## Overview

Frappe provides a powerful database abstraction layer through `frappe.db` that handles SQL generation, query execution, and result processing.

## Basic Operations

### Get Single Value

```python
# Get a single field value
value = frappe.db.get_value("Customer", "CUST-0001", "customer_name")

# Get multiple fields
values = frappe.db.get_value("Customer", "CUST-0001", ["customer_name", "email", "phone"])

# With filters
value = frappe.db.get_value("Customer", {"email": "john@example.com"}, "name")
```

### Get List of Records

```python
# Simple list
customers = frappe.db.get_list("Customer",
    fields=["name", "customer_name"],
    filters={"disabled": 0},
    order_by="creation desc",
    limit=10
)

# With complex filters
orders = frappe.db.get_list("Sales Order",
    filters={
        "status": ["in", ["Draft", "Submitted"]],
        "grand_total": [">", 1000],
        "transaction_date": ["between", ["2024-01-01", "2024-12-31"]]
    }
)
```

### Get Single Document

```python
# Get complete document
doc = frappe.db.get_doc("Customer", "CUST-0001")

# Get with specific fields
customer = frappe.db.get("Customer", "CUST-0001", ["customer_name", "email"])
```

### Check Existence

```python
# Check if record exists
exists = frappe.db.exists("Customer", "CUST-0001")

# With filters
exists = frappe.db.exists("Customer", {"email": "john@example.com"})
```

## Insert Operations

### Insert Single Record

```python
# Insert new record
frappe.db.insert({
    "doctype": "Customer",
    "customer_name": "John Doe",
    "customer_type": "Individual",
    "email": "john@example.com"
})

# With commit
doc = frappe.get_doc({
    "doctype": "Customer",
    "customer_name": "Jane Doe"
}).insert()
frappe.db.commit()
```

### Bulk Insert

```python
# Insert multiple records
records = [
    {"customer_name": "Customer 1", "email": "cust1@example.com"},
    {"customer_name": "Customer 2", "email": "cust2@example.com"}
]

for record in records:
    record["doctype"] = "Customer"
    frappe.get_doc(record).insert()
```

## Update Operations

### Set Value

```python
# Update single field
frappe.db.set_value("Customer", "CUST-0001", "status", "Active")

# Update multiple fields
frappe.db.set_value("Customer", "CUST-0001", {
    "status": "Active",
    "credit_limit": 50000
})

# Update with filters
frappe.db.set_value("Customer", {"email": "john@example.com"}, "status", "Inactive")
```

### SQL Update

```python
# Direct SQL update
frappe.db.sql("""
    UPDATE `tabCustomer`
    SET credit_limit = credit_limit * 1.1
    WHERE customer_group = 'Premium'
""")
```

## Delete Operations

### Delete Records

```python
# Delete single record
frappe.db.delete("Customer", "CUST-0001")

# Delete with filters
frappe.db.delete("Customer", {"status": "Inactive"})

# Delete multiple
customers = ["CUST-0001", "CUST-0002", "CUST-0003"]
for name in customers:
    frappe.db.delete("Customer", name)
```

## Raw SQL Queries

### Execute SQL

```python
# Select query
result = frappe.db.sql("""
    SELECT name, customer_name, grand_total
    FROM `tabSales Order`
    WHERE status = 'Submitted'
    AND transaction_date >= %s
""", "2024-01-01", as_dict=True)

# With named parameters
result = frappe.db.sql("""
    SELECT * FROM `tabCustomer`
    WHERE customer_group = %(group)s
    AND credit_limit > %(limit)s
""", {"group": "Premium", "limit": 10000}, as_dict=True)
```

### Get Single Value from SQL

```python
count = frappe.db.sql("""
    SELECT COUNT(*) FROM `tabSales Order`
    WHERE status = 'Submitted'
""")[0][0]
```

## Advanced Queries

### Aggregation

```python
# Count
count = frappe.db.count("Customer", filters={"disabled": 0})

# Sum
total = frappe.db.sql("""
    SELECT SUM(grand_total)
    FROM `tabSales Order`
    WHERE status = 'Submitted'
""")[0][0] or 0

# Group By
result = frappe.db.sql("""
    SELECT customer, SUM(grand_total) as total
    FROM `tabSales Order`
    WHERE docstatus = 1
    GROUP BY customer
    ORDER BY total DESC
""", as_dict=True)
```

### Joins

```python
# Inner join
result = frappe.db.sql("""
    SELECT
        so.name,
        so.customer,
        c.customer_name,
        so.grand_total
    FROM `tabSales Order` so
    INNER JOIN `tabCustomer` c ON so.customer = c.name
    WHERE so.status = 'Submitted'
""", as_dict=True)

# Left join with child table
items = frappe.db.sql("""
    SELECT
        so.name as order_id,
        soi.item_code,
        soi.qty,
        soi.rate
    FROM `tabSales Order` so
    LEFT JOIN `tabSales Order Item` soi ON soi.parent = so.name
    WHERE so.customer = %s
""", customer_name, as_dict=True)
```

## Transaction Management

### Commit and Rollback

```python
try:
    # Start transaction
    frappe.db.begin()

    # Perform operations
    frappe.db.set_value("Customer", "CUST-0001", "credit_limit", 100000)
    frappe.db.insert({"doctype": "Note", "title": "Credit limit updated"})

    # Commit transaction
    frappe.db.commit()
except Exception as e:
    # Rollback on error
    frappe.db.rollback()
    frappe.throw(str(e))
```

### Auto Commit

```python
# Disable auto-commit
frappe.db.auto_commit_on_many_writes = 0

# Bulk operations
for i in range(1000):
    frappe.db.insert({...})

# Manual commit
frappe.db.commit()
```

## Utility Functions

### Get Table Columns

```python
# Get column names
columns = frappe.db.get_table_columns("Customer")
```

### Get Database Size

```python
# Get table size
size = frappe.db.sql("""
    SELECT
        ROUND(((data_length + index_length) / 1024 / 1024), 2) AS size_mb
    FROM information_schema.TABLES
    WHERE table_schema = DATABASE()
    AND table_name = 'tabSales Order'
""")[0][0]
```

### Escape Values

```python
# Escape string for SQL
escaped = frappe.db.escape("John's Shop")

# Build safe query
query = f"SELECT * FROM `tabCustomer` WHERE customer_name = {escaped}"
```

## Best Practices

1. **Use ORM Methods**: Prefer `get_list`, `get_value` over raw SQL
2. **Parameterized Queries**: Always use parameters to prevent SQL injection
3. **Transaction Management**: Use transactions for multi-step operations
4. **Error Handling**: Wrap database operations in try-except blocks
5. **Indexing**: Ensure proper indexes for frequently queried columns
6. **Limit Results**: Always use LIMIT for large datasets
7. **Avoid N+1**: Fetch related data in single query using joins

## Common Patterns

### Pagination

```python
def get_paginated_data(doctype, page=1, page_size=20):
    start = (page - 1) * page_size

    data = frappe.db.get_list(doctype,
        fields=["*"],
        limit=page_size,
        start=start,
        order_by="creation desc"
    )

    total = frappe.db.count(doctype)

    return {
        "data": data,
        "total": total,
        "page": page,
        "pages": math.ceil(total / page_size)
    }
```

### Bulk Update with Progress

```python
def bulk_update_customers():
    customers = frappe.db.get_list("Customer", pluck="name")

    for i, customer in enumerate(customers):
        # Update logic
        frappe.db.set_value("Customer", customer, "updated", 1)

        # Commit every 100 records
        if i % 100 == 0:
            frappe.db.commit()
            frappe.publish_progress(i / len(customers) * 100)
```

