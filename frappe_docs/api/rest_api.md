# Frappe REST API

## Overview

Frappe automatically generates REST API endpoints for all DocTypes. The API supports CRUD operations, filtering, pagination, and custom methods.

## Authentication

### Token Based
```bash
# Get API keys from User Settings
curl -X POST "https://site.com/api/method/frappe.auth.get_logged_user" \
  -H "Authorization: token api_key:api_secret"
```

### Password Based
```bash
# Login and get session
curl -X POST "https://site.com/api/method/login" \
  -d "usr=admin@example.com" \
  -d "pwd=password"
```

## Standard CRUD Operations

### Create (POST)
```bash
# Create new document
curl -X POST "https://site.com/api/resource/Customer" \
  -H "Authorization: token api_key:api_secret" \
  -H "Content-Type: application/json" \
  -d '{
    "customer_name": "John Doe",
    "customer_type": "Individual",
    "email": "john@example.com"
  }'
```

### Read (GET)
```bash
# Get single document
curl "https://site.com/api/resource/Customer/CUST-0001" \
  -H "Authorization: token api_key:api_secret"

# Get list with filters
curl "https://site.com/api/resource/Customer?filters=[[\"disabled\",\"=\",0]]&fields=[\"name\",\"customer_name\"]&limit=20" \
  -H "Authorization: token api_key:api_secret"
```

### Update (PUT)
```bash
# Update document
curl -X PUT "https://site.com/api/resource/Customer/CUST-0001" \
  -H "Authorization: token api_key:api_secret" \
  -H "Content-Type: application/json" \
  -d '{
    "customer_name": "John Smith",
    "credit_limit": 50000
  }'
```

### Delete (DELETE)
```bash
# Delete document
curl -X DELETE "https://site.com/api/resource/Customer/CUST-0001" \
  -H "Authorization: token api_key:api_secret"
```

## Query Parameters

### Fields Selection
```bash
# Select specific fields
GET /api/resource/Customer?fields=["name","customer_name","email"]
```

### Filtering
```bash
# Simple filter
GET /api/resource/Customer?filters=[["customer_type","=","Company"]]

# Multiple filters
GET /api/resource/Customer?filters=[["disabled","=",0],["customer_group","=","Premium"]]

# Complex filters
GET /api/resource/Sales Order?filters=[["grand_total",">",1000],["status","in",["Draft","Submitted"]]]
```

### Pagination
```bash
# Limit and offset
GET /api/resource/Customer?limit=20&offset=40

# Or use limit_start and limit_page_length
GET /api/resource/Customer?limit_start=40&limit_page_length=20
```

### Sorting
```bash
# Order by field
GET /api/resource/Customer?order_by=creation desc

# Multiple sort fields
GET /api/resource/Customer?order_by=customer_group asc,creation desc
```

## Advanced Filtering

### Filter Operators
- `=` : Equals
- `!=` : Not equals
- `>` : Greater than
- `<` : Less than
- `>=` : Greater than or equal
- `<=` : Less than or equal
- `like` : SQL LIKE
- `not like` : SQL NOT LIKE
- `in` : In list
- `not in` : Not in list
- `between` : Between two values
- `is` : IS NULL or IS NOT NULL

### Examples
```javascript
// JavaScript/Node.js example
const filters = [
    ["customer_name", "like", "%John%"],
    ["creation", "between", ["2024-01-01", "2024-12-31"]],
    ["status", "in", ["Active", "Prospective"]],
    ["credit_limit", ">", 10000]
];

const url = `https://site.com/api/resource/Customer?filters=${JSON.stringify(filters)}`;
```

## Custom Methods

### Call Whitelisted Methods
```python
# In Python file
@frappe.whitelist()
def get_customer_orders(customer):
    return frappe.db.get_list("Sales Order", 
        filters={"customer": customer},
        fields=["name", "grand_total", "status"]
    )
```

```bash
# Call custom method
curl -X POST "https://site.com/api/method/myapp.api.get_customer_orders" \
  -H "Authorization: token api_key:api_secret" \
  -d "customer=CUST-0001"
```

### RPC Style Calls
```bash
# Call any whitelisted function
curl -X POST "https://site.com/api/method/frappe.client.get_list" \
  -H "Authorization: token api_key:api_secret" \
  -H "Content-Type: application/json" \
  -d '{
    "doctype": "Customer",
    "filters": {"disabled": 0},
    "fields": ["name", "customer_name"],
    "limit": 10
  }'
