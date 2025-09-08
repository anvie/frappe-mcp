# Creating DocTypes in Frappe

## What is a DocType?

A DocType is the core building block in Frappe Framework. It represents a data model with automatic CRUD operations, forms, and API endpoints.

## Creating a New DocType

### Method 1: Using Frappe UI

1. Go to the DocType List
2. Click "New"
3. Fill in the DocType details:
   - **Name**: Unique identifier (e.g., "Customer Order")
   - **Module**: The module this DocType belongs to
   - **Is Submittable**: Whether documents can be submitted
   - **Is Child Table**: For use as a child table only
   - **Is Single**: Only one document instance allowed
   - **Is Tree**: Hierarchical structure with parent-child relationships

### Method 2: Using JSON Definition

Create a JSON file in your app's doctype folder:

```json
{
 "creation": "2024-01-01 10:00:00.000000",
 "doctype": "DocType",
 "engine": "InnoDB",
 "field_order": [
  "customer_name",
  "order_date",
  "total_amount"
 ],
 "fields": [
  {
   "fieldname": "customer_name",
   "fieldtype": "Data",
   "label": "Customer Name",
   "reqd": 1
  },
  {
   "fieldname": "order_date",
   "fieldtype": "Date",
   "label": "Order Date",
   "default": "Today"
  },
  {
   "fieldname": "total_amount",
   "fieldtype": "Currency",
   "label": "Total Amount",
   "options": "USD"
  }
 ],
 "modified": "2024-01-01 10:00:00.000000",
 "modified_by": "Administrator",
 "module": "Sales",
 "name": "Customer Order",
 "owner": "Administrator",
 "permissions": [
  {
   "create": 1,
   "delete": 1,
   "read": 1,
   "role": "System Manager",
   "write": 1
  }
 ]
}
```

### Method 3: Using Bench Commands

```bash
bench --site yoursite.local new-doctype "Customer Order"
```

## DocType Options

### Naming Options
- **Autoname**: Pattern for automatic naming (e.g., "ORD-.#####")
- **Name Case**: Title Case or UPPER CASE
- **Allow Rename**: Whether documents can be renamed

### Behavior Options
- **Track Changes**: Log all changes to documents
- **Track Views**: Track when documents are viewed
- **Track Seen**: Show unread indicators
- **Max Attachments**: Limit number of file attachments

### Database Options
- **Engine**: InnoDB (default) or MyISAM
- **Is Virtual**: DocType without database table
- **Queue in Background**: Process saves asynchronously

## Controller Methods

Create a Python controller file alongside your DocType:

```python
import frappe
from frappe.model.document import Document

class CustomerOrder(Document):
    def validate(self):
        # Called before saving
        self.validate_order_date()
        
    def on_submit(self):
        # Called when document is submitted
        self.create_delivery_note()
        
    def on_cancel(self):
        # Called when document is cancelled
        self.cancel_linked_documents()
    
    def validate_order_date(self):
        if self.order_date < frappe.utils.today():
            frappe.throw("Order date cannot be in the past")
```

## Permissions

Set granular permissions for different roles:

```python
# In hooks.py
permission_query_conditions = {
    "Customer Order": "frappe.db.get_value('Customer Order', doc.name, 'owner') == frappe.session.user"
}
```

## Best Practices

1. **Naming Convention**: Use clear, descriptive names
2. **Module Organization**: Group related DocTypes in modules
3. **Field Order**: Place important fields at the top
4. **Mandatory Fields**: Mark essential fields as required
5. **Validation**: Implement proper validation in controllers
6. **Permissions**: Set appropriate role-based permissions
7. **Documentation**: Add descriptions to fields and DocTypes