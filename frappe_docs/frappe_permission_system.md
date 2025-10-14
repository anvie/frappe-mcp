# Frappe Permission System Documentation

## Table of Contents

1. [Overview](#overview)
1. [Architecture](#architecture)
1. [Permission Types](#permission-types)
1. [Core Concepts](#core-concepts)
1. [Implementation Guide](#implementation-guide)
1. [API Reference](#api-reference)
1. [Advanced Topics](#advanced-topics)
1. [Examples](#examples)
1. [Best Practices](#best-practices)
1. [Troubleshooting](#troubleshooting)

## Overview

The Frappe Permission System is a sophisticated, multi-layered access control
framework that provides granular control over who can access and perform actions
on documents. It combines role-based permissions, user-specific restrictions,
document ownership rules, and custom permission logic to create a flexible yet
secure authorization system.

### Key Features

- **Role-Based Access Control (RBAC)**: Assign permissions based on user roles
- **User Permissions**: Restrict users to specific records
- **Permission Levels**: Control field-level access within documents
- **Document Ownership**: Special permissions for document owners
- **Document Sharing**: Explicitly share documents with specific users
- **Custom Permission Logic**: Implement complex business rules via hooks

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Permission Check Flow                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  1. Administrator Check ──► Bypass all checks if Admin       │
│           │                                                   │
│           ▼                                                   │
│  2. Role-Based Permissions ──► Check DocPerm/Custom DocPerm  │
│           │                                                   │
│           ▼                                                   │
│  3. User Permissions ──► Apply user-specific restrictions    │
│           │                                                   │
│           ▼                                                   │
│  4. Owner Permissions ──► Apply if_owner rules               │
│           │                                                   │
│           ▼                                                   │
│  5. Controller Hooks ──► Execute custom permission logic     │
│           │                                                   │
│           ▼                                                   │
│  6. Document Sharing ──► Check explicit shares               │
│           │                                                   │
│           ▼                                                   │
│  7. Final Permission Decision                                │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### File Structure

- `/app/frappe/permissions.py` - Core permission logic
- `/app/frappe/model/document.py` - Document-level permission integration
- `/app/frappe/model/db_query.py` - Database query permission filters
- `/app/frappe/core/doctype/docperm/` - DocPerm model
- `/app/frappe/core/doctype/user_permission/` - User Permission model
- `/app/frappe/core/page/permission_manager/` - Permission Manager UI

## Permission Types

Frappe supports 14 different permission types, each controlling specific
actions:

### Basic Permissions

| Permission | Description                | Use Case                                                                   |
| ---------- | -------------------------- | -------------------------------------------------------------------------- |
| **select** | View records in list views | Allow users to see records in dropdowns and lists without full read access |
| **read**   | View document details      | Allow users to open and view complete documents                            |
| **write**  | Edit existing documents    | Allow users to modify documents but not create new ones                    |
| **create** | Create new documents       | Allow users to create new records                                          |
| **delete** | Delete documents           | Allow users to permanently remove documents                                |
| **submit** | Submit documents           | Allow users to submit documents (for submittable DocTypes)                 |
| **cancel** | Cancel submitted documents | Allow users to cancel previously submitted documents                       |
| **amend**  | Amend cancelled documents  | Allow users to create amended versions of cancelled documents              |

### Additional Permissions

| Permission | Description     | Use Case                                         |
| ---------- | --------------- | ------------------------------------------------ |
| **print**  | Print documents | Control who can generate print formats           |
| **email**  | Email documents | Control who can send documents via email         |
| **report** | Access reports  | Control access to report generation              |
| **import** | Import data     | Control who can bulk import data                 |
| **export** | Export data     | Control who can export data to files             |
| **share**  | Share documents | Control who can share documents with other users |

## Core Concepts

### 1. DocPerm (Document Permissions)

DocPerm defines role-based permissions for a DocType. Each DocPerm entry
contains:

```python
class DocPerm:
    role: str  # Role name (e.g., "Sales User")
    permlevel: int  # Permission level (0 for document, 1+ for fields)
    read: bool  # Can read documents
    write: bool  # Can edit documents
    create: bool  # Can create documents
    delete: bool  # Can delete documents
    submit: bool  # Can submit documents
    cancel: bool  # Can cancel documents
    amend: bool  # Can amend documents
    if_owner: bool  # Apply only if user owns the document
    # ... other permission types
```

### 2. User Permissions

User Permissions restrict users to specific records of a DocType:

```python
class UserPermission:
    user: str  # User email
    allow: str  # DocType to restrict
    for_value: str  # Specific value/record
    applicable_for: str  # Apply to specific DocType only
    apply_to_all_doctypes: bool  # Apply across all DocTypes
    is_default: bool  # Use as default value
    hide_descendants: bool  # Hide child records in tree structures
```

### 3. Permission Levels (permlevel)

Permission levels control field-level access:

- **Level 0**: Standard document access
- **Level 1+**: Restricted fields requiring higher permissions

Example:

```python
# In DocType field definition
{
    "fieldname": "gross_profit",
    "label": "Gross Profit",
    "fieldtype": "Currency",
    "permlevel": 1,  # Requires Level 1 permission to read/write
}
```

### 4. If Owner Permissions

Special permissions that apply only to document owners:

```python
# DocPerm with if_owner
{
    "role": "Employee",
    "read": 1,
    "write": 0,
    "delete": 0,
    "if_owner": 1,  # These permissions only apply to owner
}
```

### 5. Automatic Roles

Frappe automatically assigns these roles based on user type:

- **Guest**: Non-logged-in users
- **All**: All users (including website users)
- **Desk User**: Users with system access
- **Administrator**: System administrators

## Implementation Guide

### Setting Up Basic Permissions

#### 1. Using Permission Manager UI

```python
# Navigate to Permission Manager
# Setup > Permissions > Permission Manager

# Select DocType and Role
# Configure permission checkboxes
# Save changes
```

#### 2. Programmatic Permission Setup

```python
from frappe.permissions import add_permission, update_permission_property

# Add new permission
add_permission("Customer", "Sales User", 0)

# Update specific permission property
update_permission_property("Customer", "Sales User", 0, "write", 1)
update_permission_property("Customer", "Sales User", 0, "create", 1)
```

### Adding User Permissions

```python
from frappe.permissions import add_user_permission

# Restrict user to specific company
add_user_permission(
    "Company",
    "ACME Corp",
    "john@example.com",
    applicable_for="Sales Order",  # Optional: Apply only to Sales Orders
)

# Set default value
add_user_permission("Warehouse", "Main Warehouse", "john@example.com", is_default=1)
```

### Custom Permission Hooks

#### 1. Controller-Based Permissions

In your DocType's controller:

```python
class Customer(Document):
    def has_permission(self, ptype="read", user=None):
        """Custom permission logic"""
        if not user:
            user = frappe.session.user

        # Example: Sales managers can always read
        if ptype == "read" and "Sales Manager" in frappe.get_roles(user):
            return True

        # Example: Only owner can delete
        if ptype == "delete" and self.owner != user:
            return False

        # Fall back to standard permission check
        return None  # Return None to continue standard checks
```

#### 2. Permission Query Conditions

Control which records appear in list views:

```python
def get_permission_query_conditions(user):
    """Return SQL conditions to filter records"""
    if not user:
        user = frappe.session.user

    if "Sales Manager" in frappe.get_roles(user):
        # Sales managers see all records
        return ""

    # Others see only their own records
    return f"(`tabCustomer`.owner = {frappe.db.escape(user)})"
```

#### 3. Match Conditions (Deprecated)

```python
def get_match_conditions(doctype, user):
    """Legacy method - use get_permission_query_conditions instead"""
    conditions = []

    if not "Sales Manager" in frappe.get_roles(user):
        conditions.append(f"`owner` = {frappe.db.escape(user)}")

    return " and ".join(conditions) if conditions else ""
```

## API Reference

### Core Functions

#### has_permission()

Check if user has specific permission for a DocType/document:

```python
frappe.has_permission(
    doctype: str,
    ptype: str = "read",
    doc: Document = None,
    user: str = None,
    raise_exception: bool = True,
    parent_doctype: str = None,
    debug: bool = False
) -> bool

# Examples
frappe.has_permission("Customer", ptype="write")
frappe.has_permission(doc=customer_doc, ptype="delete")
```

#### get_doc_permissions()

Get all permissions for a specific document:

```python
from frappe.permissions import get_doc_permissions

permissions = get_doc_permissions(doc, user="john@example.com")
# Returns: {"read": 1, "write": 0, "delete": 0, ...}
```

#### get_role_permissions()

Get permissions based on user roles:

```python
from frappe.permissions import get_role_permissions

perms = get_role_permissions(doctype_meta, user="john@example.com", is_owner=True)
```

### User Permission Functions

#### add_user_permission()

```python
from frappe.permissions import add_user_permission

add_user_permission(
    doctype: str,
    name: str,
    user: str,
    ignore_permissions: bool = False,
    applicable_for: str = None,
    is_default: bool = False,
    hide_descendants: bool = False,
    apply_to_all_doctypes: bool = True
)
```

#### remove_user_permission()

```python
from frappe.permissions import remove_user_permission

remove_user_permission(
    doctype: str,
    name: str,
    user: str
)
```

#### clear_user_permissions()

```python
from frappe.permissions import clear_user_permissions

clear_user_permissions(user="john@example.com")
clear_user_permissions_for_doctype("Customer", user="john@example.com")
```

### Document Sharing

#### Share a document

```python
frappe.share.add_docshare(
    doctype: str,
    name: str,
    user: str,
    read: int = 0,
    write: int = 0,
    share: int = 0,
    notify: int = 1
)
```

#### Check shared documents

```python
frappe.share.get_shared(
    doctype: str,
    user: str,
    rights: list = None
)
```

## Advanced Topics

### Multi-Level Permissions

Implement field-level security using permission levels:

```python
# In DocType definition
def get_field_perms():
    return [
        {"fieldname": "basic_salary", "permlevel": 1},  # Requires Level 1 permission
        {"fieldname": "bank_account", "permlevel": 2},  # Requires Level 2 permission
    ]


# Grant level-specific permissions
add_permission("Employee", "HR User", permlevel=0)
add_permission("Employee", "HR Manager", permlevel=1)
add_permission("Employee", "Payroll Manager", permlevel=2)
```

### Dynamic Permissions with Server Scripts

Create runtime permission rules:

```python
# Server Script for Permission Query
conditions = []

# Check custom conditions
if frappe.db.get_value("Employee", {"user_id": user}, "department") == "Sales":
    conditions.append("department = 'Sales'")

return " AND ".join(conditions) if conditions else ""
```

### Permission Caching

Frappe caches permissions for performance:

```python
# Clear permission cache
frappe.clear_cache(doctype="Customer")

# Clear all caches
frappe.clear_cache()

# Clear user-specific cache
frappe.clear_cache(user="john@example.com")
```

### Bypassing Permissions

For system operations:

```python
# Temporarily bypass permissions
frappe.set_user("Administrator")
# Perform operations
frappe.set_user(original_user)

# Or use context manager
with frappe.set_user("Administrator"):
    # Operations with admin privileges
    pass

# Or use flags
doc.flags.ignore_permissions = True
doc.save()
```

## Examples

### Example 1: Department-Based Access

Restrict employees to view only their department's documents:

```python
# In Employee DocType controller
def get_permission_query_conditions(user):
    employee = frappe.db.get_value("Employee", {"user_id": user}, "department")
    if employee:
        return f"`tabEmployee`.department = {frappe.db.escape(employee)}"
    return "1=0"  # No access if not an employee
```

### Example 2: Hierarchical Permissions

Manager can see their team's documents:

```python
class Employee(Document):
    def has_permission(self, ptype="read", user=None):
        if ptype == "read":
            # Get user's employee record
            employee = frappe.db.get_value("Employee", {"user_id": user}, "name")

            # Check if current employee reports to this user
            if self.reports_to == employee:
                return True

        return None  # Continue with standard checks
```

### Example 3: Time-Based Permissions

Allow editing only during business hours:

```python
from datetime import datetime


class TimeSheet(Document):
    def has_permission(self, ptype="write", user=None):
        if ptype == "write":
            current_hour = datetime.now().hour
            if not (9 <= current_hour < 18):
                frappe.throw("Editing allowed only during business hours (9 AM - 6 PM)")
        return None
```

### Example 4: Workflow-Based Permissions

Permissions based on document status:

```python
class PurchaseOrder(Document):
    def has_permission(self, ptype="write", user=None):
        # Only allow editing draft documents
        if ptype == "write" and self.docstatus != 0:
            return False

        # Only approvers can submit
        if ptype == "submit":
            if "Purchase Approver" not in frappe.get_roles(user):
                return False

        return None
```

### Example 5: Complex User Permissions

Multi-company restriction with exceptions:

```python
# Setup user permissions
from frappe.permissions import add_user_permission

# Restrict to Company A for most doctypes
add_user_permission("Company", "Company A", "john@example.com", apply_to_all_doctypes=1)

# Allow access to all warehouses regardless of company
add_user_permission(
    "Warehouse", "Main Warehouse", "john@example.com", applicable_for="Stock Entry"
)
```

## Best Practices

### 1. Security Guidelines

- **Principle of Least Privilege**: Grant only necessary permissions
- **Regular Audits**: Periodically review permission configurations
- **Test Permissions**: Always test permission changes with test users
- **Document Custom Logic**: Clearly document any custom permission
  implementations

### 2. Performance Optimization

```python
# Good: Use permission levels for field restrictions
{"fieldname": "sensitive_data", "permlevel": 1}

# Bad: Checking permissions in validate method for each field
def validate(self):
    if not frappe.has_permission(self.doctype, "write", permlevel=1):
        self.sensitive_data = None  # Inefficient
```

### 3. User Permission Best Practices

- **Avoid Too Many User Permissions**: Can slow down queries
- **Use `applicable_for`**: Limit scope when possible
- **Set Defaults Carefully**: Only one default per DocType per user
- **Consider Hierarchy**: Use `hide_descendants` for tree structures

### 4. Custom Permission Logic

```python
# Good: Return None to continue standard checks
def has_permission(self, ptype="read", user=None):
    if special_condition:
        return True  # Grant access
    return None  # Continue standard checks


# Bad: Always return boolean
def has_permission(self, ptype="read", user=None):
    return True  # Bypasses all standard checks!
```

### 5. Permission Debugging

```python
# Enable debug mode for detailed permission logs
has_permission = frappe.has_permission(
    "Customer", ptype="write", doc=customer_doc, debug=True  # Enables detailed logging
)

# Check permission logs
print(frappe.local.permission_debug_log)
```

## Troubleshooting

### Common Issues and Solutions

#### 1. User Can't Access Documents Despite Having Role

**Possible Causes:**

- User Permissions restricting access
- Document ownership issues
- Permission level mismatch

**Debug Steps:**

```python
# Check effective permissions
from frappe.permissions import get_doc_permissions

perms = get_doc_permissions(doc, user="user@example.com")
print(perms)

# Check user permissions
user_perms = frappe.get_all("User Permission", filters={"user": "user@example.com"})
print(user_perms)
```

#### 2. Permissions Not Taking Effect

**Solution:**

```python
# Clear all permission caches
frappe.clear_cache()

# Or specifically for a doctype
frappe.clear_cache(doctype="Customer")

# Reload metadata
frappe.reload_doc("module", "doctype", "customer")
```

#### 3. User Sees All Records Despite Restrictions

**Check for:**

- Administrator role
- System Manager role with elevated permissions
- Missing `get_permission_query_conditions` implementation
- Ignored user permissions in queries

#### 4. Performance Issues with Permissions

**Optimization Strategies:**

```python
# Use select permission for dropdowns
update_permission_property("Customer", "Sales User", 0, "select", 1)
update_permission_property("Customer", "Sales User", 0, "read", 0)

# Optimize user permissions
# Instead of many specific permissions, use patterns
add_user_permission("Company", "ACME%", "user@example.com")  # If supported

# Cache permission checks
if not hasattr(frappe.local, "permission_cache"):
    frappe.local.permission_cache = {}

cache_key = (doctype, ptype, user)
if cache_key not in frappe.local.permission_cache:
    frappe.local.permission_cache[cache_key] = frappe.has_permission(...)
```

#### 5. Debugging Permission Denials

```python
# Enable detailed logging
frappe.flags["has_permission_check_logs"] = []

# Check permission with raise_exception
try:
    frappe.has_permission("Customer", ptype="write", raise_exception=True)
except frappe.PermissionError as e:
    print(frappe.flags.get("has_permission_check_logs", []))
```

### Permission Check Order

When debugging, remember the permission check order:

1. **Administrator** → Always allowed
1. **DocPerm Check** → Role-based permissions
1. **User Permissions** → User-specific restrictions
1. **Owner Check** → if_owner rules
1. **Controller Hook** → Custom has_permission method
1. **Share Check** → Explicit document shares

### Useful SQL Queries for Debugging

```sql
-- Check user's roles
SELECT * FROM `tabHas Role` WHERE parent = 'user@example.com';

-- Check DocPerm for a DocType
SELECT * FROM `tabDocPerm` WHERE parent = 'Customer';

-- Check Custom DocPerm (overrides standard)
SELECT * FROM `tabCustom DocPerm` WHERE parent = 'Customer';

-- Check User Permissions
SELECT * FROM `tabUser Permission` WHERE user = 'user@example.com';

-- Check document shares
SELECT * FROM `tabDocShare` WHERE user = 'user@example.com';
```

## Summary

The Frappe Permission System provides a comprehensive and flexible framework for
controlling access to documents and operations. By understanding and properly
implementing its various components—from basic role permissions to advanced
custom logic—you can create secure, performant, and maintainable applications.

### Key Takeaways

1. **Layer Your Permissions**: Use role-based permissions as the foundation, add
   user permissions for restrictions, and implement custom logic only when
   necessary.

1. **Performance Matters**: Be mindful of query performance, especially with
   user permissions. Use select permissions and caching strategically.

1. **Test Thoroughly**: Always test permission changes with actual user
   accounts, not just as Administrator.

1. **Document Custom Logic**: Any custom permission implementation should be
   well-documented for future maintenance.

1. **Regular Audits**: Periodically review and audit permission configurations
   to ensure they align with business requirements and security policies.

______________________________________________________________________

_This documentation is based on the Frappe version 15 permission system
implementation. For the latest updates and changes, refer to the official Frappe
documentation and source code._