```

## File Operations

### Upload File
```bash
# Upload file attachment
curl -X POST "https://site.com/api/method/upload_file" \
  -H "Authorization: token api_key:api_secret" \
  -F "file=@/path/to/file.pdf" \
  -F "doctype=Customer" \
  -F "docname=CUST-0001" \
  -F "fieldname=attachments"
```

### Get File
```bash
# Download file
curl "https://site.com/api/method/frappe.utils.file_manager.download_file" \
  -H "Authorization: token api_key:api_secret" \
  -d "file_url=/files/document.pdf"
```

## Batch Operations

### Multiple Creates
```python
# Python example for batch insert
@frappe.whitelist()
def batch_create_customers(customers):
    created = []
    for customer_data in customers:
        doc = frappe.get_doc({
            "doctype": "Customer",
            **customer_data
        })
        doc.insert()
        created.append(doc.name)
    frappe.db.commit()
    return created
```

### Bulk Update
```python
@frappe.whitelist()
def bulk_update_status(doctype, names, status):
    for name in names:
        frappe.db.set_value(doctype, name, "status", status)
    frappe.db.commit()
    return {"updated": len(names)}
```

## Response Format

### Success Response
```json
{
  "data": {
    "name": "CUST-0001",
    "customer_name": "John Doe",
    "email": "john@example.com",
    "creation": "2024-01-01 10:00:00",
    "modified": "2024-01-02 15:30:00"
  }
}
```

### Error Response
```json
{
  "exc_type": "ValidationError",
  "exception": "frappe.exceptions.ValidationError: Customer Email is required",
  "_server_messages": "[{\"message\": \"Customer Email is required\"}]"
}
```

### List Response
```json
{
  "data": [
    {"name": "CUST-0001", "customer_name": "John Doe"},
    {"name": "CUST-0002", "customer_name": "Jane Smith"}
  ]
}
```

## JavaScript/Fetch Examples

### GET Request
```javascript
async function getCustomer(name) {
    const response = await fetch(`https://site.com/api/resource/Customer/${name}`, {
        headers: {
            'Authorization': 'token api_key:api_secret'
        }
    });
    return await response.json();
}
```

### POST Request
```javascript
async function createCustomer(data) {
    const response = await fetch('https://site.com/api/resource/Customer', {
        method: 'POST',
        headers: {
            'Authorization': 'token api_key:api_secret',
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(data)
    });
    return await response.json();
}
```

### With Query Parameters
```javascript
async function getCustomerList(filters = {}) {
    const params = new URLSearchParams({
        fields: JSON.stringify(["name", "customer_name", "email"]),
        filters: JSON.stringify(filters),
        limit: 20
    });
    
    const response = await fetch(`https://site.com/api/resource/Customer?${params}`, {
        headers: {
            'Authorization': 'token api_key:api_secret'
        }
    });
    return await response.json();
}
```

## Python Client Examples

### Using requests library
```python
import requests
import json

class FrappeClient:
    def __init__(self, url, api_key, api_secret):
        self.url = url
        self.headers = {
            'Authorization': f'token {api_key}:{api_secret}',
            'Content-Type': 'application/json'
        }
    
    def get_doc(self, doctype, name):
        response = requests.get(
            f"{self.url}/api/resource/{doctype}/{name}",
            headers=self.headers
        )
        return response.json()
    
    def create_doc(self, doctype, data):
        response = requests.post(
            f"{self.url}/api/resource/{doctype}",
            headers=self.headers,
            data=json.dumps(data)
        )
        return response.json()
    
    def update_doc(self, doctype, name, data):
        response = requests.put(
            f"{self.url}/api/resource/{doctype}/{name}",
            headers=self.headers,
            data=json.dumps(data)
        )
        return response.json()
```

## Rate Limiting

Frappe implements rate limiting to prevent API abuse:

```python
# In site_config.json
{
    "rate_limit": {
        "window": 86400,
        "limit": 1000
    }
}
```

## Webhooks

Configure webhooks for DocType events:

```python
# Create webhook in Frappe
webhook = frappe.get_doc({
    "doctype": "Webhook",
    "webhook_doctype": "Customer",
    "webhook_docevent": "after_insert",
    "request_url": "https://your-endpoint.com/webhook",
    "request_method": "POST"
}).insert()
```

## Best Practices

1. **Use API Keys**: Never expose credentials in client-side code
2. **Implement Caching**: Cache frequently accessed data
3. **Batch Operations**: Use bulk endpoints for multiple operations
4. **Error Handling**: Always handle API errors gracefully
5. **Rate Limiting**: Implement client-side rate limiting
6. **Field Selection**: Only request needed fields
7. **Pagination**: Always paginate large datasets
8. **Compression**: Enable gzip compression for responses